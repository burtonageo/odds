//! Slice iterators

use std::mem::size_of;
use std::marker::PhantomData;
use std::ops::Index;

/// Slice (contiguous data) iterator.
///
/// Iterator element type is `T` (by value).
/// This iterator exists mainly to have the constructor from a pair
/// of raw pointers available, which the libcore slice iterator does not allow.
///
/// Implementation note: Aliasing/optimization issues disappear if we use
/// non-pointer iterator element type, so we use `T`. (The libcore slice
/// iterator has `assume` and other tools available to combat it).
///
/// `T` must not be a zero sized type.
#[derive(Debug)]
pub struct SliceCopyIter<'a, T: 'a> {
    ptr: *const T,
    end: *const T,
    ty: PhantomData<&'a T>,
}

impl<'a, T> Copy for SliceCopyIter<'a, T> { }
impl<'a, T> Clone for SliceCopyIter<'a, T> {
    fn clone(&self) -> Self { *self }
}

impl<'a, T> SliceCopyIter<'a, T>
    where T: Copy
{
    #[inline]
    pub unsafe fn new(ptr: *const T, end: *const T) -> Self {
        assert!(size_of::<T>() != 0);
        SliceCopyIter {
            ptr: ptr,
            end: end,
            ty: PhantomData,
        }
    }

    /// Return the start, end pointer of the iterator
    pub fn into_raw(self) -> (*const T, *const T) {
        (self.ptr, self.end)
    }

    /// Return the start pointer
    pub fn start(&self) -> *const T {
        self.ptr
    }

    /// Return the end pointer
    pub fn end(&self) -> *const T {
        self.end
    }
}

impl<'a, T> Iterator for SliceCopyIter<'a, T>
    where T: Copy,
{
    type Item = T;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr != self.end {
            unsafe {
                let elt = Some(*self.ptr);
                self.ptr = self.ptr.offset(1);
                elt
            }
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (self.end as usize - self.ptr as usize) / size_of::<T>();
        (len, Some(len))
    }

    fn count(self) -> usize {
        self.len()
    }

    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }
}

impl<'a, T> DoubleEndedIterator for SliceCopyIter<'a, T>
    where T: Copy
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.ptr != self.end {
            unsafe {
                self.end = self.end.offset(-1);
                let elt = Some(*self.end);
                elt
            }
        } else {
            None
        }
    }
}

impl<'a, T> ExactSizeIterator for SliceCopyIter<'a, T> where T: Copy { }

impl<'a, T> From<&'a [T]> for SliceCopyIter<'a, T>
    where T: Copy
{
    fn from(slice: &'a [T]) -> Self {
        assert!(size_of::<T>() != 0);
        unsafe {
            let ptr = slice.as_ptr();
            let end = ptr.offset(slice.len() as isize);
            SliceCopyIter::new(ptr, end)
        }
    }
}

impl<'a, T> Default for SliceCopyIter<'a, T>
    where T: Copy
{
    /// Create an empty `SliceCopyIter`.
    fn default() -> Self {
        unsafe {
            SliceCopyIter::new(0x1 as *const T, 0x1 as *const T)
        }
    }
}

impl<'a, T> Index<usize> for SliceCopyIter<'a, T>
    where T: Copy
{
    type Output = T;
    fn index(&self, i: usize) -> &T {
        assert!(i < self.len());
        unsafe {
            &*self.ptr.offset(i as isize)
        }
    }
}

/// Slice (contiguous data) iterator.
///
/// Iterator element type is `&T`
/// This iterator exists mainly to have the constructor from a pair
/// of raw pointers available, which the libcore slice iterator does not allow.
///
/// `T` must not be a zero sized type.
#[derive(Debug)]
pub struct SliceIter<'a, T: 'a> {
    ptr: *const T,
    end: *const T,
    ty: PhantomData<&'a T>,
}

impl<'a, T> Copy for SliceIter<'a, T> { }
impl<'a, T> Clone for SliceIter<'a, T> {
    fn clone(&self) -> Self { *self }
}

impl<'a, T> SliceIter<'a, T> {
    /// Create a new slice iterator
    ///
    /// See also ``SliceIter::from, SliceIter::default``.
    #[inline]
    pub unsafe fn new(ptr: *const T, end: *const T) -> Self {
        assert!(size_of::<T>() != 0);
        SliceIter {
            ptr: ptr,
            end: end,
            ty: PhantomData,
        }
    }

