// Copyright 2014-2016 bluss and ndarray developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! The data (inner representation) traits for ndarray

use std::mem::{self, size_of};
use std::rc::Rc;

use {
    ArrayBase,
    Dimension,
    ViewRepr,
};

/// Array representation trait.
///
/// ***Note:*** `Data` is not an extension interface at this point.
/// Traits in Rust can serve many different roles. This trait is public because
/// it is used as a bound on public methods.
pub unsafe trait Data : Sized {
    /// The array element type.
    type Elem;
    #[doc(hidden)]
    // This method is only used for debugging
    fn _data_slice(&self) -> &[Self::Elem];
}

/// Array representation trait.
///
/// For an array with writable elements.
/// 
/// ***Internal trait, see `Data`.***
pub unsafe trait DataMut : Data {
    #[doc(hidden)]
    #[inline]
    fn ensure_unique<D>(&mut ArrayBase<Self, D>)
        where Self: Sized,
              D: Dimension
    { }

    #[doc(hidden)]
    #[inline]
    fn is_unique(&mut self) -> bool {
        true
    }
}

/// Array representation trait.
///
/// An array representation that can be cloned.
///
/// ***Internal trait, see `Data`.***
pub unsafe trait DataClone : Data {
    #[doc(hidden)]
    /// Unsafe because, `ptr` must point inside the current storage.
    unsafe fn clone_with_ptr(&self, ptr: *mut Self::Elem) -> (Self, *mut Self::Elem);

    #[doc(hidden)]
    unsafe fn clone_from_with_ptr(&mut self, other: &Self, ptr: *mut Self::Elem) -> *mut Self::Elem {
        let (data, ptr) = other.clone_with_ptr(ptr);
        *self = data;
        ptr
    }
}

unsafe impl<A> Data for Rc<Vec<A>> {
    type Elem = A;
    fn _data_slice(&self) -> &[A] {
        self
    }
}

// NOTE: Copy on write
unsafe impl<A> DataMut for Rc<Vec<A>>
    where A: Clone
{
    fn ensure_unique<D>(self_: &mut ArrayBase<Self, D>)
        where Self: Sized,
              D: Dimension
    {
        if Rc::get_mut(&mut self_.data).is_some() {
            return;
        }
        if self_.dim.size() <= self_.data.len() / 2 {
            // Create a new vec if the current view is less than half of
            // backing data.
            unsafe {
                *self_ = ArrayBase::from_shape_vec_unchecked(self_.dim.clone(),
                                                             self_.iter()
                                                            .cloned()
                                                            .collect());
            }
            return;
        }
        let a_size = mem::size_of::<A>() as isize;
        let our_off = if a_size != 0 {
            (self_.ptr as isize - self_.data.as_ptr() as isize) / a_size
        } else { 0 };
        let rvec = Rc::make_mut(&mut self_.data);
        unsafe {
            self_.ptr = rvec.as_mut_ptr().offset(our_off);
        }
    }

    fn is_unique(&mut self) -> bool {
        Rc::get_mut(self).is_some()
    }
}

unsafe impl<A> DataClone for Rc<Vec<A>> {
    unsafe fn clone_with_ptr(&self, ptr: *mut Self::Elem) -> (Self, *mut Self::Elem) {
        // pointer is preserved
        (self.clone(), ptr)
    }
}

unsafe impl<A> Data for Vec<A> {
    type Elem = A;
    fn _data_slice(&self) -> &[A] {
        self
    }
}

unsafe impl<A> DataMut for Vec<A> { }

unsafe impl<A> DataClone for Vec<A>
    where A: Clone
{
    unsafe fn clone_with_ptr(&self, ptr: *mut Self::Elem) -> (Self, *mut Self::Elem) {
        let mut u = self.clone();
        let mut new_ptr = u.as_mut_ptr();
        if size_of::<A>() != 0 {
            let our_off = (ptr as isize - self.as_ptr() as isize) /
                          mem::size_of::<A>() as isize;
            new_ptr = new_ptr.offset(our_off);
        }
        (u, new_ptr)
    }

    unsafe fn clone_from_with_ptr(&mut self, other: &Self, ptr: *mut Self::Elem) -> *mut Self::Elem {
        let our_off = if size_of::<A>() != 0 {
            (ptr as isize - other.as_ptr() as isize) /
                          mem::size_of::<A>() as isize
        } else {
            0
        };
        self.clone_from(other);
        self.as_mut_ptr().offset(our_off)
    }
}

unsafe impl<'a, A> Data for ViewRepr<&'a A> {
    type Elem = A;
    fn _data_slice(&self) -> &[A] {
        &[]
    }
}

unsafe impl<'a, A> DataClone for ViewRepr<&'a A> {
    unsafe fn clone_with_ptr(&self, ptr: *mut Self::Elem) -> (Self, *mut Self::Elem) {
        (*self, ptr)
    }
}

unsafe impl<'a, A> Data for ViewRepr<&'a mut A> {
    type Elem = A;
    fn _data_slice(&self) -> &[A] {
        &[]
    }
}

unsafe impl<'a, A> DataMut for ViewRepr<&'a mut A> { }

/// Array representation trait.
///
/// A representation that is a unique or shared owner of its data.
///
/// ***Internal trait, see `Data`.***
pub unsafe trait DataOwned : Data {
    #[doc(hidden)]
    fn new(elements: Vec<Self::Elem>) -> Self;
    #[doc(hidden)]
    fn into_shared(self) -> Rc<Vec<Self::Elem>>;
}

/// Array representation trait.
///
/// A representation that is a lightweight view.
///
/// ***Internal trait, see `Data`.***
pub unsafe trait DataShared : Clone + DataClone { }

unsafe impl<A> DataShared for Rc<Vec<A>> {}
unsafe impl<'a, A> DataShared for ViewRepr<&'a A> {}

unsafe impl<A> DataOwned for Vec<A> {
    fn new(elements: Vec<A>) -> Self {
        elements
    }
    fn into_shared(self) -> Rc<Vec<A>> {
        Rc::new(self)
    }
}

unsafe impl<A> DataOwned for Rc<Vec<A>> {
    fn new(elements: Vec<A>) -> Self {
        Rc::new(elements)
    }
    fn into_shared(self) -> Rc<Vec<A>> {
        self
    }
}

