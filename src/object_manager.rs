use crate::interface::{IBase, IObjectBase};
use crate::{Data, IBaseAsRc};
use bytes::{Buf, BufMut};
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
use std::ops::Deref;
use std::rc::{Rc, Weak};


/// object 读取写入接口
pub trait IObjectManager {
    fn write_to<T: WriteObject>(&self, data: &mut Data, arg: &T);
    fn write<T: WriteObject>(&self, data: &mut Data, arg: &T);
    fn write_core<T:IBase>(&self,data:&mut Data,arg:&Rc<T>);
    fn read_from<T: ReadObject>(&self, data: &mut Data, arg: &mut T) -> Result<(), u32>;
    fn read<T: ReadOnlyObject>(&self, data: &mut Data, arg: &T) -> Result<(), u32>;
    fn read_core(&self,data: &mut Data)->Result<Rc<dyn IBase>,u32>;
}

pub struct ObjectManager {
    pub fs: Vec<Box<dyn Fn() -> Option<Rc<dyn IBase>>>>,
}

impl IObjectManager for ObjectManager {
    /// 写入
    fn write_to<T: WriteObject>(&self, data: &mut Data, arg: &T) {
        data.w_ptr_dict.clear();
        arg.write_(data, self);
    }

    /// 写入结构字段用
    fn write<T: WriteObject>(&self, data: &mut Data, arg: &T) {
        arg.write_(data, self);
    }

    ///写入RC<IBASE>
    fn write_core<T:IBase>(&self, data: &mut Data, arg: &Rc<T>) {
        //arg.write_(data,self);
        self.write_ptr(data,arg);
    }

    /// 读取一个预设值
    fn read_from<T: ReadObject>(&self, data: &mut Data, arg: &mut T) -> Result<(), u32> {
        arg.read_(data, self)?;
        data.r_ptr_dict.clear();
        Ok(())
    }

    /// 用于结构读取
    fn read<T: ReadOnlyObject>(&self, data: &mut Data, arg: &T) -> Result<(), u32> {
        arg.read_(data, self)
    }

    /// 根据DATA 数据读取一个RC<ANY> 返回type_id
    fn read_core(&self, data: &mut Data) -> Result<Rc<dyn IBase>, u32> {
        let mut value:Option<Rc<dyn IBase>>=None;
        value.read_(data,self)?;

        match value {
            None=>{
                Err(line!())
            },
            Some(p)=>{
                Ok(p)
            }
        }

    }
}

impl ObjectManager {
    pub fn new() -> ObjectManager {
        let mut fs: Vec<Box<dyn Fn() -> Option<Rc<dyn IBase>>>> = Vec::with_capacity(65536);
        for _ in 0..65536 {
            fs.push(Box::new(|| None))
        }

        ObjectManager { fs }
    }

