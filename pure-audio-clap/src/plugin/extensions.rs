use clap_sys::{events::{clap_input_events, clap_output_events}, ext::{audio_ports::{clap_audio_port_info, clap_plugin_audio_ports, CLAP_AUDIO_PORT_IS_MAIN}, note_ports::{clap_note_port_info, clap_plugin_note_ports, CLAP_NOTE_DIALECT_CLAP}, params::{clap_param_info, clap_plugin_params, CLAP_PARAM_IS_AUTOMATABLE, CLAP_PARAM_IS_MODULATABLE}}, id::CLAP_INVALID_ID, plugin::clap_plugin};
use pure_audio::IntoProcessor;
use std::{ffi::{c_char, CStr}, fmt::Write, sync::atomic::Ordering};
use crate::util::Writable;
use super::get_plugin_data;

// todo: split in module per extension
pub trait Extensions<const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, S> {
    const EXT_AUDIO_PORTS: clap_plugin_audio_ports;
    const EXT_NOTE_PORTS: clap_plugin_note_ports;
    const EXT_PARAMS: clap_plugin_params;

    unsafe extern "C" fn audio_count(_clap_plugin: *const clap_plugin, is_input: bool) -> u32 {
        if is_input { NUM_INPUTS as u32 } else { NUM_OUTPUTS as u32 }
    }
    
    unsafe extern "C" fn audio_get(_clap_plugin: *const clap_plugin, index: u32, is_input: bool, info: *mut clap_audio_port_info) -> bool {
        let max_index = if is_input { NUM_INPUTS } else { NUM_OUTPUTS };
        if index as usize + 1  > max_index {
            false
        } else {
            let info = &mut *info;
            info.channel_count = NUM_CHANNELS as u32;
            if index == 0 {
                info.flags |= CLAP_AUDIO_PORT_IS_MAIN;
            }
            info.id = index;
            info.in_place_pair = CLAP_INVALID_ID;
            let direction = if is_input { "input" } else { "output" };
            write!(info.name.writable(), "audio {direction} {index}").unwrap();
            true
        }
    }
    
    unsafe extern "C" fn note_count(_clap_plugin: *const clap_plugin, is_input: bool) -> u32 {
        // currently fixed 1 note input
        if is_input { 1 } else { 0 }
    }
    
    unsafe extern "C" fn note_get(_clap_plugin: *const clap_plugin, index: u32, is_input: bool, info: *mut clap_note_port_info) -> bool {
        if !is_input || index > 0 {
            false
        } else {
            let info = &mut *info;
            info.id = 0;
            write!(info.name.writable(), "note port 0").unwrap();
            info.preferred_dialect = CLAP_NOTE_DIALECT_CLAP;
            info.supported_dialects = CLAP_NOTE_DIALECT_CLAP;
            true
        }
    }

    // Returns the number of parameters.
    // [main-thread]
    unsafe extern "C" fn params_count(_clap_plugin: *const clap_plugin) -> u32 {
        NUM_PARAMS as u32
    }
}

impl<P, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, S> Extensions<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S> for P
where
    P: 'static + IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S>
{
    const EXT_AUDIO_PORTS: clap_plugin_audio_ports = clap_plugin_audio_ports {
        count: Some(Self::audio_count),
        get: Some(Self::audio_get),
    };

    const EXT_NOTE_PORTS: clap_plugin_note_ports = clap_plugin_note_ports {
        count: Some(Self::note_count),
        get: Some(Self::note_get),
    };

    const EXT_PARAMS: clap_plugin_params = clap_plugin_params {
        count: Some(Self::params_count),
        get_info: Some(Self::params_get_info),
        get_value: Some(Self::params_get_value),
        value_to_text: Some(Self::params_value_to_text),
        text_to_value: Some(Self::params_text_to_value),
        flush: Some(Self::params_flush)
    };
}

