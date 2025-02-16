use crate::util::Writable;
use clap_sys::{
    ext::audio_ports::{clap_audio_port_info, clap_plugin_audio_ports, CLAP_AUDIO_PORT_IS_MAIN},
    id::CLAP_INVALID_ID,
    plugin::clap_plugin,
};
use pure_audio::IntoProcessor;
use std::fmt::Write;

pub(crate) trait AudioPortsExtension<
    const NUM_INPUTS: usize,
    const NUM_OUTPUTS: usize,
    const NUM_CHANNELS: usize,
    const NUM_PARAMS: usize,
    A,
    Params,
    S,
>
{
    const EXT_AUDIO_PORTS: clap_plugin_audio_ports;
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
    > AudioPortsExtension<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S> for P
where
    P: 'static + IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>,
{
    const EXT_AUDIO_PORTS: clap_plugin_audio_ports = clap_plugin_audio_ports {
        count: Some(Self::audio_count),
        get: Some(Self::audio_get),
    };
}

trait AudioPortsFunctions<
    const NUM_INPUTS: usize,
    const NUM_OUTPUTS: usize,
    const NUM_CHANNELS: usize,
    const NUM_PARAMS: usize,
    A,
    Params,
    S,
>
{
    unsafe extern "C" fn audio_count(_clap_plugin: *const clap_plugin, is_input: bool) -> u32 {
        if is_input {
            NUM_INPUTS as u32
        } else {
            NUM_OUTPUTS as u32
        }
    }

    unsafe extern "C" fn audio_get(
        _clap_plugin: *const clap_plugin,
        index: u32,
        is_input: bool,
        info: *mut clap_audio_port_info,
    ) -> bool {
        let max_index = if is_input { NUM_INPUTS } else { NUM_OUTPUTS };
        if index as usize + 1 > max_index {
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
    > AudioPortsFunctions<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S> for P
where
    P: 'static + IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>,
{
}