    /// 注册PKG
    pub fn register<T: IObjectBase + 'static>(&mut self) {
        let typeid = T::get_static_typeid() as usize;
        self.fs[typeid] = Box::new(T::new);
    }

    /// 根据TYPEID 返回 对象
    pub fn create(&self, typeid: u16) -> Option<Rc<dyn IBase>> {
        let typeid = typeid as usize;
        if let Some(p) = (self.fs[typeid])() {
            return Some(p);
        }
        None
    }


    /// 写入RC
    fn write_ptr<T: IBase>(&self, data: &mut Data, arg: &Rc<T>) {
        let typeid = arg.get_typeid();
        data.write_bit7(typeid);
        if typeid == 0 {
            return;
        }

        let len = data.w_ptr_dict.len();

        let offset = {
            let addr = arg.deref() as *const T as usize;
            *data.w_ptr_dict.entry(addr).or_insert(len as u32 + 1)
        };

        data.write_bit7(offset);
        if data.w_ptr_dict.len() != len {
            (*arg).deref().write_(data, self);
        }
    }

    /// 写入一个 RC<IBASE> 对象
    pub(crate) fn write_rc<T: IBase>(&self, data: &mut Data, arg: &Option<Rc<T>>) {
        if let Some(arg) = arg {
            self.write_ptr(data, arg)
        } else {
            data.write_to(0u8);
        }
    }

    /// 写入一个weak
    pub(crate) fn write_weak<T: IBase>(&self, data: &mut Data, arg: &Option<Weak<T>>) {
        if let Some(arg) = arg {
            if let Some(x) = arg.upgrade() {
                self.write_rc(data, &Some(x));
                return;
            }
        }
        data.write_to(0u8);
    }

    /// 写入一个可空类型
    pub(crate) fn write_option<T: WriteObject>(&self, data: &mut Data, arg: &Option<T>) {
        match arg {
            Some(x) => {
                data.write_to(1u8);
                x.write_(data, self);
            }
            None => {
                data.write_to(0u8);
            }
        }
    }

    /// 写入一个obj
    pub(crate) fn write_obj<T: IBase>(&self, data: &mut Data, arg: &T) {
        arg.write(data, self)
    }

    /// 写入一个VEC
    pub(crate) fn write_vec<T: WriteObject>(&self, data: &mut Data, args: &Vec<T>) {
        data.write_bit7(args.len() as u64);
        if args.is_empty() {
            return;
        }

        for arg in args {
            arg.write_(data, self);
        }
    }

    /// 写入一个VEC
    pub(crate) fn write_vec_rc<T: IBase>(&self, data: &mut Data, args: &Vec<Rc<T>>) {
        data.write_bit7(args.len() as u64);
        if args.is_empty() {
            return;
        }

        for arg in args {
            self.write_ptr(data, arg);
        }
    }

    /// 写入一个字符串
    pub(crate) fn write_string(&self, data: &mut Data, arg: &str) {
        data.write_str_bit7(arg)
    }

    /// 写入一个 treemap
    pub(crate) fn write_treemap<K: WriteObject, V: WriteObject>(
        &self,
        data: &mut Data,
        args: &BTreeMap<K, V>,
    ) {
        data.write_bit7(args.len() as u64);
        for (k, v) in args.iter() {
            k.write_(data, self);
            v.write_(data, self);
        }
    }

    /// 写入一个 treemap value 是RC<IBASE>
    pub(crate) fn write_treemap_rc_value<K: WriteObject, V: IBase>(
        &self,
        data: &mut Data,
        args: &BTreeMap<K, Rc<V>>,
    ) {
        data.write_bit7(args.len() as u64);
        for (k, v) in args.iter() {
            k.write_(data, self);
            self.write_ptr(data, v);
        }
    }

    /// 写入一个hashmap
    pub(crate) fn write_hashmap<K: WriteObject, V: WriteObject>(
        &self,
        data: &mut Data,
        args: &HashMap<K, V>,
    ) {
        data.write_bit7(args.len() as u64);
        for (k, v) in args.iter() {
            k.write_(data, self);
            v.write_(data, self);
        }
    }

    /// 写入一个hashmap value 是RC<IBASE>
    pub(crate) fn write_hashmap_rc_value<K: WriteObject, V: IBase>(
        &self,
        data: &mut Data,
        args: &HashMap<K, Rc<V>>,
    ) {
        data.write_bit7(args.len() as u64);
        for (k, v) in args.iter() {
            k.write_(data, self);
            self.write_ptr(data, v);
        }
    }

    /// 写入一个hashmap key 是RC<IBASE>
    pub(crate) fn write_hashmap_rc_key<K: IBase, V: WriteObject>(
        &self,
        data: &mut Data,
        args: &HashMap<Rc<K>, V>,
    ) {
        data.write_bit7(args.len() as u64);
        for (k, v) in args.iter() {
            self.write_ptr(data, k);
            v.write_(data, self);
        }
    }

    /// 写入一个hashmap RC<IBASE>
    pub(crate) fn write_hashmap_rc<K: IBase, V: IBase>(
        &self,
        data: &mut Data,
        args: &HashMap<Rc<K>, Rc<V>>,
    ) {
        data.write_bit7(args.len() as u64);
        for (k, v) in args.iter() {
            self.write_ptr(data, k);
            self.write_ptr(data, v);
        }
    }

    /// 读取RC IBASE
    pub(crate) fn read_rc_ibase(&self,data:&mut Data,v:&mut Option<Rc<dyn IBase>>)-> Result<(), u32>{
        let type_id = {
            match data.read_bit7_u16() {
                None => {
                    return Err(line!());
                }
                Some((_, type_id)) => {
                    if type_id == 0 {
                        *v = None;
                        return Ok(());
                    }
                    type_id
                }
            }
        };

        let offs = {
            match data.read_bit7_u32() {
                None => {
                    return Err(line!());
                }
                Some((_, offs)) => offs,
            }
        };

        let len = data.r_ptr_dict.len() as u32;
        if offs == len + 1 {
            if let Some(ref v) = v {
                data.r_ptr_dict.insert(offs, v.clone());
                v.read(data, self)?;
                Ok(())
            } else {
                let vv = self.create(type_id);
                if let Some(vv) = vv {
                    data.r_ptr_dict.insert(offs, vv.clone());
                    vv.read(data, self)?;
                    *v = Some(vv);
                    Ok(())
                } else {
                    Err(line!())
                }
            }
        } else {
            if offs > len {
                Err(line!())
            } else {
                if let Some(o) = data.r_ptr_dict.get(&offs) {
                    *v = Some(o.clone());
                    return Ok(());
                } else {
                    Err(line!())
                }
            }
        }
    }


    /// 填充RC
    pub(crate) fn read_rc<T: IBase + IObjectBase + 'static>(
        &self,
        data: &mut Data,
        v: &mut Option<Rc<T>>,
    ) -> Result<(), u32> {
        let type_id = {
            match data.read_bit7_u16() {
                None => {
                    return Err(line!());
                }
                Some((_, type_id)) => {
                    if type_id == 0 {
                        *v = None;
                        return Ok(());
                    }
                    type_id
                }
            }
        };

        let offs = {
            match data.read_bit7_u32() {
                None => {
                    return Err(line!());
                }
                Some((_, offs)) => offs,
            }
        };

        let len = data.r_ptr_dict.len() as u32;
        if offs == len + 1 {
            if T::get_static_typeid() != type_id {
                Err(line!())
            } else {
                if let Some(ref v) = v {
                    data.r_ptr_dict.insert(offs, v.clone());
                    v.read(data, self)?;
                    Ok(())
                } else {
                    let vv = self.create(type_id);
                    if let Some(vv) = vv {
                        let vv = vv.cast::<T>();
                        if let Ok(vv) = vv {
                            data.r_ptr_dict.insert(offs, vv.clone());
                            vv.read(data, self)?;
                            *v = Some(vv);
                            Ok(())
                        } else {
                            Err(line!())
                        }
                    } else {
                        Err(line!())
                    }
                }
            }
        } else {
            if offs > len {
                Err(line!())
            } else {
                if let Some(o) = data.r_ptr_dict.get(&offs) {
                    if let Ok(vv) = o.clone().cast::<T>() {
                        *v = Some(vv);
                        return Ok(());
                    } else {
                        Err(line!())
                    }
                } else {
                    Err(line!())
                }
            }
        }
    }

    /// 读取weak
    pub(crate) fn read_weak<T: IBase + IObjectBase + 'static>(
        &self,
        data: &mut Data,
        v: &mut Option<Weak<T>>,
    ) -> Result<(), u32> {
        let mut rc: Option<Rc<T>> = None;
        self.read_rc(data, &mut rc)?;
        if let Some(rc) = rc {
            *v = Some(Rc::downgrade(&rc))
        } else {
            *v = None;
        }
        Ok(())
    }

    /// 读取一个obj
    pub(crate) fn read_obj<T: IBase + 'static>(
        &self,
        data: &mut Data,
        v: &mut T,
    ) -> Result<(), u32> {
        v.read(data, self)
    }

    /// 读取一个option
    pub(crate) fn read_option<T: ReadObject + Default>(
        &self,
        data: &mut Data,
        v: &mut Option<T>,
    ) -> Result<(), u32> {
        if data.get_u8() == 1 {
            if let Some(v) = v {
                v.read_(data, self)?;
                Ok(())
            } else {
                let mut x = T::default();
                x.read_(data, self)?;
                *v = Some(x);
                Ok(())
            }
        } else {
            *v = None;
            Ok(())
        }
    }

    ///读取 VEC<T> T为常规实现了default+ReadObject 的类型
    pub(crate) fn read_vec_default<T: ReadObject + Default>(
        &self,
        data: &mut Data,
        v: &mut Vec<T>,
    ) -> Result<(), u32> {
        let size = {
            match data.read_bit7_u64() {
                None => {
                    return Err(line!());
                }
                Some((_, size)) => size,
            }
        };

        for _ in 0..size {
            let mut p = T::default();
            p.read_(data, self)?;
            v.push(p);
        }

        Ok(())
    }

    ///读取vec<T> T 为RC<IBASE>
    pub(crate) fn read_vec_rc<T: IBase + IObjectBase + 'static>(
        &self,
        data: &mut Data,
        v: &mut Vec<Rc<T>>,
    ) -> Result<(), u32> {
        let size = {
            match data.read_bit7_u64() {
                None => {
                    return Err(line!());
                }
                Some((_, size)) => size,
            }
        };

        for _ in 0..size {
            let mut vv: Option<Rc<T>> = None;
            self.read_rc(data, &mut vv)?;
            if let Some(p) = vv {
                v.push(p);
            } else {
                break;
            }
        }

        Ok(())
    }

    ///读取一个weak
    pub(crate) fn read_vec_weak<T: IBase + IObjectBase + 'static>(
        &self,
        data: &mut Data,
        v: &mut Vec<Weak<T>>,
    ) -> Result<(), u32> {
        let size = {
            match data.read_bit7_u64() {
                None => {
                    return Err(line!());
                }
                Some((_, size)) => size,
            }
        };

        for _ in 0..size {
            let mut vv: Option<Rc<T>> = None;
            self.read_rc(data, &mut vv)?;
            if let Some(p) = vv {
                v.push(Rc::downgrade(&p));
            } else {
                break;
            }
        }

        Ok(())
    }

    ///读取treemap 默认值KEY 默认值VALUE
    pub(crate) fn read_treemap_rc_default<
        K: ReadObject + Default + Ord,
        V: ReadObject + Default,
    >(
        &self,
        data: &mut Data,
        v: &mut BTreeMap<K, V>,
    ) -> Result<(), u32> {
        let size = {
            match data.read_bit7_u64() {
                None => {
                    return Err(line!());
                }
                Some((_, size)) => size,
            }
        };

        for _ in 0..size {
            let mut key = K::default();
            let mut value = V::default();
            key.read_(data, self)?;
            value.read_(data, self)?;
            v.insert(key, value);
        }

        Ok(())
    }

    ///读取treemap 默认值KEY,RC<IBASE> Value
    pub(crate) fn read_treemap_rc_key_default<
        K: ReadObject + Default + Ord,
        V: IBase + IObjectBase + 'static,
    >(
        &self,
        data: &mut Data,
        v: &mut BTreeMap<K, Rc<V>>,
    ) -> Result<(), u32> {
        let size = {
            match data.read_bit7_u64() {
                None => {
                    return Err(line!());
                }
                Some((_, size)) => size,
            }
        };

        for _ in 0..size {
            let mut key = K::default();
            key.read_(data, self)?;
            let mut value: Option<Rc<V>> = None;
            value.read_(data, self)?;

            if let Some(value) = value {
                v.insert(key, value);
            } else {
                return Err(line!());
            }
        }

        Ok(())
    }

    ///读取hashmap 默认值KEY 默认值VALUE
    pub(crate) fn read_hashmap_rc_default<
        K: ReadObject + Default + Eq + Hash,
        V: ReadObject + Default,
    >(
        &self,
        data: &mut Data,
        v: &mut HashMap<K, V>,
    ) -> Result<(), u32> {
        let size = {
            match data.read_bit7_u64() {
                None => {
                    return Err(line!());
                }
                Some((_, size)) => size,
            }
        };

        for _ in 0..size {
            let mut key = K::default();
            let mut value = V::default();
            key.read_(data, self)?;
            value.read_(data, self)?;
            v.insert(key, value);
        }

        Ok(())
    }

    ///读取hashmap 默认值KEY,RC<IBASE> Value
    pub(crate) fn read_hashmap_rc_key_default<
        K: ReadObject + Default + Eq + Hash,
        V: IBase + IObjectBase + 'static,
    >(
        &self,
        data: &mut Data,
        v: &mut HashMap<K, Rc<V>>,
    ) -> Result<(), u32> {
        let size = {
            match data.read_bit7_u64() {
                None => {
                    return Err(line!());
                }
                Some((_, size)) => size,
            }
        };

        for _ in 0..size {
            let mut key = K::default();
            key.read_(data, self)?;
            let mut value: Option<Rc<V>> = None;
            value.read_(data, self)?;

            if let Some(value) = value {
                v.insert(key, value);
            } else {
                return Err(line!());
            }
        }

        Ok(())
    }

    ///读取hashmap RC<IBASE> KEY Value
    pub(crate) fn read_hashmap_rc<
        K: IBase + IObjectBase + Eq + Hash + 'static,
        V: IBase + IObjectBase + 'static,
    >(
        &self,
        data: &mut Data,
        v: &mut HashMap<Rc<K>, Rc<V>>,
    ) -> Result<(), u32> {
        let size = {
            match data.read_bit7_u64() {
                None => {
                    return Err(line!());
                }
                Some((_, size)) => size,
            }
        };

        for _ in 0..size {
            let mut key: Option<Rc<K>> = None;
            key.read_(data, self)?;

            let mut value: Option<Rc<V>> = None;
            value.read_(data, self)?;

            if let Some(key) = key {
                if let Some(value) = value {
                    v.insert(key, value);
                } else {
                    return Err(line!());
                }
            } else {
                return Err(line!());
            }
        }

        Ok(())
    }

    ///读取hashmap RC<IBASE> Key  默认值Value
    pub(crate) fn read_hashmap_rc_value_default<
        K: IBase + IObjectBase + Eq + Hash + 'static,
        V: ReadObject + Default,
    >(
        &self,
        data: &mut Data,
        v: &mut HashMap<Rc<K>, V>,
    ) -> Result<(), u32> {
        let size = {
            match data.read_bit7_u64() {
                None => {
                    return Err(line!());
                }
                Some((_, size)) => size,
            }
        };

        for _ in 0..size {
            let mut key: Option<Rc<K>> = None;
            key.read_(data, self)?;

            let mut value = V::default();
            value.read_(data, self)?;

            if let Some(key) = key {
                v.insert(key, value);
            } else {
                return Err(line!());
            }
        }

        Ok(())
    }
}

