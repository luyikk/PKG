use pkg::interface::{IBase, IObjectBase};
use pkg::object_manager::IObjectManager;
use pkg::{Data, ObjectManager};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Path {
    pub x: Cell<i32>,
    pub y: Cell<i32>,
    pub name: RefCell<Option<String>>,
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct PathBase {
    pub index: Cell<i32>,
    pub path: RefCell<Option<Rc<Path>>>,
}

impl IBase for Path {
    fn write(&self, data: &mut Data, obj_manager: &ObjectManager) {
        obj_manager.write(data, &self.x);
        obj_manager.write(data, &self.y);
        obj_manager.write(data, &self.name);
    }

    fn read(
        &self,
        data: &mut Data,
        obj_manager: &ObjectManager,
    ) -> std::result::Result<(), u32> {
        obj_manager.read(data, &self.x)?;
        obj_manager.read(data, &self.y)?;
        obj_manager.read(data, &self.name)?;
        Ok(())
    }

    fn get_typeid(&self) -> u16 {
        Path::get_static_typeid()
    }
}

impl IObjectBase for Path {
    fn get_static_typeid() -> u16 {
        1
    }
    fn new() -> Option<Rc<dyn IBase>> {
        Some(Rc::new(Path {
            x: Cell::new(1000),
            y: Cell::new(10000),
            name: RefCell::new(Some("abcdefg".to_string())),
        }))
    }
}

impl IBase for PathBase {
    fn write(&self, data: &mut Data, obj_manager: &ObjectManager) {
        obj_manager.write(data, &self.index);
        obj_manager.write(data, &self.path);
    }

    fn read(
        &self,
        data: &mut Data,
        obj_manager: &ObjectManager,
    ) -> std::result::Result<(), u32> {
        obj_manager.read(data, &self.index)?;
        obj_manager.read(data, &self.path)?;
        Ok(())
    }

    fn get_typeid(&self) -> u16 {
        PathBase::get_static_typeid()
    }
}

impl IObjectBase for PathBase {
    fn get_static_typeid() -> u16 {
        2
    }

    fn new() -> Option<Rc<dyn IBase>> {
        Some(Rc::new(PathBase {
            index: Cell::new(1),
            path: RefCell::new(None),
        }))
    }
}
