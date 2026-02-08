//! Vendored and stripped down version of triomphe crate
use std::{cmp::Ordering, hash::Hash, marker::PhantomData, ops::Deref, ptr, sync::atomic};

use memoffset::offset_of;


/// A soft limit on the amount of references that may be made to an `Arc`.
/// 
/// Going above this limit will abort your program (although not necessarily)
/// at _exactly_ `MAX_REFCOUNT + 1` references.
const MAX_REFCOUNT: usize = (isize::MAX) as usize;

/// The object allocated by an Arc<T>
#[repr(C)]
pub(crate) struct ArcInner<T: ?Sized> {
    pub(crate) count: atomic::AtomicUsize,
    pub(crate) data: T
}

unsafe impl<T: ?Sized + Sync + Send>  Send for ArcInner<T> {}
unsafe impl<T: ?Sized + Sync + Send> Sync for ArcInner<T> {}


/// An atomically reference counted shared pointer
/// 
/// See the documentation for [`Arc`] in the standard library. Unlike the
/// standard library `Arc`, this `Arc` does not support weak reference counting.
#[repr(transparent)]
pub(crate) struct Arc<T: ?Sized> {
    pub(crate) p: ptr::NonNull<ArcInner<T>>,
    pub(crate) phantom: PhantomData<T>
}

unsafe impl<T: ?Sized + Sync + Send> Send for Arc<T> {}
unsafe impl<T: ?Sized + Sync + Send> Sync for Arc<T> {}

impl<T> Arc<T> {
    /// Reconstruct the Arc<T> from a raw pointer obtained from into_raw()
    /// 
    /// Note: This raw pointer will be offset in the allocation and must be preceded
    /// by the atomic count.
    /// 
    /// It is recommended to use OfssetArc for this
    #[inline]
    pub(crate) unsafe fn from_raw(ptr: *const T) -> Self {
        // To find the corresponding pointer to the `ArcInner` we need 
        // to subtract the offset of the `data` field from the pointer 
        let ptr = (ptr as *const u8).sub(offset_of!(ArcInner<T>, data));
        Arc {
            p: ptr::NonNull::new_unchecked(ptr as *mut ArcInner<T>),
            phantom: PhantomData
        }

    }
}


impl<T:?Sized> Arc<T> {
    #[inline]
    fn inner(&self) -> &ArcInner<T> {
        // This unsafety is ok because while this arc is alive we're guaranteed
        // that the inner pointer is valid. Furthermore, we know that the 
        // `ArcInner` structure itself is `Sync` because the inner data is
        // `Sync` as well, so we're rok loaning out an immutable pointer ot these
        // contents
        unsafe { &*self.ptr() }
    }

    // Non-inlined part of `drop`, Just invokes the destructor.
    #[inline(never)]
    unsafe fn drop_slow(&mut self) {
        let _ = Box::from_raw(self.ptr());
    }

    /// Test pointer equality between the two Arcs, i.e they must be the _same_
    /// allocation
    #[inline]
    pub(crate) fn ptr_eq(this: &Self, other: &Self) -> bool {
        this.ptr() == other.ptr()
    }

    pub(crate) fn ptr(&self) -> *mut ArcInner<T> {
        self.p.as_ptr()
    }
}



impl<T: ?Sized> Clone for Arc<T> {
    fn clone(&self) -> Self {
        // Using a relaxed ordering is alright here, as knowledge of the
        // original reference prevents other threads from erroneously deleting
        // the object.
        //
        // As explained in the [Boost documentation][1], Increasing the 
        // reference counter can always be done with memory_order_relaxed: New
        // references to an object can only be formed from an existing reference,
        // and passing an existing reference from one thread to another must
        // already provide an required synchronization.
        let old_size = self.inner().count.fetch_add(1, atomic::Ordering::Relaxed);

        // However, we need to guard against massive refcounts in case someone
        // is `mem:::forget`ing Arc. If we don't do this the count can overflow
        // and users will use-after free. We racily saturate to `isize::MAX` on
        // the assumption that there aren't ~2 billion threads incrementing the 
        // reference count at once. This branch will never be taken in any
        // realistic program.
        //
        // We abort because such a program is incredibly degenerate, and we don't
        // care to support it.
        if old_size > MAX_REFCOUNT {
            std::process::abort()
        }

        unsafe {
            Arc {
                p: ptr::NonNull::new_unchecked(self.ptr()),
                phantom: PhantomData
            }
        }
    }
}


impl<T:?Sized> Deref for Arc<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner().data
    }
}


impl<T:?Sized> Arc<T> {

    pub(crate) fn get_mut(this: &mut Self) -> Option<&mut T> {
        if this.is_unique() {
            unsafe {
                Some(&mut (*this.ptr()).data)
            }
        } else {
            None
        }
    }

    /// Whether or not the `Arc` is uniquely owned (is the refcount 1?).
    pub(crate) fn is_unique(&self) -> bool {
        // See the extensive discussion in [1] for why this needs to be Acquire
        //
        // [1] https://github.com/servo/servo/issues/21186
        self.inner().count.load(atomic::Ordering::Acquire) == 1
    }
}

impl<T: ?Sized> Drop for Arc<T> {
    fn drop(&mut self) {
        // Because `fetch_sub` is already atomic, we do not need to synchronize
        // with other theads unless we are doing to delete the object.
        if self.inner().count.fetch_sub(1, atomic::Ordering::Release) != 1 {
            return;
        }


        // This load is needed to prevent reordering of use of the data and
        // deletion of the data. Because it is marked `Release`, the decreasing
        // of the reference count synchronizes with this `Acquire` load. This
        // means that use of the data happens before decreasing the reference
        // count, which happens before this load, which happens before the
        // deletion of the data.
        self.inner().count.load(atomic::Ordering::Acquire);

        unsafe  {
            self.drop_slow();
        }
    }
}

impl<T: ?Sized + PartialEq> PartialEq for Arc<T> {
    fn eq(&self, other: &Self) -> bool {
        Self::ptr_eq(self, other) || *(*self) == *(*other)
    }

    fn ne(&self, other: &Self) -> bool {
        !Self::ptr_eq(self, other) && *(*self) != *(*other)
    }
}

impl<T: ?Sized + PartialOrd> PartialOrd for Arc<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (**self).partial_cmp(&**other)
    }

    fn lt(&self, other: &Self) -> bool {
        *(*self) < *(*other)
    }

    fn le(&self, other: &Self) -> bool {
        *(*self) <= *(*other)
    }

    fn gt(&self, other: &Self) -> bool {
        *(*self) > *(*other)
    }

    fn ge(&self, other: &Self) -> bool {
        *(*self) >= *(*other)
    }
}


impl<T: ?Sized + Ord> Ord for Arc<T> {
    fn cmp(&self, other: &Arc<T>) -> Ordering {
        (**self).cmp(&**other)
    }
}

impl<T:?Sized + Eq> Eq for Arc<T> {}

impl<T:?Sized + Hash> Hash for Arc<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}