pub trait WriteObject {
    fn write_(&self, data: &mut Data, obj_manager: &ObjectManager);
}

impl<T: IBase + WriteObject> WriteObject for Option<Rc<T>> {
    fn write_(&self, data: &mut Data, obj_manager: &ObjectManager) {
        obj_manager.write_rc(data, self);
    }
}

impl<T: IBase + WriteObject> WriteObject for Option<Weak<T>> {
    fn write_(&self, data: &mut Data, obj_manager: &ObjectManager) {
        obj_manager.write_weak(data, self);
    }
}

impl<T: IBase> WriteObject for T {
    fn write_(&self, data: &mut Data, obj_manager: &ObjectManager) {
        obj_manager.write_obj(data, self);
    }
}

impl<T: WriteObject> WriteObject for Option<T> {
    fn write_(&self, data: &mut Data, obj_manager: &ObjectManager) {
        obj_manager.write_option(data, self);
    }
}

impl<T: WriteObject> WriteObject for Vec<T> {
    fn write_(&self, data: &mut Data, obj_manager: &ObjectManager) {
        obj_manager.write_vec(data, self);
    }
}

impl<T: IBase> WriteObject for Vec<Rc<T>> {
    fn write_(&self, data: &mut Data, obj_manager: &ObjectManager) {
        obj_manager.write_vec_rc(data, self);
    }
}

