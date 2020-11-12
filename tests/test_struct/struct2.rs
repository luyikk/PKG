use pkg::interface::{IBase, IObjectBase};
use pkg::object_manager::IObjectManager;
use pkg::{Data, ObjectManager};
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
use std::collections::{HashMap, BTreeMap};

#[derive(Debug, Default, PartialEq)]
pub struct Base {
    pub id: Cell<i32>,
    pub name: RefCell<String>,
}



impl IBase for Base {
    fn write(&self, data: &mut Data, o: &ObjectManager) {
        o.write(data, &self.id);
        o.write(data, &self.name);
    }

    fn read(&self, data: &mut Data, o: &ObjectManager) -> Result<(), u32> {
        o.read(data, &self.id)?;
        o.read(data, &self.name)?;
        Ok(())
    }

    fn get_typeid(&self) -> u16 {
        Self::get_static_typeid()
    }
}

impl IObjectBase for Base {
    fn get_static_typeid() -> u16 {
        101
    }

    fn new() -> Option<Rc<dyn IBase>> {
        Some(Rc::new(Self::default()))
    }
}

#[derive(Debug, Default)]
pub struct Fly{
    pub base:RefCell<Base>,

    pub x:Cell<f32>,
    pub y:Cell<f32>,

    pub rc:RefCell<Option<Rc<Base>>>,
    pub wk:RefCell<Option<Weak<Base>>>,

    pub vec:RefCell<Vec<Option<Rc<Base>>>>,
    pub vec_wk:RefCell<Vec<Option<Weak<Base>>>>,

    pub hash:RefCell<HashMap<i32,Option<Rc<Base>>>>,
    pub hash_wk:RefCell<HashMap<i32,Option<Weak<Base>>>>,

    pub treemap:RefCell<BTreeMap<i64,Option<Rc<Base>>>>,
    pub treemap_wk:RefCell<BTreeMap<i64,Option<Weak<Base>>>>
}

impl IBase for Fly {
    fn write(&self, data: &mut Data, o: &ObjectManager) {
        o.write(data,&self.base);
        o.write(data, &self.x);
        o.write(data, &self.y);
        o.write(data, &self.rc);
        o.write(data, &self.wk);
        o.write(data,&self.vec);
        o.write(data,&self.vec_wk);
        o.write(data,&self.hash);
        o.write(data,&self.hash_wk);
        o.write(data,&self.treemap);
        o.write(data,&self.treemap_wk);
    }

    fn read(&self, data: &mut Data, o: &ObjectManager) -> Result<(), u32> {
        o.read(data,&self.base)?;
        o.read(data, &self.x)?;
        o.read(data, &self.y)?;
        o.read(data, &self.rc)?;
        o.read(data, &self.wk)?;
        o.read(data,&self.vec)?;
        o.read(data,&self.vec_wk)?;
        o.read(data,&self.hash)?;
        o.read(data,&self.hash_wk)?;
        o.read(data,&self.treemap)?;
        o.read(data,&self.treemap_wk)?;
        Ok(())
    }

    fn get_typeid(&self) -> u16 {
        Self::get_static_typeid()
    }
}

impl IObjectBase for Fly {
    fn get_static_typeid() -> u16 {
        102
    }

    fn new() -> Option<Rc<dyn IBase>> {
        Some(Rc::new(Self::default()))
    }
}