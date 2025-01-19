use std::ffi::{c_char, c_void};
use clap_sys::{entry::clap_plugin_entry, version::CLAP_VERSION};
use entry::Entry;
use factory::Factory;

pub mod entry;
pub mod factory;
mod plugin;
mod util;

// re-export clap-sys
pub use clap_sys;
// todo
// macro: pure_audio_clap_entry(id, name, is_instrument, process)

static ENTRY: Entry = Entry::new(&Factory::new(c"pureaudio.TestGain", c"PureAudioGain", false, gain::process) as *const _ as *const c_void);


#[no_mangle]
#[allow(non_upper_case_globals)]
pub static clap_entry: clap_plugin_entry = clap_plugin_entry {
    clap_version: CLAP_VERSION,
    init: Some(init),
    deinit: Some(deinit),
    get_factory: Some(get_factory),
};

// init and deinit in most cases are called once, in a matched pair, when the dso is loaded / unloaded.
// In some rare situations it may be called multiple times in a process, so the functions must be defensive,
// mutex locking and counting calls if undertaking non trivial non idempotent actions.
unsafe extern "C" fn init(path: *const c_char) -> bool {
    ENTRY.init(path)
}

unsafe extern "C" fn deinit() {
    ENTRY.deinit();
}

// The returned pointer must *not* be freed by the caller.
unsafe extern "C" fn get_factory(factory_id: *const c_char) -> *const c_void {
    ENTRY.get_factory(factory_id)
}

// temporary for testing
mod gain {
    use std::ops::Mul;
    use pure_audio::{AudioData, InputBuffer, OutputBuffer, ParameterAutomationRate, ParameterDescriptor, ProcessorParameter};

    #[derive(Copy, Clone)]
    pub struct GainVolumeParameter(f32);

    impl ProcessorParameter for GainVolumeParameter {
        const DESCRIPTOR: ParameterDescriptor = ParameterDescriptor {
            automation_rate: ParameterAutomationRate::K,
            default_value: 1.0,
            max_value: 1.0,
            min_value: 0.0,
            name: "Volume",
        };

        #[inline]
        fn from_parameter(value: f32) -> Self {
            GainVolumeParameter(value)
        }
        
        #[inline]
        fn value_to_text(value: f64, writer: &mut impl std::fmt::Write) -> bool {
            write!(writer, "{value}").is_ok()
        }
        
        #[inline]
        fn text_to_value(text: &str) -> Option<f64> {
            match text.parse::<f64>() {
                Ok(parsed_value) => {
                    Some(parsed_value)
                },
                Err(_) => None
            }
        }
    }

    impl Mul<f32> for GainVolumeParameter {
        type Output = f32;

        fn mul(self, rhs: f32) -> Self::Output {
            self.0 * rhs
        }
    }

    impl Mul<&f32> for GainVolumeParameter {
        type Output = f32;

        fn mul(self, rhs: &f32) -> Self::Output {
            self.0 * rhs
        }
    }

    impl Mul<GainVolumeParameter> for f32 {
        type Output = f32;

        fn mul(self, rhs: GainVolumeParameter) -> Self::Output {
            self * rhs.0
        }
    }

    impl Mul<GainVolumeParameter> for &f32 {
        type Output = f32;

        fn mul(self, rhs: GainVolumeParameter) -> Self::Output {
            self * rhs.0
        }
    }

    pub fn process(
        AudioData {
            inputs: InputBuffer([[input]]),
            outputs: OutputBuffer([[output]]),
            ..
        }: AudioData,
        volume: GainVolumeParameter,
    ) {
        for (input_sample, output_sample) in input.iter().zip(output) {
            *output_sample = input_sample * volume;
        }
    }
}