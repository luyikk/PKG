use crate::{Data, ObjectManager};
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::fmt::Debug;
use std::mem::forget;

/// Base接口
pub trait IBase:Debug {
    fn write(&self, data: &mut Data, o: &ObjectManager);
    fn read(&self, data: &mut Data, o: &ObjectManager) -> Result<(), u32>;
    fn get_typeid(&self) -> u16;
}

/// OBJECT BASE 接口
pub trait IObjectBase: IBase + Default {
    fn get_static_typeid() -> u16;
    fn new() -> Option<Rc<dyn IBase>>;
}



pub trait IBaseAsRc{
    fn cast<T:IObjectBase>(self)->Result<Rc<T>,Self> where Self: Sized;
}

impl IBaseAsRc for Rc<dyn IBase>{
    fn cast<T:IObjectBase>(self) -> Result<Rc<T>, Self> {
        if self.get_typeid()!=T::get_static_typeid()  {
            Err(self)
        }
        else{
            let ptr=&self as *const Rc<dyn IBase> as *const Rc<T>;
            forget(self);
            unsafe {
                Ok(ptr.read())
            }
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

    fn set_none(&self) {
        *self.borrow_mut() = None;
    }

    fn is_none(&self) -> bool {
        self.borrow().is_none()
    }

    fn get(&self) -> Option<Self::ReturnType> {
        if let Some(ref r) = *self.borrow() {
            return Some(r.clone());
        }
        None
    }

    fn set(&self, v: T) {
        self.borrow_mut().replace(v);
    }

    fn get_refmut(&mut self) -> Option<&mut T> {
        if let Some(p) = self.get_mut() {
            return Some(p);
        }
        None
    }

    unsafe fn get_unsafe_refmut(&self) -> Option<&mut Self::ReturnType> {
        if let Some(ref r) = *self.borrow() {
            return Some(&mut *(r.deref() as *const Self::ReturnType as *mut Self::ReturnType));
        }
        None
    }

    unsafe fn get_unsafe_ref(&self) -> Option<&Self::ReturnType> {
        if let Some(ref r) = *self.borrow() {
            return Some(&*(r.deref() as *const Self::ReturnType));
        }
        None
    }
}
