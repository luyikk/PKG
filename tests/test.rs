mod test_struct;

use bytes::{Buf, BufMut};
use pkg::interface::{GetValue, IBase, IObjectBase};
use pkg::object_manager::{IObjectManager, ObjectManager};
use pkg::{Data, IBaseAsRc};
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::rc::Rc;
use test_struct::*;

#[test]
fn test_buff() -> Result<(), Box<dyn Error>> {
    let mut data = Data::new();
    let test_data = b"1231233122";
    for _ in 0..1000 {
        data.write(test_data);
    }

    for _ in 0..1000 {
        let v = data.read(test_data.len());
        if let Some(p) = v {
            assert_eq!(p, test_data);
        } else {
            panic!("is none");
        }
    }

    data.set_position(0);
    for _ in 0..1000 {
        let v = data.read(test_data.len());
        if let Some(p) = v {
            assert_eq!(p, test_data);
        } else {
            panic!("is none");
        }
    }

    data.set_position(20);
    for _ in 0..998 {
        let v = data.read(test_data.len());
        if let Some(p) = v {
            assert_eq!(p, test_data);
        } else {
            panic!("is none");
        }
    }

    Ok(())
}

#[test]
pub fn test_buf_mut() -> Result<(), Box<dyn Error>> {
    let mut data = Data::default();
    data.put_u32_le(10);
    data.put_i64_le(1000i64);
    data.put_f64_le(100.0);
    data.put_i32(100);

    assert_eq!(data.get_u32_le(), 10);
    assert_eq!(data.get_i64_le(), 1000);
    assert_eq!(data.get_f64_le(), 100.0);
    assert_eq!(data.get_i32(), 100);

    data.write_to_le(1i32);
    data.write_to_le(2i64);
    data.write_to_le(3u64);
    data.write_to_le(4u32);

    assert_eq!(data.get_le::<i32>(), 1);
    assert_eq!(data.get_le::<i64>(), 2);
    assert_eq!(data.get_le::<u64>(), 3);
    assert_eq!(data.get_le::<u32>(), 4);

    data.write_to(1.0f32);
    data.write_to(2.1f64);
    data.write_to(2u8);
    data.write_to(2u16);
    data.write_to(true);
    data.write_to(false);

    assert_eq!(data.get::<f32>(), 1.0f32);
    assert_eq!(data.get::<f64>(), 2.1f64);
    assert_eq!(data.get::<u8>(), 2u8);
    assert_eq!(data.get::<u16>(), 2u16);
    assert_eq!(data.get::<bool>(), true);
    assert_eq!(data.get::<bool>(), false);

    data.set_position(0);
    assert_eq!(data.get_u32_le(), 10);
    assert_eq!(data.get_i64_le(), 1000);
    assert_eq!(data.get_f64_le(), 100.0);
    assert_eq!(data.get_i32(), 100);
    assert_eq!(data.get_le::<i32>(), 1);
    assert_eq!(data.get_le::<i64>(), 2);
    assert_eq!(data.get_le::<u64>(), 3);
    assert_eq!(data.get_le::<u32>(), 4);
    assert_eq!(data.get::<f32>(), 1.0f32);
    assert_eq!(data.get::<f64>(), 2.1f64);
    assert_eq!(data.get::<u8>(), 2u8);
    assert_eq!(data.get::<u16>(), 2u16);
    assert_eq!(data.get::<bool>(), true);
    assert_eq!(data.get::<bool>(), false);

    Ok(())
}

