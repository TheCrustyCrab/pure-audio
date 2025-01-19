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
    pub fn init(&self, path: *const c_char) -> bool {
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