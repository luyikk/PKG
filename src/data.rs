use crate::IBase;
use bytes::buf::UninitSlice;
use bytes::{Buf, BufMut};
use paste::paste;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct Data {
    buf: Vec<u8>,
    offset: usize,
    pub(crate) w_ptr_dict: HashMap<usize, u32>,
    pub(crate) r_ptr_dict: HashMap<u32, Rc<dyn IBase>>,
}

unsafe impl BufMut for Data {
    #[inline]
    fn remaining_mut(&self) -> usize {
        self.buf.remaining_mut()
    }

    #[inline]
    unsafe fn advance_mut(&mut self, cnt: usize) {
        self.buf.advance_mut(cnt);
    }

    #[inline]
    fn bytes_mut(&mut self) -> &mut UninitSlice {
        self.buf.bytes_mut()
    }
}

impl Buf for Data {
    #[inline]
    fn remaining(&self) -> usize {
        if self.buf.len() > self.offset {
            self.buf.len() - self.offset
        } else {
            0
        }
    }

    #[inline]
    fn bytes(&self) -> &[u8] {
        let len = self.remaining();

        if len > 0 {
            &self.buf[self.offset..]
        } else {
            &[]
        }
    }

    #[inline]
    fn advance(&mut self, cnt: usize) {
        self.offset += cnt;
    }
}

impl Default for Data {
    #[inline]
    fn default() -> Self {
        Data {
            buf: Vec::new(),
            offset: 0,
            w_ptr_dict: HashMap::new(),
            r_ptr_dict: HashMap::new(),
        }
    }
}

impl Data {
    #[inline]
    pub fn new() -> Data {
        Data::default()
    }

    #[inline]
    pub fn with_capacity(cap: usize) -> Data {
        Data {
            buf: Vec::with_capacity(cap),
            offset: 0,
            w_ptr_dict: HashMap::new(),
            r_ptr_dict: HashMap::new(),
        }
    }

    /// 获取OFFSET
    #[inline]
    pub fn get_position(&self) -> usize {
        self.offset
    }

    /// 设置OFFSET
    #[inline]
    pub fn set_position(&mut self, offset: usize) -> bool {
        if offset > self.buf.len() {
            return false;
        }
        self.offset = offset;
        return true;
    }

    /// 长度
    #[inline]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// 重置
    #[inline]
    pub fn reset(&mut self) {
        self.buf.resize(0, 0);
        self.offset = 0;
    }

    /// 重置长度
    #[inline]
    pub fn resize(&mut self,new_len:usize,value:u8) {
        self.buf.resize(new_len, value);
    }

    /// 清理
    #[inline]
    pub fn clear(&mut self) {
        self.buf.clear();
        self.offset = 0;
    }

    /// 写入buff
    #[inline]
    pub fn write(&mut self, buff: &[u8]) {
        //self.buf.extend_from_slice(buff)
        self.write_ptr(buff.as_ptr(),buff.len());
    }

    #[inline]
    pub fn write_ptr(&mut self, buff: *const u8, len: usize) {
        if self.buf.len() + len > self.buf.capacity() {
            self.buf.reserve(len);
        }
        unsafe {
            let end = self.buf.as_mut_ptr().add(self.buf.len());
            end.copy_from(buff, len);
            self.buf.set_len(self.buf.len() + len);
        }
    }