impl WriteObject for String {
    fn write_(&self, data: &mut Data, obj_manager: &ObjectManager) {
        obj_manager.write_string(data, self);
    }
}

impl<K: WriteObject, V: WriteObject> WriteObject for BTreeMap<K, V> {
    fn write_(&self, data: &mut Data, obj_manager: &ObjectManager) {
        obj_manager.write_treemap(data, self);
    }
}

impl<K: WriteObject, V: WriteObject> WriteObject for HashMap<K, V> {
    fn write_(&self, data: &mut Data, obj_manager: &ObjectManager) {
        obj_manager.write_hashmap(data, self);
    }
}

impl<K: WriteObject, V: IBase> WriteObject for HashMap<K, Rc<V>> {
    fn write_(&self, data: &mut Data, obj_manager: &ObjectManager) {
        obj_manager.write_hashmap_rc_value(data, self);
    }
}

impl<K: IBase, V: WriteObject> WriteObject for HashMap<Rc<K>, V> {
    fn write_(&self, data: &mut Data, obj_manager: &ObjectManager) {
        obj_manager.write_hashmap_rc_key(data, self);
    }
}

impl<K: IBase, V: IBase> WriteObject for HashMap<Rc<K>, Rc<V>> {
    fn write_(&self, data: &mut Data, obj_manager: &ObjectManager) {
        obj_manager.write_hashmap_rc(data, self);
    }
}

