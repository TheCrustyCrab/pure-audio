use pure_audio_clap::{
    clap_sys::{entry::clap_plugin_entry, plugin::clap_plugin_descriptor, version::CLAP_VERSION},
    entry::Entry,
    factory::Factory,
};
use std::ffi::{c_char, c_void};

static ENTRY: Entry = Entry::new(&Factory::new(c"pureaudio.TestGain", c"PureAudioGain", false, gain::process_stereo) as *const _ as *const c_void);

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
