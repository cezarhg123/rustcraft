
/// wrapper for pointers, doesnt actually hold the data.
/// 
/// very unsafe but fuck rust i know what im doing
#[derive(Debug, Clone)]
pub struct PtrWrapper<T> {
    ptr: *const T
}

impl<T> PtrWrapper<T> {
    pub fn new(ptr: *const T) -> PtrWrapper<T> {
        PtrWrapper { ptr }
    }

    pub fn as_ref(&self) -> &T {
        unsafe { &*self.ptr }
    }

    pub fn as_mut(&self) -> &mut T {
        unsafe { &mut *(self.ptr.cast_mut()) }
    }
}

unsafe impl<T> Send for PtrWrapper<T> {}