#[test]
pub fn test_bit7() -> Result<(), Box<dyn Error>> {
    let mut data = Data::default();
    data.bit7_write_i16(1i16);
    data.bit7_write_i32(2i32);
    data.bit7_write_i64(3i64);
    data.bit7_write_u16(1u16);
    data.bit7_write_u32(2u32);
    data.bit7_write_u64(3u64);

    assert_eq!(data.read_bit7_i16(), Some((1, 1)));
    assert_eq!(data.read_bit7_i32(), Some((1, 2)));
    assert_eq!(data.read_bit7_i64(), Some((1, 3)));

    assert_eq!(data.read_bit7_u16(), Some((1, 1)));
    assert_eq!(data.read_bit7_u32(), Some((1, 2)));
    assert_eq!(data.read_bit7_u64(), Some((1, 3)));

    data.write_bit7(1i16);
    data.write_bit7(2i32);
    data.write_bit7(3i64);
    data.write_bit7(1u16);
    data.write_bit7(2u32);
    data.write_bit7(3u64);

    assert_eq!(data.get_bit7::<i16>(), Some((1, 1)));
    assert_eq!(data.get_bit7::<i32>(), Some((1, 2)));
    assert_eq!(data.get_bit7::<i64>(), Some((1, 3)));
    assert_eq!(data.get_bit7::<u16>(), Some((1, 1)));
    assert_eq!(data.get_bit7::<u32>(), Some((1, 2)));
    assert_eq!(data.get_bit7::<u64>(), Some((1, 3)));

    data.set_position(1);
    for _ in 0..100 {
        assert_eq!(data.read_bit7_i32(), Some((1, 2)));
        data.set_position(data.get_position() - 1);
    }
    Ok(())
}

#[test]
pub fn test_buff_str() -> Result<(), Box<dyn Error>> {
    let mut data = Data::new();
    data.write_buff_bit7(b"1234567890");
    data.write_str_bit7(&"asfasf".to_string());
    assert_eq!(b"1234567890".to_vec(), data.get_buff_bit7().unwrap());
    assert_eq!("asfasf", data.get_str_bit7().unwrap());

    data.write_buff_fixed_le(b"1234567890");
    data.write_str_fixed_le("asfasf hhi 操");
    assert_eq!(b"1234567890".to_vec(), data.get_buff_fixed_le().unwrap());
    assert_eq!("asfasf hhi 操", data.get_str_fixed_le().unwrap());

    data.write_buff_fixed(b"1234567890");
    data.write_str_fixed("asfasf hhi 操");
    assert_eq!(b"1234567890".to_vec(), data.get_buff_fixed().unwrap());
    assert_eq!("asfasf hhi 操", data.get_str_fixed().unwrap());

    Ok(())
}

#[test]
pub fn test_write_core() {
    let mut data = Data::with_capacity(1024);
    data.write_core(&"123123123");
    data.write_core(&"3333333333".to_string());

    let p = vec![1, 2, 3, 4, 5, 6, 7];
    data.write_core(&p);

    let mut v = Vec::new();
    let mut v1 = Vec::new();
    let mut v2 = Vec::new();
    v2.push("asdfqwer");
    v1.push(v2);
    v.push(v1);

    data.write_core(&v);

    assert_eq!("123123123", data.read_core::<String>());
    assert_eq!("3333333333", data.read_core::<String>());
    assert_eq!(vec![1, 2, 3, 4, 5, 6, 7], data.read_core::<Vec<i32>>());
    assert_eq!(v, data.read_core::<Vec<Vec<Vec<String>>>>());
}

#[test]
pub fn test_rc() {
    #[derive(Debug)]
    struct NX {
        i: i32,
    };

    impl IBase for NX {
        fn write(&self, _: &mut Data, _: &ObjectManager) {
            unimplemented!()
        }

        fn read(&self, _: &mut Data, _: &ObjectManager) -> std::result::Result<(), u32> {
            unimplemented!()
        }

        fn get_typeid(&self) -> u16 {
            NX::get_static_typeid()
        }
    }

    impl IObjectBase for NX {
        fn get_static_typeid() -> u16 {
            1
        }

        fn new() -> Option<Rc<dyn IBase>> {
            Some(Rc::new(NX { i: 10000 }))
        }
    }

    impl Default for NX {
        fn default() -> Self {
            NX { i: 0 }
        }
    }

    let x = NX::new().unwrap();
    let x = x.cast::<NX>();

    if let Ok(p) = x {
        assert_eq!(p.i, 10000);
    } else {
        panic!("error")
    }
}

