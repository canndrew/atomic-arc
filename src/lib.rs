use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::Arc;
use std::ptr;

pub struct AtomicArc<T> {
    ptr: AtomicPtr<T>,
}

impl<T> AtomicArc<T> {
    pub fn new(data: Option<T>) -> AtomicArc<T> {
        let arc = data.map(|val| Arc::new(val));
        let ptr = into_raw(arc) as *mut _;
        let ptr = AtomicPtr::new(ptr);
        AtomicArc {
            ptr,
        }
    }

    pub fn get_mut(&mut self) -> Option<Arc<T>> {
        let ptr = *self.ptr.get_mut();
        unsafe {
            let arc = from_raw(ptr);
            let ret = arc.clone();
            let _ = into_raw(arc);
            ret
        }
    }

    pub fn into_arc(mut self) -> Option<Arc<T>> {
        let ptr = *self.ptr.get_mut();
        unsafe {
            from_raw(ptr)
        }
    }

    pub fn load(&self, order: Ordering) -> Option<Arc<T>> {
        let ptr = self.ptr.load(order);
        unsafe {
            let arc = from_raw(ptr);
            let ret = arc.clone();
            let _ = into_raw(arc);
            ret
        }
    }

    pub fn store(&self, new: Option<Arc<T>>, order: Ordering) {
        let _drop = self.swap(new, order);
    }

    pub fn swap(&self, new: Option<Arc<T>>, order: Ordering) -> Option<Arc<T>> {
        let new_ptr = into_raw(new) as *mut _;
        let old_ptr = self.ptr.swap(new_ptr, order);
        unsafe {
            from_raw(old_ptr)
        }
    }

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

