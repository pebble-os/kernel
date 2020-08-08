//! This module contains functions that help us validate the inputs to system calls that try to
//! make sure userspace can't crash or exploit the kernel in any way. For example, if we take an
//! address from userspace, we should make sure it's mapped (so we don't page-fault) and an address
//! that userspace could ordinarily access itself (otherwise, we could leak information to a
//! userspace task that it shouldn't be able to access).

use core::{marker::PhantomData, ptr, slice, str};

pub struct UserPointer<T> {
    ptr: *mut T,
    can_write: bool,
}

impl<T> UserPointer<T> {
    pub fn new(ptr: *mut T, needs_write: bool) -> UserPointer<T> {
        // TODO: validate that this is a valid pointer:
        //  - the address is canonical
        //  - the address is in user-space
        //  - the address is actually mapped
        //  - if we're writing, that the mapping is writable
        UserPointer { ptr, can_write: needs_write }
    }

    pub fn read(&self) -> Result<T, ()> {
        Ok(unsafe { ptr::read_volatile(self.ptr) })
    }

    pub fn write(&mut self, value: T) -> Result<(), ()> {
        if !self.can_write {
            return Err(());
        }

        /*
         * This has two subtleties:
         *    - Using `write_volatile` instead of `write` makes sure the compiler doesn't think it can elide the
         *      write, as the data is read and written to from both the kernel and userspace.
         *    - Using `ptr::write_volatile(x, ...)` instead of `*x = ...` makes sure we don't attempt to drop
         *      the existing value, which could read uninitialized memory.
         */
        unsafe { ptr::write_volatile(self.ptr, value) }
        Ok(())
    }
}

/// Represents a slice of `T`s in userspace.
pub struct UserSlice<'a, T> {
    length: usize,
    ptr: *mut T,
    _phantom: PhantomData<&'a ()>,
}

impl<'a, T> UserSlice<'a, T> {
    pub fn new(length: usize, ptr: *mut T) -> UserSlice<'a, T> {
        UserSlice { length, ptr, _phantom: PhantomData }
    }

    pub fn validate_read(&self) -> Result<&'a [T], ()> {
        // TODO: validate access is valid
        Ok(unsafe { slice::from_raw_parts(self.ptr, self.length) })
    }
}

pub struct UserString<'a>(UserSlice<'a, u8>);

impl<'a> UserString<'a> {
    pub fn new(length: usize, ptr: *mut u8) -> UserString<'a> {
        UserString(UserSlice::new(length, ptr))
    }

    pub fn validate(&self) -> Result<&'a str, ()> {
        str::from_utf8(self.0.validate_read()?).map_err(|_| ())
    }
}