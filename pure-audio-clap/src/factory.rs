use std::{ffi::{c_char, CStr}, marker::PhantomData};
use clap_sys::{factory::plugin_factory::clap_plugin_factory, host::clap_host, plugin::{clap_plugin, clap_plugin_descriptor}, plugin_features::{CLAP_PLUGIN_FEATURE_AUDIO_EFFECT, CLAP_PLUGIN_FEATURE_INSTRUMENT}, version::CLAP_VERSION};
use pure_audio::IntoProcessor;
use crate::plugin::{bridge, PluginWrapper};

// layout must match with clap_plugin_factory: repr(C) + clap_plugin_factory as first field
#[repr(C)]
pub struct Factory<P, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, S> {
    inner: clap_plugin_factory,
    descriptor: clap_plugin_descriptor,
    p: P,
    marker: PhantomData<(Params, S)>
}

const FEATURES_EFFECT: &[*const c_char] = &[
    CLAP_PLUGIN_FEATURE_AUDIO_EFFECT.as_ptr(),
    core::ptr::null()
];

const FEATURES_INSTRUMENT: &[*const c_char] = &[
    CLAP_PLUGIN_FEATURE_INSTRUMENT.as_ptr(),
    core::ptr::null()
];

impl<P, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, S> Factory<P, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S>
where 
    P: 'static + Copy + IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S>
{
    pub const fn new(id: &'static CStr, name: &'static CStr, p: P) -> Self {
        Self {
            inner: clap_plugin_factory {
                create_plugin: Some(Self::create_plugin),
                get_plugin_count: Some(Self::get_plugin_count),
                get_plugin_descriptor: Some(Self::get_plugin_descriptor)
            },
            descriptor: clap_plugin_descriptor {
                clap_version: CLAP_VERSION,
                id: id.as_ptr(),
                name: name.as_ptr(),
                vendor: c"".as_ptr(),
                url: c"".as_ptr(),
                manual_url: c"".as_ptr(),
                support_url: c"".as_ptr(),
                version: c"".as_ptr(),
                description: c"".as_ptr(),
                features: if NUM_INPUTS == 0 { FEATURES_INSTRUMENT.as_ptr() } else { FEATURES_EFFECT.as_ptr() },
            },
            p,
            marker: PhantomData
        }
    }

    // Get the number of plugins available.
    // [thread-safe]
    unsafe extern "C" fn get_plugin_count(_factory: *const clap_plugin_factory) -> u32 {
        1
    }

    // Retrieves a plugin descriptor by its index.
    // Returns null in case of error.
    // The descriptor must not be freed.
    // [thread-safe]
    unsafe extern "C" fn get_plugin_descriptor(factory: *const clap_plugin_factory, _index: u32) -> *const clap_plugin_descriptor {
        &Self::get_factory(factory).descriptor
    }

    // Create a clap_plugin by its plugin_id.
    // The returned pointer must be freed by calling plugin->destroy(plugin);
    // The plugin is not allowed to use the host callbacks in the create method.
    // Returns null in case of error.
    // [thread-safe]
    unsafe extern "C" fn create_plugin(factory: *const clap_plugin_factory, _host: *const clap_host, id: *const i8) -> *const clap_plugin {
        let factory = Self::get_factory(factory);
        let descriptor = &factory.descriptor;
        if CStr::from_ptr(id) == CStr::from_ptr(descriptor.id) {
            let wrapper = Box::new(PluginWrapper::new(factory.p));
            let plugin = Box::new(clap_plugin {            
                desc: descriptor,
                plugin_data: Box::into_raw(wrapper).cast(),
                init: Some(bridge::init::<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S, P::Out>),
                destroy: Some(bridge::destroy::<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S, P>),
                activate: Some(bridge::activate::<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S, P>),
                deactivate: Some(bridge::deactivate::<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S, P::Out>),
                start_processing: Some(bridge::start_processing::<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S, P::Out>),
                stop_processing: Some(bridge::stop_processing::<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S, P::Out>),
                reset: Some(bridge::reset::<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S, P::Out>),
                process: Some(bridge::process::<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S, P>),
                get_extension: Some(bridge::get_extension::<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S, P>),
                on_main_thread: Some(bridge::on_main_thread::<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S, P::Out>),
            });
            
            Box::into_raw(plugin)
        } else {
            core::ptr::null()
        }
    }

    #[inline]
    unsafe fn get_factory<'a>(factory: *const clap_plugin_factory) -> &'a Self {
        &*(factory as *const Self)
    }
}