impl<K: WriteObject, V: IBase> WriteObject for BTreeMap<K, Rc<V>> {
    fn write_(&self, data: &mut Data, obj_manager: &ObjectManager) {
        obj_manager.write_treemap_rc_value(data, self);
    }
}

macro_rules! impl_integer_var {
    ($type:ty) => {
        impl WriteObject for $type {
            fn write_(&self, data: &mut Data, _: &ObjectManager) {
                data.write_bit7(*self)
            }
        }
    };
}

impl WriteObject for i8 {
    fn write_(&self, data: &mut Data, _: &ObjectManager) {
        data.put_i8(*self);
    }
}

impl WriteObject for u8 {
    fn write_(&self, data: &mut Data, _: &ObjectManager) {
        data.put_u8(*self);
    }
}

impl_integer_var!(i16);
impl_integer_var!(u16);
impl_integer_var!(i32);
impl_integer_var!(u32);
impl_integer_var!(i64);
impl_integer_var!(u64);

impl WriteObject for f32 {
    fn write_(&self, data: &mut Data, _: &ObjectManager) {
        data.put_f32_le(*self);
    }
}

impl WriteObject for f64 {
    fn write_(&self, data: &mut Data, _: &ObjectManager) {
        data.put_f64_le(*self);
    }
}

