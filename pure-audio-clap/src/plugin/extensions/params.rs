use std::ffi::CStr;
use std::{ffi::c_char, sync::atomic::Ordering};
use std::fmt::Write;
use clap_sys::{events::{clap_input_events, clap_output_events}, ext::params::{clap_param_info, clap_plugin_params, CLAP_PARAM_IS_AUTOMATABLE, CLAP_PARAM_IS_ENUM, CLAP_PARAM_IS_MODULATABLE, CLAP_PARAM_IS_STEPPED}, plugin::clap_plugin};
use pure_audio::{IntoProcessor, ParameterKind};
use crate::plugin::get_plugin_data;
use crate::util::Writable;

pub trait ParamsExtension<const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, A, Params, S> {
    const EXT_PARAMS: clap_plugin_params;
}

impl<P, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, A, Params, S> ParamsExtension<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S> for P
where
    P: 'static + IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>
{
    const EXT_PARAMS: clap_plugin_params = clap_plugin_params {
        count: Some(Self::params_count),
        get_info: Some(Self::params_get_info),
        get_value: Some(Self::params_get_value),
        value_to_text: Some(Self::params_value_to_text),
        text_to_value: Some(Self::params_text_to_value),
        flush: Some(Self::params_flush)
    };
}

trait ParamsFunctions<const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, A, Params, S> {
    unsafe extern "C" fn params_count(_clap_plugin: *const clap_plugin) -> u32;
    unsafe extern "C" fn params_get_info(clap_plugin: *const clap_plugin, index: u32, info: *mut clap_param_info) -> bool;
    unsafe extern "C" fn params_get_value(clap_plugin: *const clap_plugin, id: u32, value: *mut f64) -> bool;
    unsafe extern "C" fn params_flush(clap_plugin: *const clap_plugin, in_events: *const clap_input_events, out_events: *const clap_output_events);
    unsafe extern "C" fn params_value_to_text(clap_plugin: *const clap_plugin, id: u32, value: f64, out_buffer: *mut c_char, out_buffer_capacity: u32) -> bool;
    unsafe extern "C" fn params_text_to_value(clap_plugin: *const clap_plugin, id: u32, text: *const c_char, value: *mut f64) -> bool;
}

impl<P, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, A, Params, S> ParamsFunctions<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S> for P
where
    P: 'static + IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>
{
    
    // Returns the number of parameters.
    // [main-thread]
    unsafe extern "C" fn params_count(_clap_plugin: *const clap_plugin) -> u32 {
        P::PARAMS_COUNT as u32
    }

    // Copies the parameter's info to param_info.
    // Returns true on success.
    // [main-thread]
    unsafe extern "C" fn params_get_info(_clap_plugin: *const clap_plugin, index: u32, info: *mut clap_param_info) -> bool {
        if index as usize + 1 > NUM_PARAMS {
            false
        } else {
            let (real_index, (desc, ..), ..) = P::get_local_param_descriptor(index as usize);
            let info = &mut *info;
            // it's crucial to assign the real index to the id, as the other param functions depend on it
            info.id = real_index as u32;
            // todo: add to ParameterDescriptor if needed (specific to CLAP)
            info.flags = CLAP_PARAM_IS_AUTOMATABLE | CLAP_PARAM_IS_MODULATABLE;
            if let ParameterKind::Bool | ParameterKind::Enum(_) | ParameterKind::I32 | ParameterKind::U32 = desc.kind {
                info.flags |= CLAP_PARAM_IS_STEPPED; 

                if let ParameterKind::Enum(_) = desc.kind {
                    // This parameter represents an enumerated value.
                    // If you set this flag, then you must set CLAP_PARAM_IS_STEPPED too.
                    // All values from min to max must not have a blank value_to_text().
                    info.flags |= CLAP_PARAM_IS_ENUM;
                }
            }
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
        let plugin = get_plugin_data::<P, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>(clap_plugin);
        let index = id as usize;
        *value = P::parameter_value_to_f64(index, plugin.parameters[index].load(Ordering::Relaxed));
        
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
        let plugin = get_plugin_data::<P, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>(clap_plugin);
        plugin.handle_input_events(in_events);
    }
}