#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ops::{Deref, DerefMut};
use std::os::raw::c_void;
use bitmask::bitmask;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub mod dds_error;
pub use dds_error::DDSError;

//pub type DdsEntity = dds_entity_t;
pub struct DdsEntity(dds_entity_t);

impl DdsEntity {
    pub unsafe fn new(entity: dds_entity_t) -> Self {
        DdsEntity(entity)
    }
    pub unsafe fn entity(&self) -> dds_entity_t {
        self.0
    }
}

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

pub fn write<T>(entity: DdsEntity, msg: &T) -> Result<(), DDSError>
where
    T: Sized + DDSGenType,
{
    unsafe {
        let ret = dds_write(entity.entity(), msg as *const T as *const std::ffi::c_void);
        if ret >= 0 {
            Ok(())
        } else {
            Err(DDSError::from(ret))
        }
    }
}

pub unsafe fn read<'a, T>(entity: &DdsEntity) -> Result<DdsLoanedData<T>, DDSError> 
where
    T: Sized + DDSGenType,
{
    let mut info: dds_sample_info = dds_sample_info::default();
    let mut voidp: *mut c_void = std::ptr::null::<T>() as *mut c_void;
    let voidpp: *mut *mut c_void = &mut voidp;

    let ret = dds_read_wl(entity.entity(), voidpp, &mut info as *mut _,1);

    if ret >= 0 {
        if !voidp.is_null() && info.valid_data {
            let ptr_to_ts : *const *const T = voidpp as *const *const T;
            let data = DdsLoanedData::new(ptr_to_ts,entity,ret as usize);
            Ok(data)
        } else {
            Err(DDSError::OutOfResources)
        }
    } else {
        Err(DDSError::from(ret))
    }
}

pub unsafe fn take<'a, T>(entity: &DdsEntity) -> Result<DdsLoanedData<T>, DDSError> 
where
    T: Sized + DDSGenType,
{
    let mut info = dds_sample_info::default();
    let mut voidp: *mut c_void = std::ptr::null::<T>() as *mut c_void;
    let voidpp: *mut *mut c_void = &mut voidp;

    let ret = dds_take_wl(entity.entity(), voidpp, &mut info as *mut _,1);

    if ret >= 0 {
        if !voidp.is_null() && info.valid_data {
            let ptr_to_ts : *const *const T = voidpp as *const *const T;
            let data = DdsLoanedData::new(ptr_to_ts,entity,ret as usize);
            Ok(data)
        } else {
            Err(DDSError::OutOfResources)
        }
    } else {
        Err(DDSError::from(ret))
    }
}

pub struct DdsLoanedData<T: Sized + DDSGenType>(*const *const T, dds_entity_t, usize);

impl<T> DdsLoanedData<T>
where
    T: Sized + DDSGenType,
{
    pub unsafe fn new(p: *const *const T, entity: &DdsEntity, size: usize) -> Self {
        //let ptr_to_ts = *p as *const T;
        if  !p.is_null() && !(*p as *const T).is_null() {
            Self(p, entity.entity(), size)
        } else {
            panic!("Bad pointer when creating DdsLoanedData");
        }
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe {
            let ptr_to_ts = *self.0 as *const T;
            std::slice::from_raw_parts(ptr_to_ts, self.2)
        }
    }
}

impl<T> Drop for DdsLoanedData<T>
where
    T: Sized + DDSGenType,
{
    fn drop(&mut self) {
        //unsafe {
            //let ret = dds_return_loan(self.1, self.0 as *mut *mut std::ffi::c_void, self.2 as i32);
            //if ret < 0 {
            //    panic!("Panic as drop cannot fail: {}", DDSError::from(ret));
            //}
        //}
    }
}

bitmask! {
    pub mask StateMask : u32 where flags Flags {
        DdsReadSampleState = 0x1,
        DdsNotReadSampleState = 0x2,
        DdsAnySampleState = 0x1 | 0x2,
        DdsNewViewState = 0x4,
        DdsNotNewViewState = 0x8,
        DdsAnyViewState = 0x4 | 0x8,
        DdsAliveInstanceState = 16,
        DdsNotAliveDisposedInstanceState = 32,
        DdsNotAliveNoWritersInstanceState = 64,
        DdsAnyInstanceState = 16 | 32 | 64,
        DdsAnyState =  1 | 2  | 4 | 8 | 16 | 32 | 64,
    }
}