impl<T: WriteObject> WriteObject for RefCell<T> {
    fn write_(&self, data: &mut Data, obj_manager: &ObjectManager) {
        self.borrow().write_(data, obj_manager);
    }
}

impl<T: WriteObject + Copy> WriteObject for Cell<T> {
    fn write_(&self, data: &mut Data, obj_manager: &ObjectManager) {
        self.get().write_(data, obj_manager);
    }
}



pub trait ReadObject {
    fn read_(&mut self, data: &mut Data, obj_manager: &ObjectManager) -> Result<(), u32>;
}

pub trait ReadOnlyObject {
    fn read_(&self, data: &mut Data, obj_manager: &ObjectManager) -> Result<(), u32>;
}

impl<T: IBase + 'static> ReadObject for T {
    fn read_(&mut self, data: &mut Data, obj_manager: &ObjectManager) -> Result<(), u32> {
        obj_manager.read_obj(data, self)
    }
}



impl ReadObject for Option<Rc<dyn IBase>> {
    fn read_(&mut self, data: &mut Data, obj_manager: &ObjectManager) -> Result<(), u32> {
        obj_manager.read_rc_ibase(data, self)
    }
}

impl<T: IBase + IObjectBase + 'static> ReadObject for Option<Rc<T>> {
    fn read_(&mut self, data: &mut Data, obj_manager: &ObjectManager) -> Result<(), u32> {
        obj_manager.read_rc(data, self)
    }
}

impl<T: IBase + IObjectBase + 'static> ReadObject for Option<Weak<T>> {
    fn read_(&mut self, data: &mut Data, obj_manager: &ObjectManager) -> Result<(), u32> {
        obj_manager.read_weak(data, self)
    }
}

impl<T: ReadObject + Default> ReadObject for Option<T> {
    fn read_(&mut self, data: &mut Data, obj_manager: &ObjectManager) -> Result<(), u32> {
        obj_manager.read_option(data, self)
    }
}

impl<T: ReadObject + Default> ReadObject for Vec<T> {
    fn read_(&mut self, data: &mut Data, obj_manager: &ObjectManager) -> Result<(), u32> {
        obj_manager.read_vec_default(data, self)
    }
}

impl<T: IBase + IObjectBase + 'static> ReadObject for Vec<Rc<T>> {
    fn read_(&mut self, data: &mut Data, obj_manager: &ObjectManager) -> Result<(), u32> {
        obj_manager.read_vec_rc(data, self)
    }
}

impl<T: IBase + IObjectBase + 'static> ReadObject for Vec<Weak<T>> {
    fn read_(&mut self, data: &mut Data, obj_manager: &ObjectManager) -> Result<(), u32> {
        obj_manager.read_vec_weak(data, self)
    }
}


impl<K: ReadObject + Default + Ord, V: ReadObject + Default> ReadObject for BTreeMap<K, V> {
    fn read_(&mut self, data: &mut Data, obj_manager: &ObjectManager) -> Result<(), u32> {
        obj_manager.read_treemap_rc_default(data, self)
    }
}

