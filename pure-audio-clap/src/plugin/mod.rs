pub mod bridge;
mod extensions;

use std::marker::PhantomData;
use atomic_float::AtomicF64;
use clap_sys::{events::{clap_event_note, clap_event_param_value, clap_input_events, CLAP_CORE_EVENT_SPACE_ID, CLAP_EVENT_NOTE_OFF, CLAP_EVENT_NOTE_ON, CLAP_EVENT_PARAM_VALUE}, plugin::clap_plugin};
use pure_audio::{Event, IntoProcessor, Processor};

pub struct PluginWrapper<P, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, S>
where
    P: 'static + IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S>
{
    events: Vec<Event>,
    parameters: [AtomicF64; NUM_PARAMS],
    processor: P::Out,
    marker: PhantomData<(Params, S)>
}

impl<P, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, S> PluginWrapper<P, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S>
where 
    P: 'static + IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S>
{
    pub fn new(processor: P) -> Self {
        Self {
            events: vec![],
            parameters: P::PARAM_DESCRIPTORS.map(|desc| AtomicF64::new(desc.default_value as f64)),
            processor: processor.into_processor(0.0),
            marker: PhantomData
        }
    }

    pub fn activate(&mut self, sample_rate: f64) {
        self.processor.set_sample_rate(sample_rate as f32);
    }

    #[inline]
    /// map input events to our own event structure and parameters
    /// called from flush (main thread) and process (audio thread)
    pub unsafe fn handle_input_events(&mut self, input_events: *const clap_input_events) {        
        let input_events = &*input_events;
        let input_event_count = input_events.size.unwrap()(input_events);

        for i in 0..input_event_count {
            let event_ptr = input_events.get.unwrap()(input_events, i);
            let event = &*event_ptr;
            if event.space_id != CLAP_CORE_EVENT_SPACE_ID {
                continue;
            }
            if event.type_ == CLAP_EVENT_NOTE_ON {
                let note_event = &*(event_ptr as *const clap_event_note);
                // todo: correct types + note_id, channel, time
                self.events.push(Event::NoteOn { key: note_event.key as u8, velocity: (note_event.velocity * 127.0) as u8 });// todo: use normalized velocity everywhere
            } else if event.type_ == CLAP_EVENT_NOTE_OFF {
                let note_event = &*(event_ptr as *const clap_event_note);
                // todo: correct types + note_id, channel, time
                self.events.push(Event::NoteOff { key: note_event.key as u8, velocity: (note_event.velocity * 127.0) as u8 });// todo: use normalized velocity everywhere
            } else if event.type_ == CLAP_EVENT_PARAM_VALUE {
                let param_value_event = &*(event_ptr as *const clap_event_param_value);
                self.parameters[param_value_event.param_id as usize].store(param_value_event.value, std::sync::atomic::Ordering::Relaxed);
            }
        }
    }
}

#[inline]
unsafe fn get_plugin_data<'a, P, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, S>(plugin: *const clap_plugin) 
-> &'a mut PluginWrapper<P, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S> 
where 
    P: IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S>
{
    &mut *((&*plugin).plugin_data as *mut PluginWrapper<P, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S>)
}