    /// 读取buff
    #[inline]
    pub fn read(&mut self, len: usize) -> Option<Vec<u8>> {
        if self.buf.len() >= self.offset + len {
            let data = self.buf[self.offset..self.offset + len].to_vec();
            self.offset += len;
            Some(data)
        } else {
            None
        }
    }
    /// 写入变成U16
    #[inline]
    pub fn bit7_write_u16(&mut self, value: u16) {
        let mut v = value;
        while v >= 1 << 7 {
            self.buf.push((v & 0x7f | 0x80) as u8);
            v = v >> 7;
        }
        self.buf.push(v as u8);
    }
    /// 写入变成U32
    #[inline]
    pub fn bit7_write_u32(&mut self, value: u32) {
        let mut v = value;
        while v >= 1 << 7 {
            self.buf.push((v & 0x7f | 0x80) as u8);
            v = v >> 7;
        }
        self.buf.push(v as u8);
    }
    /// 写入变成U64
    #[inline]
    pub fn bit7_write_u64(&mut self, value: u64) {
        let mut v = value;
        while v >= 1 << 7 {
            self.buf.push((v & 0x7f | 0x80) as u8);
            v = v >> 7;
        }
        self.buf.push(v as u8);
    }
    /// 写入变长i64
    #[inline]
    pub fn bit7_write_i16(&mut self, value: i16) {
        self.bit7_write_u16(zig_zag_encode_u16(value))
    }
    /// 写入变长i32
    #[inline]
    pub fn bit7_write_i32(&mut self, value: i32) {
        self.bit7_write_u32(zig_zag_encode_u32(value))
    }
    /// 写入变长i64
    #[inline]
    pub fn bit7_write_i64(&mut self, value: i64) {
        self.bit7_write_u64(zig_zag_encode_u64(value))
    }
    /// 读取变长 u16
    #[inline]
    pub fn read_bit7_u16(&mut self) -> Option<(usize, u16)> {
        let mut v = 0;
        let mut offset = self.offset;
        let mut shift = 0;
        while shift < 2 * 8 {
            if offset >= self.buf.len() {
                return None;
            }

            let b = self.buf[offset];
            offset += 1;
            v |= ((b & 0x7F) as u16) << shift;
            if b & 0x80 == 0 {
                let len = offset - self.offset;
                self.offset = offset;
                return Some((len, v));
            }
            shift += 7;
        }
        None
    }
    /// 读取变长i16
    #[inline]
    pub fn read_bit7_i16(&mut self) -> Option<(usize, i16)> {
        if let Some((offset, v)) = self.read_bit7_u16() {
            let v = zig_zag_decode_i16(v);
            Some((offset, v))
        } else {
            None
        }
    }
    /// 读取变长u32
    #[inline]
    pub fn read_bit7_u32(&mut self) -> Option<(usize, u32)> {
        let mut v = 0;
        let mut offset = self.offset;
        let mut shift = 0;
        while shift < 4 * 8 {
            if offset >= self.buf.len() {
                return None;
            }

            let b = self.buf[offset];
            offset += 1;
            v |= ((b & 0x7F) as u32) << shift;
            if b & 0x80 == 0 {
                let len = offset - self.offset;
                self.offset = offset;
                return Some((len, v));
            }
            shift += 7;
        }
        None
    }
    /// 读取变长i32
    #[inline]
    pub fn read_bit7_i32(&mut self) -> Option<(usize, i32)> {
        if let Some((offset, v)) = self.read_bit7_u32() {
            let v = zig_zag_decode_i32(v);
            Some((offset, v))
        } else {
            None
        }
    }
    /// 读取变长u64
    #[inline]
    pub fn read_bit7_u64(&mut self) -> Option<(usize, u64)> {
        let mut v = 0;
        let mut offset = self.offset;
        let mut shift = 0;
        while shift < 8 * 8 {
            if offset >= self.buf.len() {
                return None;
            }

            let b = self.buf[offset];
            offset += 1;
            v |= ((b & 0x7F) as u64) << shift;
            if b & 0x80 == 0 {
                let len = offset - self.offset;
                self.offset = offset;
                return Some((len, v));
            }
            shift += 7;
        }
        None
    }
    /// 读取变长i64
    #[inline]
    pub fn read_bit7_i64(&mut self) -> Option<(usize, i64)> {
        if let Some((offset, v)) = self.read_bit7_u64() {
            let v = zig_zag_decode_i64(v);
            Some((offset, v))
        } else {
            None
        }
    }

    /// 写入二进制变长长度
    #[inline]
    pub fn write_buff_bit7(&mut self, data: &[u8]) {
        self.bit7_write_u64(data.len() as u64);
        self.write(data)
    }

    /// 写入二进制定长长度 u32 le
    #[inline]
    pub fn write_buff_fixed_le(&mut self, data: &[u8]) {
        self.write_to_le(data.len() as u32);
        self.write(data)
    }