#[test]
pub fn test_obj_manager_register_create() {
    let mut obj_manager = ObjectManager::new();
    obj_manager.register::<Path>();
    obj_manager.register::<PathBase>();

    let path = obj_manager.create(1).unwrap();
    let path_base = obj_manager.create(2).unwrap();

    assert!(obj_manager.create(4).is_none());

    let path = path.cast::<Path>().unwrap();
    let path_base = path_base.cast::<PathBase>().unwrap();

    path_base.path.set(path);
    assert_eq!(1, path_base.index.get());

    if let Some(p) = path_base.path.get() {
        assert_eq!(Some("abcdefg".to_string()), *p.name.borrow());
    } else {
        panic!("error");
    }

    unsafe {
        if let Some(p) = path_base.path.get_unsafe_ref() {
            if let Some(name) = p.name.get_unsafe_refmut() {
                *name = "aaabbbccc".to_string();
            }
            assert_eq!(Some("aaabbbccc".to_string()), *p.name.borrow());
        } else {
            panic!("error");
        }
    }
}

#[test]
pub fn test_struct_1() -> Result<(), u32> {

    fn new_path() -> Path {
        Path {
            x: Cell::new(1000),
            y: Cell::new(10000),
            name: RefCell::new(Some("abcdefg".to_string())),
        }
    }

    fn new_path_base() -> Rc<PathBase> {
        let path_base = PathBase::default();
        path_base.index.set(100);
        path_base.path.set(Rc::new(new_path()));
        Rc::new(path_base)
    }


    let mut obj_manager = ObjectManager::new();
    obj_manager.register::<Path>();
    obj_manager.register::<PathBase>();
    let mut data = Data::new();


    obj_manager.write_to(&mut data, &1);
    obj_manager.write_to(&mut data, &1.0f32);
    obj_manager.write_to(&mut data, &"123123123".to_string());

    let a = vec![1, 2, 3, 4, 5];
    obj_manager.write_to(&mut data, &a);

    let path = new_path();
    obj_manager.write_to(&mut data, &path);

    let path_array = vec![new_path(), new_path(), new_path(), new_path(), new_path()];
    obj_manager.write_to(&mut data, &path_array);
    obj_manager.write_to(&mut data, &Some(new_path_base()));

    let mut hashmap = HashMap::new();
    hashmap.insert(1, Some(new_path_base()));
    obj_manager.write_to(&mut data, &hashmap);

    let mut treemap = BTreeMap::new();
    treemap.insert("map".to_string(), hashmap);
    obj_manager.write_to(&mut data, &treemap);

    let vec = vec![new_path_base(), new_path_base(), new_path_base()];
    obj_manager.write_to(&mut data, &vec);

   // ---------------------read----------------------------------------
    let mut i = i32::default();
    obj_manager.read_from(&mut data, &mut i)?;
    assert_eq!(1, i);

    let mut i = f32::default();
    obj_manager.read_from(&mut data, &mut i)?;
    assert_eq!(1.0, i);

    let mut i = String::default();
    obj_manager.read_from(&mut data, &mut i)?;
    assert_eq!("123123123", i);

    let mut i = Vec::default();
    obj_manager.read_from(&mut data, &mut i)?;
    assert_eq!(vec![1, 2, 3, 4, 5], i);

    let mut path = Path::default();
    obj_manager.read_from(&mut data, &mut path)?;
    assert_eq!(new_path(), path);

    let mut vec = vec![];
    obj_manager.read_from(&mut data, &mut vec)?;
    assert_eq!(path_array, vec);

    let mut i: Option<Rc<PathBase>> = None;
    obj_manager.read_from(&mut data, &mut i)?;
    assert_eq!(Some(new_path_base()), i);

    let mut i = HashMap::new();
    obj_manager.read_from(&mut data, &mut i)?;

    let mut test = HashMap::new();
    test.insert(1, Some(new_path_base()));
    assert_eq!(test, i);

    println!("{:?}", data);

    Ok(())
}

