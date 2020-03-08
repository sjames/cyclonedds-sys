#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub trait DDSGenType{
    /// Get the address of the static descriptor created by the generated code
    unsafe fn get_descriptor() -> &'static dds_topic_descriptor_t;
    /// get the raw pointer to the structure
    unsafe fn get_raw_ptr(&self) -> *mut std::ffi::c_void;
}