    /// 写入二进制定长长度 u32
    #[inline]
    pub fn write_buff_fixed(&mut self, data: &[u8]) {
        self.write_to(data.len() as u32);
        self.write(data)
    }

    /// 写入字符串变长长度
    #[inline]
    pub fn write_str_bit7(&mut self, data: &str) {
        //self.bit7_write_u64(data.len() as u64);
        //self.write_ptr(data.as_ptr(),data.len());
        self.write_buff_bit7(data.as_bytes());
    }

    /// 写入字符串定长长度 u32 le
    #[inline]
    pub fn write_str_fixed_le(&mut self, data: &str) {
        self.write_buff_fixed_le(data.as_bytes())
    }

    /// 写入字符串定长长度 u32
    #[inline]
    pub fn write_str_fixed(&mut self, data: &str) {
        self.write_buff_fixed(data.as_bytes())
    }

    /// 读取二进制 变长长度
    #[inline]
    pub fn get_buff_bit7(&mut self) -> Option<Vec<u8>> {
        let res = self.get_bit7::<u64>();
        if let Some((_, len)) = res {
            self.read(len as usize)
        } else {
            None
        }
    }

    /// 读取字符串 变长
    #[inline]
    pub fn get_str_bit7(&mut self) -> Option<String> {
        if let Some(buff) = self.get_buff_bit7() {
            Some(String::from_utf8_lossy(&buff).to_string())
        } else {
            None
        }
    }

    /// 读取二进制 U32 LE
    #[inline]
    pub fn get_buff_fixed_le(&mut self) -> Option<Vec<u8>> {
        let len = self.get_le::<u32>();
        if len > 0 {
            return self.read(len as usize);
        }
        None
    }
    /// 读取字符串 U32 LE
    #[inline]
    pub fn get_str_fixed_le(&mut self) -> Option<String> {
        if let Some(buff) = self.get_buff_fixed_le() {
            Some(String::from_utf8_lossy(&buff).to_string())
        } else {
            None
        }
    }
    /// 获取二进制 定长 u32
    #[inline]
    pub fn get_buff_fixed(&mut self) -> Option<Vec<u8>> {
        let len = self.get::<u32>();
        if len > 0 {
            return self.read(len as usize);
        }
        None
    }

    /// 获取字符串 定长u32
    #[inline]
    pub fn get_str_fixed(&mut self) -> Option<String> {
        if let Some(buff) = self.get_buff_fixed() {
            Some(String::from_utf8_lossy(&buff).to_string())
        } else {
            None
        }
    }
}
#[inline]
fn zig_zag_encode_u16(v: i16) -> u16 {
    ((v << 1) ^ (v >> 15)) as u16
}
#[inline]
fn zig_zag_encode_u32(v: i32) -> u32 {
    ((v << 1) ^ (v >> 31)) as u32
}
#[inline]
fn zig_zag_encode_u64(v: i64) -> u64 {
    ((v << 1) ^ (v >> 63)) as u64
}
#[inline]
fn zig_zag_decode_i16(v: u16) -> i16 {
    ((v >> 1) as i16) ^ (-((v & 1) as i16))
}
#[inline]
fn zig_zag_decode_i32(v: u32) -> i32 {
    ((v >> 1) as i32) ^ (-((v & 1) as i32))
}
#[inline]
fn zig_zag_decode_i64(v: u64) -> i64 {
    ((v >> 1) as i64) ^ (-((v & 1) as i64))
}

impl Data {
    #[inline]
    pub fn write_to_le<T: Writer>(&mut self, p: T) {
        p.write_to_le(self)
    }
    #[inline]
    pub fn write_to<T: Writer>(&mut self, p: T) {
        p.write_to(self)
    }
}

pub trait Writer: Copy {
    fn write_to_le(&self, data: &mut Data);
    fn write_to(&self, data: &mut Data);
}

