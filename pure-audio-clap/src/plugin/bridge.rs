use std::{array, ffi::{c_void, CStr}, slice};
use clap_sys::{ext::{audio_ports::{clap_plugin_audio_ports, CLAP_EXT_AUDIO_PORTS}, note_ports::{clap_plugin_note_ports, CLAP_EXT_NOTE_PORTS}, params::{clap_plugin_params, CLAP_EXT_PARAMS}}, plugin::clap_plugin, process::{clap_process, clap_process_status, CLAP_PROCESS_CONTINUE}};
use pure_audio::{IntoProcessor, Processor};
use super::{get_plugin_data, extensions::Extensions, PluginWrapper};

// bridge between unsafe CLAP API and inner Plugin
pub(crate) unsafe extern "C" fn init<const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, S, P>(plugin: *const clap_plugin) -> bool
where 
    P: 'static + Processor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params>
{
    true
}

pub(crate) unsafe extern "C" fn destroy<const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, S, P>(plugin: *const clap_plugin)
where 
    P: 'static + IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S>
{
    // free plugin and plugin_data
    let plugin = Box::from_raw(plugin as *mut clap_plugin);
    let _ = Box::from_raw(plugin.plugin_data as *mut PluginWrapper<P, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S>);
}

pub(crate) unsafe extern "C" fn activate<const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, S, P>(plugin: *const clap_plugin, sample_rate: f64, min_frame_count: u32, max_frame_count: u32) -> bool
where 
    P: 'static + IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S>
{
    let this = get_plugin_data::<P, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S>(plugin);
    this.activate(sample_rate);
    true
}

pub(crate) unsafe extern "C" fn deactivate<const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, S, P>(plugin: *const clap_plugin)
where 
    P: 'static + Processor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params>
{
}

pub(crate) unsafe extern "C" fn start_processing<const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, S, P>(plugin: *const clap_plugin) -> bool
where 
    P: 'static + Processor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params>
{
    true
}

pub(crate) unsafe extern "C" fn stop_processing<const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, S, P>(plugin: *const clap_plugin)
where 
    P: 'static + Processor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params>
{
}

pub(crate) unsafe extern "C" fn reset<const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, S, P>(plugin: *const clap_plugin)
where 
    P: 'static + Processor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params>
{
}

pub(crate) unsafe extern "C" fn process<const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, S, P>(plugin: *const clap_plugin, process: *const clap_process) -> clap_process_status 
where 
    P: 'static + IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S>
{
    let this = get_plugin_data::<P, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S>(plugin);
    let process = &*process;

    // map events and parameters
    this.handle_input_events(process.in_events);

    let frames_count = process.frames_count as usize;

    // todo: to avoid conversions, is it better to use f64 for parameters everywhere, including wasm?
    let parameters = this.parameters.each_ref().map(|p| p.load(std::sync::atomic::Ordering::Relaxed) as f32);
    
    let inputs = array::from_fn(|input_index|{
        let input_ptr = process.audio_inputs.add(input_index);
        let input_channels_ptr = (*input_ptr).data32;
        let input_channels = array::from_fn(|channel_index| {
            let channel = *(input_channels_ptr.add(channel_index));
            let samples = slice::from_raw_parts(channel, frames_count);
            samples
        });
        input_channels
    });
    
    let outputs = array::from_fn(|output_index| {
        let output_ptr = process.audio_outputs.add(output_index);
        let output_channels_ptr = (*output_ptr).data32 as *const *mut f32;
        let output_channels = array::from_fn(|channel_index| {
            let channel = *(output_channels_ptr.add(channel_index));
            let samples = slice::from_raw_parts_mut(channel, frames_count);
            samples
        });
        output_channels
    });

    this.processor.process(&inputs, outputs, &parameters, &this.events);
    this.events.clear();

    CLAP_PROCESS_CONTINUE
}

pub(crate) unsafe extern "C" fn get_extension<const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, S, P>(plugin: *const clap_plugin, id: *const i8) -> *const c_void
where 
    P: 'static + IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S>
{
    let id = CStr::from_ptr(id);
    if id == CLAP_EXT_AUDIO_PORTS {
        (&P::EXT_AUDIO_PORTS as *const clap_plugin_audio_ports).cast()
    } else if id == CLAP_EXT_NOTE_PORTS {
        (&P::EXT_NOTE_PORTS as *const clap_plugin_note_ports).cast()
    } else if id == CLAP_EXT_PARAMS {
        (&P::EXT_PARAMS as *const clap_plugin_params).cast()
    } else {
        core::ptr::null()
    }
}

pub(crate) unsafe extern "C" fn on_main_thread<const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, S, P>(plugin: *const clap_plugin)
where 
    P: 'static + Processor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params>
{
}