#[test]
pub fn test_struct_2() -> Result<(), u32> {
    let mut obj_manager = ObjectManager::new();
    obj_manager.register::<Base>();
    obj_manager.register::<Fly>();
    let mut data = Data::new();
    let make_base = || {
        let base = Base::default();
        base.id.set(1000);
        base.name.replace("test ppp".to_string());
        base.data.replace(vec![1,2,3,4,5]);
        base
    };

    let base = make_base();
    obj_manager.write_to(&mut data, &base);
    let mut test_base = Base::default();
    obj_manager.read_from(&mut data, &mut test_base)?;
    assert_eq!(base, test_base);

    let test=Rc::new(make_base());
    obj_manager.write_core(&mut data,&test);
    let x= obj_manager.read_core(&mut data)?;
    assert_eq!(test,x.cast::<Base>().unwrap());

    let make_fly=||{
        let fly=Fly::default();
        *fly.base.borrow_mut()=make_base();
        fly.x.set(1.1);
        fly.y.set(2.2);
        let base=Rc::new(make_base());
        let base_weak=Rc::downgrade(&base);
        fly.wk.set(base_weak.clone());
        fly.rc.set(base.clone());
        *fly.vec.borrow_mut()=vec![Some(base.clone()),Some(base.clone()),Some(base.clone()),Some(base.clone())];
        *fly.vec_wk.borrow_mut()=vec![Some(base_weak.clone()),Some(base_weak.clone()),Some(base_weak.clone()),Some(base_weak.clone())];
        let mut hashmap=HashMap::new();
        hashmap.insert(1,Some(base.clone()));
        hashmap.insert(2,Some(base.clone()));
        hashmap.insert(3,Some(base.clone()));
        *fly.hash.borrow_mut()=hashmap;

        let mut hashmap=HashMap::new();
        hashmap.insert(1,Some(base_weak.clone()));
        hashmap.insert(2,Some(base_weak.clone()));
        hashmap.insert(3,Some(base_weak.clone()));
        *fly.hash_wk.borrow_mut()=hashmap;

        let mut treemap=BTreeMap::new();
        treemap.insert(1,Some(base.clone()));
        treemap.insert(2,Some(base.clone()));
        treemap.insert(3,Some(base.clone()));
        *fly.treemap.borrow_mut()=treemap;

        let mut treemap=BTreeMap::new();
        treemap.insert(1,Some(base_weak.clone()));
        treemap.insert(2,Some(base_weak.clone()));
        treemap.insert(3,Some(base_weak.clone()));
        *fly.treemap_wk.borrow_mut()=treemap;

        fly
    };

    let test=Rc::new(make_fly());
    obj_manager.write_core(&mut data,&test);
    let x= obj_manager.read_core(&mut data)?;
    let m=x.cast::<Fly>().unwrap();
    assert_eq!(test.base,m.base);
    assert_eq!(test.x,m.x);
    assert_eq!(test.y,m.y);

    let y= m.rc.get().unwrap().clone();
    assert_eq!(Rc::new(make_base()),y);
    let x= m.wk.get().unwrap().upgrade().unwrap();
    assert_eq!(x,y);

    let base=y.clone();
    assert_eq!(vec![Some(base.clone()),Some(base.clone()),Some(base.clone()),Some(base.clone())],*m.vec.borrow_mut());
    for x in m.vec_wk.borrow().iter() {
        assert_eq!(base,x.clone().unwrap().upgrade().unwrap());
    }

    for (key,value) in test.hash.borrow().iter() {
        assert!(*key>0&&*key<4,"{}",key);
        assert_eq!(base,value.clone().unwrap());
    }

    for (key,value) in test.hash_wk.borrow().iter() {
        assert!(*key>0&&*key<4,"{}",key);
        assert_eq!(base,value.clone().unwrap().upgrade().unwrap());
    }
    for (i,(key,value)) in test.treemap.borrow().iter().enumerate() {
        assert_eq!(i as i64+1,*key);
        assert_eq!(base,value.clone().unwrap());
    }

    for (i,(key,value)) in test.treemap_wk.borrow().iter().enumerate() {
        assert_eq!(i as i64+1,*key);
        assert_eq!(base,value.clone().unwrap().upgrade().unwrap());
    }

    Ok(())
}