macro_rules! make_writer {
    ($type:ty) => {
        impl Writer for $type {
            paste! {
             #[inline]
            fn write_to_le(&self, data:&mut Data) {
               data.[<put_ $type _le>](*self)
            }
             #[inline]
            fn write_to(&self, data: &mut Data) {
                data.[<put_ $type>](*self);
            }
            }
        }
    };
}
impl Writer for bool {
    #[inline]
    fn write_to_le(&self, data: &mut Data) {
        if *self {
            data.put_u8(1);
        } else {
            data.put_u8(0);
        }
    }
    #[inline]
    fn write_to(&self, data: &mut Data) {
        if *self {
            data.put_u8(1);
        } else {
            data.put_u8(0);
        }
    }
}

impl Writer for u8 {
    #[inline]
    fn write_to_le(&self, data: &mut Data) {
        data.put_u8(*self);
    }
    #[inline]
    fn write_to(&self, data: &mut Data) {
        data.put_u8(*self);
    }
}

impl Writer for i8 {
    #[inline]
    fn write_to_le(&self, data: &mut Data) {
        data.put_i8(*self);
    }
    #[inline]
    fn write_to(&self, data: &mut Data) {
        data.put_i8(*self);
    }
}

make_writer!(i16);
make_writer!(i32);
make_writer!(i64);
make_writer!(u16);
make_writer!(u32);
make_writer!(u64);

make_writer!(u128);
make_writer!(i128);
make_writer!(f32);
make_writer!(f64);

impl Data {
    #[inline]
    pub fn get<T: Reader>(&mut self) -> T {
        T::get(self)
    }
    #[inline]
    pub fn get_le<T: Reader>(&mut self) -> T {
        T::get_le(self)
    }
}

pub trait Reader {
    fn get(data: &mut Data) -> Self;
    fn get_le(data: &mut Data) -> Self;
}

macro_rules! make_reader {
    ($type:ty) => {
        impl Reader for $type {
            #[inline]
            fn get(data: &mut Data) -> Self {
                paste! {
                 data.[<get_ $type>]()
                }
            }
            #[inline]
            fn get_le(data: &mut Data) -> Self {
                paste! {
                 data.[<get_ $type _le>]()
                }
            }
        }
    };
}

impl Reader for bool {
    #[inline]
    fn get(data: &mut Data) -> Self {
        if data.get_u8() == 1 {
            true
        } else {
            false
        }
    }
    #[inline]
    fn get_le(data: &mut Data) -> Self {
        if data.get_u8() == 1 {
            true
        } else {
            false
        }
    }
}
impl Reader for u8 {
    #[inline]
    fn get(data: &mut Data) -> Self {
        data.get_u8()
    }
    #[inline]
    fn get_le(data: &mut Data) -> Self {
        data.get_u8()
    }
}
impl Reader for i8 {
    #[inline]
    fn get(data: &mut Data) -> Self {
        data.get_i8()
    }
    #[inline]
    fn get_le(data: &mut Data) -> Self {
        data.get_i8()
    }
}

make_reader!(i16);
make_reader!(i32);
make_reader!(i64);
make_reader!(i128);
make_reader!(u16);
make_reader!(u32);
make_reader!(u64);
make_reader!(u128);
make_reader!(f32);
make_reader!(f64);

impl Data {
    #[inline]
    pub fn write_bit7<T: WriteBit7>(&mut self, p: T) {
        p.write_bit7(self);
    }
}

pub trait WriteBit7: Copy {
    fn write_bit7(&self, data: &mut Data);
}

macro_rules! make_write_bit7 {
    ($type:ty) => {
        impl WriteBit7 for $type {
            #[inline]
            fn write_bit7(&self, data: &mut Data) {
                paste! {
                  data.[<bit7_write_ $type>](*self)
                }
            }
        }
    };
}

make_write_bit7!(i16);
make_write_bit7!(i32);
make_write_bit7!(i64);
make_write_bit7!(u16);
make_write_bit7!(u32);
make_write_bit7!(u64);

impl Data {
    #[inline]
    pub fn get_bit7<T: ReadBit7>(&mut self) -> Option<(usize, T::RetType)> {
        T::get_bit7(self)
    }
}

pub trait ReadBit7 {
    type RetType;
    fn get_bit7(data: &mut Data) -> Option<(usize, Self::RetType)>;
}