    /// Return the start pointer
    pub fn start(&self) -> *const T {
        self.ptr
    }

    /// Return the end pointer
    pub fn end(&self) -> *const T {
        self.end
    }

    /// Return the next iterator element, without stepping the iterator.
    pub fn peek_next(&self) -> Option<<Self as Iterator>::Item>
    {
        if self.ptr != self.end {
            unsafe {
                Some(&*self.ptr)
            }
        } else {
            None
        }
    }
}

impl<'a, T> Iterator for SliceIter<'a, T> {
    type Item = &'a T;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr != self.end {
            unsafe {
                let elt = Some(&*self.ptr);
                self.ptr = self.ptr.offset(1);
                elt
            }
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (self.end as usize - self.ptr as usize) / size_of::<T>();
        (len, Some(len))
    }

    fn count(self) -> usize {
        self.len()
    }

    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }

    fn find<F>(&mut self, mut p: F) -> Option<Self::Item>
        where F: FnMut(&Self::Item) -> bool,
    {
        macro_rules! find_step {
            () => {{
                let elt = &*self.ptr.post_increment();
                if p(&elt) {
                    return Some(elt);
                }
            }}
        }
        unsafe {
            while ptrdistance(self.ptr, self.end) >= 4 {
                find_step!();
                find_step!();
                find_step!();
                find_step!();
            }
            while self.ptr != self.end {
                find_step!();
            }
        }
        None
    }

    fn position<F>(&mut self, mut p: F) -> Option<usize>
        where F: FnMut(Self::Item) -> bool,
    {
        let start = self.ptr;
        macro_rules! find_step {
            () => {{
                let elt = &*self.ptr.post_increment();
                if p(&elt) {
                    return Some(ptrdistance(start, elt));
                }
            }}
        }
        unsafe {
            while ptrdistance(self.ptr, self.end) >= 4 {
                find_step!();
                find_step!();
                find_step!();
                find_step!();
            }
            while self.ptr != self.end {
                find_step!();
            }
        }
        None
    }
}

#[inline(always)]
fn ptrdistance<T>(a: *const T, b: *const T) -> usize {
    (b as usize - a as usize) / size_of::<T>()
}

impl<'a, T> DoubleEndedIterator for SliceIter<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.ptr != self.end {
            unsafe {
                self.end = self.end.offset(-1);
                Some(&*self.end)
            }
        } else {
            None
        }
    }
}

impl<'a, T> ExactSizeIterator for SliceIter<'a, T> { }

impl<'a, T> From<&'a [T]> for SliceIter<'a, T> {
    fn from(slice: &'a [T]) -> Self {
        assert!(size_of::<T>() != 0);
        unsafe {
            let ptr = slice.as_ptr();
            let end = ptr.offset(slice.len() as isize);
            SliceIter::new(ptr, end)
        }
    }
}

impl<'a, T> Default for SliceIter<'a, T> {
    /// Create an empty `SliceIter`.
    fn default() -> Self {
        unsafe {
            SliceIter::new(0x1 as *const T, 0x1 as *const T)
        }
    }
}

impl<'a, T> Index<usize> for SliceIter<'a, T> {
    type Output = T;
    fn index(&self, i: usize) -> &T {
        assert!(i < self.len());
        unsafe {
            &*self.ptr.offset(i as isize)
        }
    }
}



/// Extension methods for raw pointers
pub trait PointerExt : Copy {
    unsafe fn offset(self, i: isize) -> Self;

    /// Increment by 1
    #[inline(always)]
    unsafe fn inc(&mut self) {
        *self = self.offset(1);
    }

    /// Increment the pointer by 1, but return its old value.
    unsafe fn post_increment(&mut self) -> Self {
        let current = *self;
        *self = self.offset(1);
        current
    }

    /// Decrement by 1
    #[inline(always)]
    unsafe fn dec(&mut self) {
        *self = self.offset(-1);
    }

    /// Offset by `s` multiplied by `index`.
    #[inline(always)]
    unsafe fn stride_offset(self, s: isize, index: usize) -> Self {
        self.offset(s * index as isize)
    }
}

impl<T> PointerExt for *const T {
    #[inline(always)]
    unsafe fn offset(self, i: isize) -> Self {
        self.offset(i)
    }
}

impl<T> PointerExt for *mut T {
    #[inline(always)]
    unsafe fn offset(self, i: isize) -> Self {
        self.offset(i)
    }
}