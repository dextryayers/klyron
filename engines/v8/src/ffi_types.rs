use std::os::raw::{c_char, c_uint, c_ulong};

#[repr(C)]
pub struct V8Result {
    pub success: bool,
    pub error: [c_char; 4096],
}

#[repr(C)]
pub struct V8StringResult {
    pub success: bool,
    pub data: *mut c_char,
    pub length: c_ulong,
    pub error: [c_char; 4096],
}

#[repr(C)]
pub struct V8TypeResult {
    pub v8_type: c_uint,
    pub success: bool,
    pub error: [c_char; 4096],
}

#[repr(C)]
pub struct V8HeapStats {
    pub total_heap_size: c_ulong,
    pub total_heap_size_executable: c_ulong,
    pub total_physical_size: c_ulong,
    pub total_available_size: c_ulong,
    pub used_heap_size: c_ulong,
    pub heap_size_limit: c_ulong,
    pub malloced_memory: c_ulong,
    pub peak_malloced_memory: c_ulong,
    pub number_of_native_contexts: c_ulong,
    pub number_of_detached_contexts: c_ulong,
    pub total_global_handles_size: c_ulong,
    pub used_global_handles_size: c_ulong,
    pub external_memory: c_ulong,
}

impl V8HeapStats {
    pub fn zeroed() -> Self {
        Self {
            total_heap_size: 0,
            total_heap_size_executable: 0,
            total_physical_size: 0,
            total_available_size: 0,
            used_heap_size: 0,
            heap_size_limit: 0,
            malloced_memory: 0,
            peak_malloced_memory: 0,
            number_of_native_contexts: 0,
            number_of_detached_contexts: 0,
            total_global_handles_size: 0,
            used_global_handles_size: 0,
            external_memory: 0,
        }
    }
}

#[repr(C)]
pub struct V8Config {
    pub icu_data_path: *const c_char,
    pub snapshot_blob_path: *const c_char,
    pub max_heap_size_mb: c_uint,
    pub initial_heap_size_mb: c_uint,
    pub array_buffer_allocator_pool_size: c_uint,
    pub use_shared_memory: bool,
    pub expose_gc: bool,
    pub single_threaded: bool,
}
