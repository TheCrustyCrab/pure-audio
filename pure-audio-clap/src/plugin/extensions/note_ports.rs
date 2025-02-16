use crate::util::Writable;
use clap_sys::{
    ext::note_ports::{clap_note_port_info, clap_plugin_note_ports, CLAP_NOTE_DIALECT_CLAP},
    plugin::clap_plugin,
};
use pure_audio::IntoProcessor;
use std::fmt::Write;

pub(crate) trait NotePortsExtension<
    const NUM_INPUTS: usize,
    const NUM_OUTPUTS: usize,
    const NUM_CHANNELS: usize,
    const NUM_PARAMS: usize,
    A,
    Params,
    S,
>
{
    const EXT_NOTE_PORTS: clap_plugin_note_ports;
}

impl<
        P,
        const NUM_INPUTS: usize,
        const NUM_OUTPUTS: usize,
        const NUM_CHANNELS: usize,
        const NUM_PARAMS: usize,
        A,
        Params,
        S,
    > NotePortsExtension<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S> for P
where
    P: 'static + IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>,
{
    const EXT_NOTE_PORTS: clap_plugin_note_ports = clap_plugin_note_ports {
        count: Some(Self::note_count),
        get: Some(Self::note_get),
    };
}

trait NotePortsFunctions<
    const NUM_INPUTS: usize,
    const NUM_OUTPUTS: usize,
    const NUM_CHANNELS: usize,
    const NUM_PARAMS: usize,
    A,
    Params,
    S,
>
{
    unsafe extern "C" fn note_count(_clap_plugin: *const clap_plugin, is_input: bool) -> u32 {
        // currently fixed 1 note input
        if is_input {
            1
        } else {
            0
        }
    }

    unsafe extern "C" fn note_get(
        _clap_plugin: *const clap_plugin,
        index: u32,
        is_input: bool,
        info: *mut clap_note_port_info,
    ) -> bool {
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
}

impl<
        P,
        const NUM_INPUTS: usize,
        const NUM_OUTPUTS: usize,
        const NUM_CHANNELS: usize,
        const NUM_PARAMS: usize,
        A,
        Params,
        S,
    > NotePortsFunctions<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S> for P
where
    P: 'static + IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>,
{
}
