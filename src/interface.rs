use crate::{Data, ObjectManager};
use std::cell::RefCell;
use std::fmt::Debug;
use std::mem::forget;
use std::ops::Deref;
use std::rc::Rc;

/// Base接口
pub trait IBase: Debug {
    fn write(&self, data: &mut Data, o: &ObjectManager);
    fn read(&self, data: &mut Data, o: &ObjectManager) -> Result<(), u32>;
    fn get_typeid(&self) -> u16;
}

/// OBJECT BASE 接口
pub trait IObjectBase: IBase + Default {
    fn get_static_typeid() -> u16;
    fn new() -> Option<Rc<dyn IBase>>;
}

pub trait IBaseAsRc {
    fn cast<T: IObjectBase>(self) -> Result<Rc<T>, Self>
    where
        Self: Sized;
}

impl IBaseAsRc for Rc<dyn IBase> {
    #[inline]
    fn cast<T: IObjectBase>(self) -> Result<Rc<T>, Self> {
        if self.get_typeid() != T::get_static_typeid() {
            Err(self)
        } else {
            let ptr = &self as *const Rc<dyn IBase> as *const Rc<T>;
            forget(self);
            unsafe { Ok(ptr.read()) }
        }
    }
}

/// 读取简化接口
pub trait GetValue {
    type ReturnType;
    fn set_none(&self);
    fn is_none(&self) -> bool;
    fn get(&self) -> Option<Self::ReturnType>;
    fn set(&self, v: Self::ReturnType);
    fn get_refmut(&mut self) -> Option<&mut Self::ReturnType>;
    unsafe fn get_unsafe_refmut(&self) -> Option<&mut Self::ReturnType>;
    unsafe fn get_unsafe_ref(&self) -> Option<&Self::ReturnType>;
}

impl<T: Clone> GetValue for RefCell<Option<T>> {
    type ReturnType = T;

    #[inline]
    fn set_none(&self) {
        *self.borrow_mut() = None;
    }

    #[inline]
    fn is_none(&self) -> bool {
        self.borrow().is_none()
    }

    #[inline]
    fn get(&self) -> Option<Self::ReturnType> {
        if let Some(ref r) = *self.borrow() {
            return Some(r.clone());
        }
        None
    }

    #[inline]
    fn set(&self, v: T) {
        self.borrow_mut().replace(v);
    }

    #[inline]
    fn get_refmut(&mut self) -> Option<&mut T> {
        if let Some(p) = self.get_mut() {
            return Some(p);
        }
        None
    }

    #[inline]
    unsafe fn get_unsafe_refmut(&self) -> Option<&mut Self::ReturnType> {
        if let Some(ref r) = *self.borrow() {
            return Some(&mut *(r.deref() as *const Self::ReturnType as *mut Self::ReturnType));
        }
        None
    }

    #[inline]
    unsafe fn get_unsafe_ref(&self) -> Option<&Self::ReturnType> {
        if let Some(ref r) = *self.borrow() {
            return Some(&*(r.deref() as *const Self::ReturnType));
        }
        None
    }
}