impl<K: ReadObject + Default + Ord, V: IBase + IObjectBase + 'static> ReadObject
    for BTreeMap<K, Rc<V>>
{
    fn read_(&mut self, data: &mut Data, obj_manager: &ObjectManager) -> Result<(), u32> {
        obj_manager.read_treemap_rc_key_default(data, self)
    }
}

impl<K: ReadObject + Default + Eq + Hash, V: ReadObject + Default> ReadObject for HashMap<K, V> {
    fn read_(&mut self, data: &mut Data, obj_manager: &ObjectManager) -> Result<(), u32> {
        obj_manager.read_hashmap_rc_default(data, self)
    }
}

impl<K: ReadObject + Default + Eq + Hash, V: IBase + IObjectBase + 'static> ReadObject
    for HashMap<K, Rc<V>>
{
    fn read_(&mut self, data: &mut Data, obj_manager: &ObjectManager) -> Result<(), u32> {
        obj_manager.read_hashmap_rc_key_default(data, self)
    }
}

impl<K: IBase + IObjectBase + Eq + Hash + 'static, V: IBase + IObjectBase + 'static> ReadObject
    for HashMap<Rc<K>, Rc<V>>
{
    fn read_(&mut self, data: &mut Data, obj_manager: &ObjectManager) -> Result<(), u32> {
        obj_manager.read_hashmap_rc(data, self)
    }
}

impl<K: IBase + IObjectBase + Eq + Hash + 'static, V: ReadObject + Default> ReadObject
    for HashMap<Rc<K>, V>
{
    fn read_(&mut self, data: &mut Data, obj_manager: &ObjectManager) -> Result<(), u32> {
        obj_manager.read_hashmap_rc_value_default(data, self)
    }
}

impl ReadObject for String {
    fn read_(&mut self, data: &mut Data, _: &ObjectManager) -> Result<(), u32> {
        match data.get_str_bit7() {
            None => return Err(line!()),
            Some(p) => *self = p,
        }
        Ok(())
    }
}

macro_rules! impl_read_object_integer {
    ($type:ty) => {
        impl ReadObject for $type {
            fn read_(&mut self, data: &mut Data, _: &ObjectManager) -> Result<(), u32> {
                match data.get_bit7::<$type>() {
                    None => Err(line!()),
                    Some((_, p)) => {
                        *self = p;
                        Ok(())
                    }
                }
            }
        }
    };
}

impl_read_object_integer!(i16);
impl_read_object_integer!(u16);
impl_read_object_integer!(i32);
impl_read_object_integer!(u32);
impl_read_object_integer!(i64);
impl_read_object_integer!(u64);

impl ReadObject for i8 {
    fn read_(&mut self, data: &mut Data, _: &ObjectManager) -> Result<(), u32> {
        *self = data.get_i8();
        Ok(())
    }
}

impl ReadObject for u8 {
    fn read_(&mut self, data: &mut Data, _: &ObjectManager) -> Result<(), u32> {
        *self = data.get_u8();
        Ok(())
    }
}

impl ReadObject for i128 {
    fn read_(&mut self, data: &mut Data, _: &ObjectManager) -> Result<(), u32> {
        *self = data.get_i128_le();
        Ok(())
    }
}

impl ReadObject for u128 {
    fn read_(&mut self, data: &mut Data, _: &ObjectManager) -> Result<(), u32> {
        *self = data.get_u128_le();
        Ok(())
    }
}

impl ReadObject for f32 {
    fn read_(&mut self, data: &mut Data, _: &ObjectManager) -> Result<(), u32> {
        *self = data.get_f32_le();
        Ok(())
    }
}

impl ReadObject for f64 {
    fn read_(&mut self, data: &mut Data, _: &ObjectManager) -> Result<(), u32> {
        *self = data.get_f64_le();
        Ok(())
    }
}

impl<T: ReadObject + Copy> ReadOnlyObject for Cell<T> {
    fn read_(&self, data: &mut Data, obj_manager: &ObjectManager) -> Result<(), u32> {
        let mut v = self.get();
        v.read_(data, obj_manager)?;
        self.set(v);
        Ok(())
    }
}

impl<T: ReadObject> ReadOnlyObject for RefCell<T> {
    fn read_(&self, data: &mut Data, obj_manager: &ObjectManager) -> Result<(), u32> {
        self.borrow_mut().read_(data, obj_manager)
    }
}

