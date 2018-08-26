use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::Arc;
use std::ptr;

/// A reference-counted, nullable, atomic pointer.
pub struct AtomicArc<T> {
    ptr: AtomicPtr<T>,
}

impl<T> AtomicArc<T> {
    /// Create a new `AtomicArc`.
    pub fn new(data: Option<T>) -> AtomicArc<T> {
        let arc = data.map(|val| Arc::new(val));
        AtomicArc::from_arc(arc)
    }

    /// Create a new `AtomicArc` from an `Arc`.
    pub fn from_arc(arc: Option<Arc<T>>) -> AtomicArc<T> {
        let ptr = into_raw(arc) as *mut _;
        let ptr = AtomicPtr::new(ptr);
        AtomicArc {
            ptr,
        }
    }

    /// Get the value of the pointer as an `Arc`.
    /// This can be done non-atomically since we have a unique reference to the `AtomicArc`.
    pub fn get_arc(&mut self) -> Option<Arc<T>> {
        let ptr = *self.ptr.get_mut();
        unsafe {
            let arc = from_raw(ptr);
            let ret = arc.clone();
            let _ = into_raw(arc);
            ret
        }
    }

    /// Get a reference to the value stored in this `AtomicArc`.
    /// This can be done non-atomically since we have a unique reference to the `AtomicArc`.
    pub fn get(&mut self) -> Option<&T> {
        unsafe {
            self.ptr.get_mut().as_ref()
        }
    }

    /// Convert this `AtomicArc` into a plain old `Arc`
    pub fn into_arc(mut self) -> Option<Arc<T>> {
        let ptr = *self.ptr.get_mut();
        unsafe {
            from_raw(ptr)
        }
    }

    /// Load the value stored in this `AtomicArc`
    pub fn load(&self, order: Ordering) -> Option<Arc<T>> {
        let ptr = self.ptr.load(order);
        unsafe {
            let arc = from_raw(ptr);
            let ret = arc.clone();
            let _ = into_raw(arc);
            ret
        }
    }

    /// Store a new value.
    pub fn store(&self, new: Option<Arc<T>>, order: Ordering) {
        let _drop = self.swap(new, order);
    }

    /// Atomically swap the value stored in this `AtomicArc` with the new value, returning the old
    /// value.
    pub fn swap(&self, new: Option<Arc<T>>, order: Ordering) -> Option<Arc<T>> {
        let new_ptr = into_raw(new) as *mut _;
        let old_ptr = self.ptr.swap(new_ptr, order);
        unsafe {
            from_raw(old_ptr)
        }
    }

    /// Atomically swaps the value stored in this `AtomicArc` if `old` points to the same `Arc` as
    /// what is currently stored. This does not compare the underlying data, merely that the
    /// pointers match. Returns the previous value stored in this `AtomicArc`, which will be the
    /// same as `old` if the swap was successful.
    pub fn compare_and_swap(
        &self,
        old: Option<Arc<T>>,
        new: Option<Arc<T>>,
        order: Ordering,
    ) -> Option<Arc<T>> {
        let old_ptr = into_raw(old) as *mut _;
        let new_ptr = into_raw(new) as *mut _;
        let prev_ptr = self.ptr.compare_and_swap(old_ptr, new_ptr, order);
        if old_ptr == prev_ptr {
            unsafe {
                let _drop = from_raw(old_ptr);
                from_raw(old_ptr)
            }
        } else {
            unsafe {
                from_raw(new_ptr)
            }
        }
    }
}

fn into_raw<T>(a: Option<Arc<T>>) -> *const T {
    a.map(|a| Arc::into_raw(a)).unwrap_or(ptr::null())
}

unsafe fn from_raw<T>(ptr: *const T) -> Option<Arc<T>> {
    if ptr.is_null() {
        None
    } else {
        Some(Arc::from_raw(ptr))
    }
}

