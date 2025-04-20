pub mod bridge;
mod extensions;

use std::{array, marker::PhantomData, sync::atomic::{AtomicU32, Ordering}};
use clap_sys::{events::{clap_event_note, clap_event_param_value, clap_input_events, CLAP_CORE_EVENT_SPACE_ID, CLAP_EVENT_NOTE_OFF, CLAP_EVENT_NOTE_ON, CLAP_EVENT_PARAM_VALUE}, plugin::clap_plugin};
use pure_audio::{AutomationRate, Event, IntoProcessor, OutEvent, Processor};

pub struct PluginWrapper<P, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, A, Params, S>
where
    P: 'static + IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>
{
    events: Vec<Event>,
    out_events: Vec<OutEvent>,
    // current parameter values
    // main thread (params_get_value, read) and audio thread (process, read/write) can access them concurrently so synchronization is needed
    parameters: [AtomicU32; NUM_PARAMS],
    // parameters with sample precision
    // though both main thread (params_flush) and audio thread (process) can write to it, the host may not do it concurrently, so no synchronization is needed
    parameters_per_sample: Option<[Option<Vec<u32>>; NUM_PARAMS]>,
    last_process_changed_parameters: [bool; NUM_PARAMS],
    processor: P::Out,
    marker: PhantomData<(A, Params, S)>
}

impl<P, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, A, Params, S> PluginWrapper<P, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>
where 
    P: 'static + IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>
{
    pub fn new(processor: P) -> Self {
        let initial_parameters = array::from_fn(|param_index| {
            AtomicU32::new(P::parameter_f64_to_value(param_index, P::PARAM_DESCRIPTORS[param_index].0.default_value as f64))
        });
        Self {
            events: vec![],
            out_events: vec![],
            parameters: initial_parameters,
            parameters_per_sample: None,
            last_process_changed_parameters: [false; NUM_PARAMS],
            processor: processor.into_processor(),
            marker: PhantomData
        }
    }

    pub fn activate(&mut self, sample_rate: f64, min_frame_count: usize, max_frame_count: usize) {
        self.processor.activate(sample_rate as f32, min_frame_count, max_frame_count);

        match &mut self.parameters_per_sample {
            Some(parameters) => {
                // resize a-rate parameter vecs
                let per_sample_parameters = 
                    parameters
                        .iter_mut()
                        .zip(P::PARAM_DESCRIPTORS)
                        .filter(|&(.., (.., automation_rate))| if let AutomationRate::A = automation_rate { true } else { false });
                
                for (values, ..) in per_sample_parameters {
                    let values = values.as_mut().unwrap(); // must be initialized at this point
                    let last_value = values.last().unwrap();
                    values.resize(max_frame_count as usize, *last_value); // append with last value in case of expand
                }
            },
            None => {
                // initialize
                let initial_parameters = array::from_fn(|param_index| {
                    let (desc, automation_rate) = P::PARAM_DESCRIPTORS[param_index];
                    match automation_rate {
                        AutomationRate::A => Some(vec![P::parameter_f64_to_value(param_index, desc.default_value as f64); max_frame_count]),
                        AutomationRate::K => None,
                    }
                });
                self.parameters_per_sample = Some(initial_parameters)
            }
        }
    }

    #[inline]
    /// map input events to our own event structure and parameters
    /// called from flush (main thread) and process (audio thread)
    pub unsafe fn handle_input_events(&mut self, input_events: *const clap_input_events) {        
        let input_events = &*input_events;
        let input_event_count = input_events.size.unwrap()(input_events);
        let mut params_changed = false;

        for i in 0..input_event_count {
            let event_ptr = input_events.get.unwrap()(input_events, i);
            let event = &*event_ptr;
            if event.space_id != CLAP_CORE_EVENT_SPACE_ID {
                continue;
            }
            if event.type_ == CLAP_EVENT_NOTE_ON {
                let note_event = &*(event_ptr as *const clap_event_note);
                // todo: correct types
                self.events.push(Event::NoteOn { 
                    port_index: note_event.port_index as i32,
                    channel: note_event.channel as i32,
                    key: note_event.key as u8,
                    note_id: note_event.note_id as i32,
                    velocity: (note_event.velocity * 127.0) as u8 }
                );// todo: use normalized velocity everywhere
            } else if event.type_ == CLAP_EVENT_NOTE_OFF {
                let note_event = &*(event_ptr as *const clap_event_note);
                // todo: correct types
                self.events.push(Event::NoteOff { 
                    port_index: note_event.port_index as i32,
                    channel: note_event.channel as i32,
                    key: note_event.key as u8, 
                    note_id: note_event.note_id as i32,
                    velocity: (note_event.velocity * 127.0) as u8
                });// todo: use normalized velocity everywhere
            } else if event.type_ == CLAP_EVENT_PARAM_VALUE {
                let param_value_event = &*(event_ptr as *const clap_event_param_value);
                let param_index = param_value_event.param_id as usize;
                self.parameters[param_index].store(P::parameter_f64_to_value(param_index, param_value_event.value), Ordering::Relaxed);

                // map change to sample precise parameters
                if let Some(values) = &mut self.parameters_per_sample.as_mut().unwrap()[param_value_event.param_id as usize] {
                    let offset = event.time as usize;
                    for tail_value in &mut values[offset..] {
                        *tail_value = P::parameter_f64_to_value(param_index, param_value_event.value);
                    }
                    self.last_process_changed_parameters[param_value_event.param_id as usize] = true;
                }

                params_changed = true;
            }
            
            if params_changed {                
                self.events.push(Event::ParamsChanged);
            }
        }
    }
}

#[inline]
unsafe fn get_plugin_data<'a, P, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, A, Params, S>(plugin: *const clap_plugin) 
-> &'a mut PluginWrapper<P, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S> 
where 
    P: IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>
{
    &mut *((&*plugin).plugin_data as *mut PluginWrapper<P, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>)
}