trait ParamFunctions<const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, S> {
    unsafe extern "C" fn params_get_info(clap_plugin: *const clap_plugin, index: u32, info: *mut clap_param_info) -> bool;
    unsafe extern "C" fn params_get_value(clap_plugin: *const clap_plugin, id: u32, value: *mut f64) -> bool;
    unsafe extern "C" fn params_flush(clap_plugin: *const clap_plugin, in_events: *const clap_input_events, out_events: *const clap_output_events);
    unsafe extern "C" fn params_value_to_text(clap_plugin: *const clap_plugin, id: u32, value: f64, out_buffer: *mut c_char, out_buffer_capacity: u32) -> bool;
    unsafe extern "C" fn params_text_to_value(clap_plugin: *const clap_plugin, id: u32, text: *const c_char, value: *mut f64) -> bool;
}

impl<P, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, S> ParamFunctions<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S> for P
where
    P: 'static + IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S>
{
    // Copies the parameter's info to param_info.
    // Returns true on success.
    // [main-thread]
    unsafe extern "C" fn params_get_info(_clap_plugin: *const clap_plugin, index: u32, info: *mut clap_param_info) -> bool {
        if index as usize + 1 > NUM_PARAMS {
            false
        } else {
            let (desc, ..) = P::PARAM_DESCRIPTORS[index as usize];
            let info = &mut *info;
            info.id = index;
            // todo: add to ParameterDescriptor if needed (specific to CLAP)
            info.flags = CLAP_PARAM_IS_AUTOMATABLE | CLAP_PARAM_IS_MODULATABLE;
            info.default_value = desc.default_value as f64;
            info.max_value = desc.max_value as f64;
            info.min_value = desc.min_value as f64;
            write!(info.name.writable(), "{}", desc.name).unwrap();

            true
        }
    }

    // Writes the parameter's current value to out_value.
    // Returns true on success.
    // [main-thread]
    unsafe extern "C" fn params_get_value(clap_plugin: *const clap_plugin, id: u32, value: *mut f64) -> bool {
        let plugin = get_plugin_data::<P, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S>(clap_plugin);
        *value = plugin.parameters[id as usize].load(Ordering::Relaxed);
        
        true
    }

    // Converts the null-terminated UTF-8 param_value_text into a double and writes it to out_value.
    // The host can use this to convert user input into a parameter value.
    // Returns true on success.
    // [main-thread]
    unsafe extern "C" fn params_text_to_value(_clap_plugin: *const clap_plugin, id: u32, text: *const c_char, value: *mut f64) -> bool {
        match CStr::from_ptr(text).to_str() {
            Ok(text) => {
                match P::parameter_text_to_value(id as usize, text) {
                    Some(v) => {
                        *value = v;
                        true
                    },
                    None => false
                }
            },
            Err(_) => false,
        }
    }

    // Fills out_buffer with a null-terminated UTF-8 string that represents the parameter at the
    // given 'value' argument. eg: "2.3 kHz". The host should always use this to format parameter
    // values before displaying it to the user.
    // Returns true on success.
    // [main-thread]
    unsafe extern "C" fn params_value_to_text(_clap_plugin: *const clap_plugin, id: u32, value: f64, out_buffer: *mut c_char, out_buffer_capacity: u32) -> bool {
        let out = std::slice::from_raw_parts_mut(out_buffer, out_buffer_capacity as usize);
        P::parameter_value_to_text(id as usize, value, &mut out.writable())
    }
    
    // Flushes a set of parameter changes.
    // This method must not be called concurrently to clap_plugin->process().
    //
    // Note: if the plugin is processing, then the process() call will already achieve the
    // parameter update (bi-directional), so a call to flush isn't required, also be aware
    // that the plugin may use the sample offset in process(), while this information would be
    // lost within flush().
    //
    // [active ? audio-thread : main-thread]
    unsafe extern "C" fn params_flush(clap_plugin: *const clap_plugin, in_events: *const clap_input_events, _out_events: *const clap_output_events) {
        let plugin = get_plugin_data::<P, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S>(clap_plugin);
        plugin.handle_input_events(in_events);
    }
}