macro_rules! make_read_bit7 {
    ($type:ty) => {
        impl ReadBit7 for $type {
            type RetType = $type;
            #[inline]
            fn get_bit7(data: &mut Data) -> Option<(usize, Self::RetType)> {
                paste! {
                data.[<read_bit7_ $type>]()
                }
            }
        }
    };
}

make_read_bit7!(i16);
make_read_bit7!(i32);
make_read_bit7!(i64);
make_read_bit7!(u16);
make_read_bit7!(u32);
make_read_bit7!(u64);

impl Data {
    #[inline]
    pub fn write_core<T: WriteCore>(&mut self, p: &T) {
        p.write_core(self)
    }
}

pub trait WriteCore {
    fn write_core(&self, data: &mut Data);
}

impl WriteCore for &str {
    #[inline]
    fn write_core(&self, data: &mut Data) {
        data.write_str_bit7(self)
    }
}

impl WriteCore for String {
    #[inline]
    fn write_core(&self, data: &mut Data) {
        data.write_str_bit7(self)
    }
}

impl WriteCore for &[u8] {
    #[inline]
    fn write_core(&self, data: &mut Data) {
        data.write_buff_bit7(self)
    }
}

macro_rules! make_write_core_bit7 {
    ($type:ty) => {
        impl WriteCore for Vec<$type> {
            #[inline]
            fn write_core(&self, data: &mut Data) {
                data.write_bit7(self.len() as u64);
                for i in self {
                    data.write_bit7(*i);
                }
            }
        }
    };
}

make_write_core_bit7!(i16);
make_write_core_bit7!(i32);
make_write_core_bit7!(i64);
make_write_core_bit7!(u16);
make_write_core_bit7!(u32);
make_write_core_bit7!(u64);

macro_rules! make_write_core {
    ($type:ty) => {
        impl WriteCore for Vec<$type> {
            #[inline]
            fn write_core(&self, data: &mut Data) {
                data.write_bit7(self.len() as u64);
                for i in self {
                    data.write_to_le(*i);
                }
            }
        }
    };
}

make_write_core!(f32);
make_write_core!(f64);
make_write_core!(i128);
make_write_core!(u128);

impl<T: WriteCore> WriteCore for Vec<T> {
    #[inline]
    fn write_core(&self, data: &mut Data) {
        data.write_bit7(self.len() as u64);
        for i in self.iter() {
            data.write_core(i);
        }
    }
}

impl Data {
    #[inline]
    pub fn read_core<T: ReadCore>(&mut self) -> T {
        T::read_core(self)
    }
}

pub trait ReadCore {
    fn read_core(data: &mut Data) -> Self;
}

macro_rules! make_read_core_bit7 {
    ($type:ty) => {
        impl ReadCore for Vec<$type> {
            #[inline]
            fn read_core(data: &mut Data) -> Self {
                let (_, len) = data.get_bit7::<u64>().unwrap();
                let mut ret = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    let (_, v) = data.get_bit7::<$type>().unwrap();
                    ret.push(v);
                }
                ret
            }
        }
    };
}

make_read_core_bit7!(i16);
make_read_core_bit7!(i32);
make_read_core_bit7!(i64);
make_read_core_bit7!(u16);
make_read_core_bit7!(u32);
make_read_core_bit7!(u64);

macro_rules! make_read_core_le {
    ($type:ty) => {
        impl ReadCore for Vec<$type> {
            #[inline]
            fn read_core(data: &mut Data) -> Self {
                let (_, len) = data.get_bit7::<u64>().unwrap();
                let mut ret = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    ret.push(data.get_le::<$type>());
                }
                ret
            }
        }
    };
}

make_read_core_le!(i128);
make_read_core_le!(u128);
make_read_core_le!(f32);
make_read_core_le!(f64);

impl ReadCore for String {
    #[inline]
    fn read_core(data: &mut Data) -> Self {
        data.get_str_bit7().unwrap()
    }
}

impl<T: ReadCore> ReadCore for Vec<T> {
    #[inline]
    fn read_core(data: &mut Data) -> Self {
        let (_, len) = data.get_bit7::<u64>().unwrap();
        let mut ret: Vec<T> = Vec::with_capacity(len as usize);
        for _ in 0..len {
            ret.push(T::read_core(data));
        }
        ret
    }
}
