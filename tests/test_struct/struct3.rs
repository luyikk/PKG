use pkg::pkgbuilder::*;
use pkg::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

#[derive(build, Debug)]
#[cmd(typeid(1000))]
pub struct Foo {
    #[cmd(default(5))]
    pub x: Cell<i32>,
    #[cmd(default(0.5))]
    pub y: Cell<f32>,
    #[cmd(default("sb"))]
    pub name: RefCell<String>,
}

#[derive(build, Debug)]
#[cmd(typeid(1001))]
pub struct Foo2 {
    pub base: RefCell<Foo>,
    pub name: RefCell<Option<String>>,
    pub ptr: RefCell<Option<Rc<Foo>>>,
}


