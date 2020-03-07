#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

/// All message types will implement this trait
/// to call the dds_alloc and free funtions.
pub trait DdsAllocator {
    fn alloc() -> Box<Self>;
    fn free(b : Box<Self>);
}

/// Create a topic given the topic type and the topic name.
/// Example:
///     let topic = create_topic!(participant,HelloWorldData_Msg,"HelloWorldData_Msg", None, None).unwrap();
#[macro_export]
macro_rules! impl_allocator_for_dds_type {
    ($ddstype:ident) => {

        impl DdsAllocator for $ddstype {
            fn alloc() -> Box<Self> {
                unsafe {
                    let t : *mut $ddstype = dds_alloc(std::mem::size_of::<$ddstype> as usize) as *mut $ddstype;
                    if t.is_null() {
                        panic!("dds_alloc failed for $ddstype");
                    }
                    Box::from_raw(t)
                }
            }

            fn free(b: Box<Self>) {
                unsafe {
                dds_sample_free(
                    Box::into_raw(b) as *mut $ddstype as *mut std::ffi::c_void,
                    paste::expr!{ &[<$ddstype _desc>] as &'static dds_topic_descriptor_t},
                    DDS_FREE_ALL,
                );
                }
            }
        }
    }
}

