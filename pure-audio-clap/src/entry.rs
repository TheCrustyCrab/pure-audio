use std::ffi::{c_char, c_void, CStr};
use clap_sys::factory::plugin_factory::CLAP_PLUGIN_FACTORY_ID;

pub struct Entry {
    factory_ptr: *const c_void
}

impl Entry {
    pub const fn new(factory_ptr: *const c_void) -> Self {
        Self {
            factory_ptr
        }
    }

    // init and deinit in most cases are called once, in a matched pair, when the dso is loaded / unloaded.
    // In some rare situations it may be called multiple times in a process, so the functions must be defensive,
    // mutex locking and counting calls if undertaking non trivial non idempotent actions.
    pub fn init(&self, _path: *const c_char) -> bool {
        // currently no need for a mutex, there is nothing to initialize
        true
    }

    pub fn deinit(&self) {}

    // The returned pointer must *not* be freed by the caller.
    pub unsafe fn get_factory(&self, factory_id: *const c_char) -> *const c_void {
        if CStr::from_ptr(factory_id) == CLAP_PLUGIN_FACTORY_ID {
            self.factory_ptr
        } else {
            core::ptr::null()
        }
    }
}

unsafe impl Sync for Entry {}

#[macro_export]
macro_rules! pure_audio_clap_entry {    
    ($id:expr, $name:expr, $process:expr) => {
        const _: () = {
            use pure_audio_clap::byte_strings::const_cstr;
            use pure_audio_clap::clap_sys::{entry::clap_plugin_entry, version::CLAP_VERSION};
            use pure_audio_clap::entry::Entry;
            use pure_audio_clap::factory::Factory;
            use std::ffi::{CStr, c_char, c_void};

            const ID: &CStr = const_cstr!($id);
            const NAME: &CStr = const_cstr!($name);            
            static ENTRY: Entry = Entry::new(&Factory::new(ID, NAME, $process) as *const _ as *const c_void);

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
        };
    };
}