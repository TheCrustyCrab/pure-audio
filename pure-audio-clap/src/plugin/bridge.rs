use std::{array, ffi::{c_void, CStr}, slice, sync::atomic::Ordering};
use clap_sys::{events::{clap_event_header, clap_event_note, CLAP_CORE_EVENT_SPACE_ID, CLAP_EVENT_NOTE_END}, ext::{audio_ports::{clap_plugin_audio_ports, CLAP_EXT_AUDIO_PORTS}, note_ports::{clap_plugin_note_ports, CLAP_EXT_NOTE_PORTS}, params::{clap_plugin_params, CLAP_EXT_PARAMS}}, plugin::clap_plugin, process::{clap_process, clap_process_status, CLAP_PROCESS_CONTINUE}};
use pure_audio::{IntoProcessor, OutEvent, Processor};
use super::{get_plugin_data, extensions::{audio_ports::AudioPortsExtension, note_ports::NotePortsExtension, params::ParamsExtension}, PluginWrapper};

// bridge between unsafe CLAP API and inner Plugin
pub(crate) unsafe extern "C" fn init<const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, S, P>(_plugin: *const clap_plugin) -> bool
where 
    P: 'static + Processor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params>
{
    true
}

pub(crate) unsafe extern "C" fn destroy<const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, A, Params, S, P>(plugin: *const clap_plugin)
where 
    P: 'static + IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>
{
    // free plugin and plugin_data
    let plugin = Box::from_raw(plugin as *mut clap_plugin);
    let _ = Box::from_raw(plugin.plugin_data as *mut PluginWrapper<P, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>);
}

pub(crate) unsafe extern "C" fn activate<const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, A, Params, S, P>(plugin: *const clap_plugin, sample_rate: f64, min_frame_count: u32, max_frame_count: u32) -> bool
where 
    P: 'static + IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>
{
    let this = get_plugin_data::<P, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>(plugin);
    this.activate(sample_rate, min_frame_count as usize, max_frame_count as usize);
    true
}

pub(crate) unsafe extern "C" fn deactivate<const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, S, P>(_plugin: *const clap_plugin)
where 
    P: 'static + Processor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params>
{
}

pub(crate) unsafe extern "C" fn start_processing<const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, S, P>(_plugin: *const clap_plugin) -> bool
where 
    P: 'static + Processor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params>
{
    true
}

pub(crate) unsafe extern "C" fn stop_processing<const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, S, P>(_plugin: *const clap_plugin)
where 
    P: 'static + Processor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params>
{
}

pub(crate) unsafe extern "C" fn reset<const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, S, P>(_plugin: *const clap_plugin)
where 
    P: 'static + Processor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params>
{
}

pub(crate) unsafe extern "C" fn process<const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, A, Params, S, P>(plugin: *const clap_plugin, process: *const clap_process) -> clap_process_status 
where 
    P: 'static + IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>
{
    let this = get_plugin_data::<P, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>(plugin);
    let process = &*process;

    // equalize changed a-rate param values again
    for (changed, values) in this.last_process_changed_parameters.iter_mut().filter(|changed| **changed).zip(this.parameters_per_sample.as_mut().unwrap()) {
        let values = values.as_mut().unwrap(); // must be initialized at this point
        let last_value = *values.last().unwrap_unchecked();
        values.fill(last_value);
        *changed = false;
    }

    // map events and parameters
    this.handle_input_events(process.in_events);

    let frames_count = process.frames_count as usize;

    let parameters = this.parameters.each_ref().map(|p| p.load(Ordering::Relaxed));
    
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
        let output_channels_ptr = (*output_ptr).data32;
        let output_channels = array::from_fn(|channel_index| {
            let channel = *(output_channels_ptr.add(channel_index));
            let samples = slice::from_raw_parts_mut(channel, frames_count);
            samples
        });
        output_channels
    });

    let parameters_per_sample = this.parameters_per_sample.as_ref().unwrap().each_ref().map(|p| p.as_ref().map(|p| p.as_slice()));
    this.processor.process(inputs, outputs, &parameters, &parameters_per_sample, &this.events, &mut this.out_events);

    let out_events = &*process.out_events;
    for out_event in this.out_events.iter() {
        let clap_out_event = match out_event {
            OutEvent::NoteEnd { port_index, channel, key, note_id, velocity } => {
                clap_event_note {
                    header: clap_event_header {
                        flags: 0,
                        size: std::mem::size_of::<clap_event_note>() as u32,
                        space_id: CLAP_CORE_EVENT_SPACE_ID,
                        time: process.frames_count - 1,
                        type_: CLAP_EVENT_NOTE_END
                    },
                    channel: *channel as i16,
                    key: *key as i16,
                    note_id: *note_id,
                    port_index: *port_index as i16,
                    velocity: *velocity as f64
                }
            },
        };
        out_events.try_push.unwrap()(process.out_events, &clap_out_event.header);
    }

    this.events.clear();
    this.out_events.clear();

    CLAP_PROCESS_CONTINUE
}

pub(crate) unsafe extern "C" fn get_extension<const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, A, Params, S, P>(_plugin: *const clap_plugin, id: *const i8) -> *const c_void
where 
    P: 'static + IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>
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

pub(crate) unsafe extern "C" fn on_main_thread<const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, S, P>(_plugin: *const clap_plugin)
where 
    P: 'static + Processor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params>
{
}