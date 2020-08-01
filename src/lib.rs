#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ops::{Deref, DerefMut};
use std::os::raw::c_void;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub mod dds_error;
pub use dds_error::DDSError;

pub type DdsEntity = dds_entity_t;
pub type DdsDomainId = dds_domainid_t;
pub type DdsTopicDescriptor = dds_topic_descriptor_t;

pub trait DDSGenType {
    /// Get the address of the static descriptor created by the generated code
    unsafe fn get_descriptor() -> &'static dds_topic_descriptor_t;

}

enum DDSAllocatedData<T: Sized + DDSGenType> {
    /// The type is allocated by Rust.
    RustAllocated(*mut T),
    /// The type is allocated by Cyclone DDS.  This is used for received data. Cyclone DDS uses its own
    /// allocator and does not allow us to provide our own.
    /// We ask Cyclone to allocate all memory for received data. Rust will allocate data for the structures
    /// we want to transmit. I assume this will work and will not break for any advanced use cases.
    CycloneDDSAllocated(*mut T),
}

pub struct DDSBox<T>(DDSAllocatedData<T>)
where
    T: Sized + DDSGenType;

impl<T> DDSBox<T>
where
    T: Sized + DDSGenType,
{
    /// Create a boxed DDS type from a buffer that is allocated by
    /// Cyclone DDS.
    pub unsafe fn new_from_cyclone_allocated_struct(p: *mut T) -> Self {
        if !p.is_null() {
            Self(DDSAllocatedData::<T>::CycloneDDSAllocated(p))
        } else {
            panic!("Tried to create DDSBox from null pointer");
        }
    }

    /// Build a DDSBox from a heap allocated DDSStructure
    pub fn new_from_box(b: Box<T>) -> Self {
        Self(DDSAllocatedData::RustAllocated(Box::into_raw(b)))
    }

    pub unsafe fn get_raw_mut_ptr(&self) -> *mut std::ffi::c_void {
        match self.0 {
            DDSAllocatedData::CycloneDDSAllocated(p) => p as *mut std::ffi::c_void,
            DDSAllocatedData::RustAllocated(p) => p as *mut std::ffi::c_void,
        }
    }
}

impl<T> Drop for DDSBox<T>
where
    T: Sized + DDSGenType,
{
    fn drop(&mut self) {
        match self.0 {
            DDSAllocatedData::CycloneDDSAllocated(p) => {
                println!("Dropping with dds_sample_free");
                unsafe {
                    dds_sample_free(
                        p as *mut std::ffi::c_void,
                        T::get_descriptor(),
                        dds_free_op_t_DDS_FREE_ALL,
                    )
                }
            }
            DDSAllocatedData::RustAllocated(p) => {
                Box::from(p); // The box will go out of scope immediately and release p
            }
        }
    }
}

impl<T> Deref for DDSBox<T>
where
    T: Sized + DDSGenType,
{
    type Target = T;
    fn deref(&self) -> &T {
        match self.0 {
            DDSAllocatedData::CycloneDDSAllocated(p) => unsafe { &*p as &T },
            DDSAllocatedData::RustAllocated(p) => unsafe { &*p as &T },
        }
    }
}

impl<T> DerefMut for DDSBox<T>
where
    T: Sized + DDSGenType,
{
    fn deref_mut(&mut self) -> &mut T {
        match self.0 {
            DDSAllocatedData::CycloneDDSAllocated(p) => unsafe { &mut *p },
            DDSAllocatedData::RustAllocated(p) => unsafe { &mut *p },
        }
    }
}

pub fn write<T>(entity: dds_entity_t, msg: &T) -> Result<(), DDSError>
where
    T: Sized + DDSGenType,
{
    unsafe {
        let ret = dds_write(entity, msg as *const T as *const std::ffi::c_void);
        if ret >= 0 {
            Ok(())
        } else {
            Err(DDSError::from(ret))
        }
    }
}

pub fn read<T>(entity: dds_entity_t) -> Result<DDSBox<T>, DDSError>
where
    T: Sized + DDSGenType,
{
    unsafe {
        let mut info: dds_sample_info = dds_sample_info::default();
        let mut voidp: *mut c_void = std::ptr::null::<T>() as *mut c_void;
        let voidpp: *mut *mut c_void = &mut voidp;

        let ret = dds_read(entity, voidpp, &mut info as *mut _, 1, 1);

        if ret >= 0 {
            if !voidp.is_null() && info.valid_data {
                let buf = DDSBox::<T>::new_from_cyclone_allocated_struct(voidp as *mut T);
                Ok(buf)
            } else {
                Err(DDSError::OutOfResources)
            }
        } else {
            Err(DDSError::from(ret))
        }
    }
}
