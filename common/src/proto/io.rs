use std::borrow::Cow;
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::iter;
use std::mem::MaybeUninit;

use smallvec::SmallVec;

/// Binary encoding result
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Binary encoding error
#[derive(Debug, Clone, PartialEq)]
pub struct Error(pub Cow<'static, str>);
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error decoding data: {}", self.0)
    }
}
impl std::error::Error for Error {}

fn error(err: impl Into<Cow<'static, str>>) -> Error {
    Error(err.into())
}

/// Binary encoding type
pub trait ProtoType: Sized {
    /// The derived checksum of the encoding structure
    const CHECKSUM: u128;
}

/// Binary encoding with read support
pub trait BinRead: ProtoType {
    /// Reads a value with binary encoding
    fn read(buf: &mut &[u8]) -> Result<Self>;
}

/// Binary encoding with write support
pub trait BinWrite: ProtoType {
    /// Writes a value with binary encoding
    fn write(&self, buf: &mut Vec<u8>);

    /// Returns the binary encoding of the value in a new Vec.
    fn write_to_vec(&self) -> Vec<u8> {
        let mut vec = Vec::new();
        self.write(&mut vec);
        vec
    }
}

macro_rules! primitive {
    ($ty:ty, $len:literal, $mask:literal) => {
        impl ProtoType for $ty {
            const CHECKSUM: u128 = $len | $mask;
        }

        impl BinRead for $ty {
            #[inline]
            fn read(buf: &mut &[u8]) -> Result<Self> {
                let slice = match buf.get(..$len) {
                    Some(slice) => slice,
                    None => return Err(error("buffer underflow")),
                };
                *buf = unsafe { &buf.get_unchecked($len..) };
                let array = <[u8; $len]>::try_from(slice).expect("..$len");
                let ret = <$ty>::from_le_bytes(array);
                Ok(ret)
            }
        }

        impl BinWrite for $ty {
            #[inline]
            fn write(&self, buf: &mut Vec<u8>) {
                let slice = &self.to_le_bytes()[..];
                buf.extend(slice);
            }
        }
    };
}

primitive!(i8, 1, 32);
primitive!(i16, 2, 32);
primitive!(i32, 4, 32);
primitive!(i64, 8, 32);
primitive!(i128, 16, 32);
primitive!(u8, 1, 0);
primitive!(u16, 2, 0);
primitive!(u32, 4, 0);
primitive!(u64, 8, 0);
primitive!(u128, 16, 0);
primitive!(f32, 4, 512);
primitive!(f64, 8, 512);

impl ProtoType for bool {
    const CHECKSUM: u128 = 64;
}

impl BinRead for bool {
    #[inline]
    fn read(buf: &mut &[u8]) -> Result<Self> {
        match <u8 as BinRead>::read(buf) {
            Ok(0) => Ok(false),
            Ok(1) => Ok(true),
            _ => Err(error("invalid boolean value")),
        }
    }
}

impl BinWrite for bool {
    #[inline]
    fn write(&self, buf: &mut Vec<u8>) {
        let byte = match self {
            true => 1_u8,
            false => 0_u8,
        };
        <u8 as BinWrite>::write(&byte, buf);
    }
}

impl<T: ProtoType> ProtoType for Box<T> {
    const CHECKSUM: u128 = <T as ProtoType>::CHECKSUM;
}

impl<T: BinRead> BinRead for Box<T> {
    #[inline]
    fn read(buf: &mut &[u8]) -> Result<Self> {
        <T as BinRead>::read(buf).map(Box::new)
    }
}

impl<T: BinWrite> BinWrite for Box<T> {
    #[inline]
    fn write(&self, buf: &mut Vec<u8>) {
        <T as BinWrite>::write(&**self, &mut *buf)
    }
}

impl<T: ProtoType> ProtoType for Vec<T> {
    const CHECKSUM: u128 = <T as ProtoType>::CHECKSUM | 128;
}

impl<T: BinRead> BinRead for Vec<T> {
    #[inline(always)]
    fn read(buf: &mut &[u8]) -> Result<Self> {
        read_slice(buf, Vec::<T>::with_capacity)
    }
}

#[inline(always)]
fn read_slice<T: BinRead, C: Extend<T>>(buf: &mut &[u8], create: fn(usize) -> C) -> Result<C> {
    let count = <u32 as BinRead>::read(&mut *buf)? as usize;
    if count > buf.len() {
        return Err(error("nonsense vec length detected"));
    }
    let mut coll = create(count);
    for _ in 0..count {
        coll.extend(iter::once(<T as BinRead>::read(&mut *buf)?));
    }
    Ok(coll)
}

impl<T: BinWrite> BinWrite for Vec<T> {
    #[inline(always)]
    fn write(&self, buf: &mut Vec<u8>) {
        write_slice(self.as_slice(), buf)
    }
}

#[inline(always)]
fn write_slice<T: BinWrite>(slice: &[T], buf: &mut Vec<u8>) {
    let len = u32::try_from(slice.len()).expect("Slice is unrealistically long");
    <u32 as BinWrite>::write(&len, &mut *buf);
    for elem in slice {
        <T as BinWrite>::write(elem, &mut *buf);
    }
}

impl ProtoType for String {
    const CHECKSUM: u128 = 65;
}

impl BinRead for String {
    #[inline]
    fn read(buf: &mut &[u8]) -> Result<Self> {
        let vec = <Vec<u8> as BinRead>::read(buf)?;
        String::from_utf8(vec).map_err(|_| error("UTF-8 decode error"))
    }
}

impl BinWrite for String {
    #[inline]
    fn write(&self, buf: &mut Vec<u8>) {
        let len = u32::try_from(self.len()).expect("String is unrealistically long");
        <u32 as BinWrite>::write(&len, &mut *buf);
        buf.extend(self.as_bytes());
    }
}

impl<T: ProtoType> ProtoType for Option<T> {
    const CHECKSUM: u128 = 65;
}

impl<T: BinRead> BinRead for Option<T> {
    #[inline]
    fn read(buf: &mut &[u8]) -> Result<Self> {
        let has_some = <bool as BinRead>::read(&mut *buf)?;
        if has_some {
            Ok(Some(<T as BinRead>::read(buf)?))
        } else {
            Ok(None)
        }
    }
}

impl<T: BinWrite> BinWrite for Option<T> {
    #[inline]
    fn write(&self, buf: &mut Vec<u8>) {
        <bool as BinWrite>::write(&self.is_some(), &mut *buf);
        if let Some(value) = self.as_ref() {
            <T as BinWrite>::write(value, &mut *buf);
        }
    }
}

macro_rules! tuple {
    ($($names:ident $num:tt),*) => {
        impl<$($names: ProtoType),*> ProtoType for ($($names,)*) {
            const CHECKSUM: u128 = {
                let mut output = 128_u128;
                output = output.wrapping_mul(crate::proto::PROTO_TYPE_CKSUM_PRIME);
                $(
                    output = output.wrapping_add(<$names as ProtoType>::CHECKSUM);
                    output = output.wrapping_mul(crate::proto::PROTO_TYPE_CKSUM_PRIME);
                )*
                output
            };
        }

        impl<$($names: BinRead),*> BinRead for ($($names,)*) {
            #[inline]
            fn read(buf: &mut &[u8]) -> Result<Self> {
                Ok(($(
                    <$names as BinRead>::read(&mut *buf)?,
                )*))
            }
        }

        impl<$($names: BinWrite),*> BinWrite for ($($names,)*) {
    #[inline]
            fn write(&self, buf: &mut Vec<u8>) {
                $(
                    <$names as BinWrite>::write(&(self.$num), &mut *buf);
                )*
            }
        }
    }
}

tuple!(T0 0);
tuple!(T0 0, T1 1);
tuple!(T0 0, T1 1, T2 2);
tuple!(T0 0, T1 1, T2 2, T3 3);
tuple!(T0 0, T1 1, T2 2, T3 3, T4 4);
tuple!(T0 0, T1 1, T2 2, T3 3, T4 4, T5 5);
tuple!(T0 0, T1 1, T2 2, T3 3, T4 4, T5 5, T6 6);
tuple!(T0 0, T1 1, T2 2, T3 3, T4 4, T5 5, T6 6, T7 7);

macro_rules! array {
    ($count:literal) => {
        impl<T: ProtoType> ProtoType for [T; $count] {
            const CHECKSUM: u128 = {
                let mut output = 256_u128;
                output = output.wrapping_mul(crate::proto::PROTO_TYPE_CKSUM_PRIME);
                let mut cnt = $count;
                while cnt > 0 {
                    cnt -= 1;
                    output = output.wrapping_add(<T as ProtoType>::CHECKSUM);
                    output = output.wrapping_mul(crate::proto::PROTO_TYPE_CKSUM_PRIME);
                }
                output
            };
        }

        impl<T: Copy + BinRead> BinRead for [T; $count] {
            fn read(buf: &mut &[u8]) -> Result<Self> {
                // Safety: they shall have the same size
                union Hack<T: Copy> {
                    uninit: [MaybeUninit<T>; $count],
                    init: [T; $count],
                }

                // Safety: see https://doc.rust-lang.org/core/mem/union.MaybeUninit.html#initializing-an-array-element-by-element
                let array = unsafe {
                    let mut hack = Hack {
                        uninit: MaybeUninit::uninit().assume_init(),
                    };
                    for elem in &mut hack.uninit[..] {
                        let value = <T as BinRead>::read(&mut *buf)?;
                        *elem = MaybeUninit::new(value);
                    }
                    hack.init
                };

                // all elements have been initialized
                Ok(array)
            }
        }

        impl<T: BinWrite> BinWrite for [T; $count] {
            #[allow(clippy::indexing_slicing)]
            fn write(&self, buf: &mut Vec<u8>) {
                for i in 0..$count {
                    // Safety: 0 <= i < $count, self.len() == $count
                    let t = unsafe { self.get_unchecked(i) };
                    <T as BinWrite>::write(t, &mut *buf);
                }
            }
        }
    };
}

array!(1);
array!(2);
array!(3);
array!(4);
array!(5);
array!(6);
array!(7);
array!(8);
array!(9);
array!(12);
array!(16);
array!(18);
array!(24);
array!(32);
array!(36);
array!(48);
array!(64);
array!(128);
array!(256);
array!(512);
array!(1024);
array!(2048);
array!(4096);

impl ProtoType for crate::space::Vector {
    const CHECKSUM: u128 = {
        #[derive(codegen::Gen)]
        struct _Vector([f64; 3]);
        _Vector::CHECKSUM
    };
}

impl BinRead for crate::space::Vector {
    fn read(buf: &mut &[u8]) -> Result<Self> {
        let inner = <[f64; 3]>::read(buf)?;
        Ok(Self::from_column_slice(&inner[..]))
    }
}

impl BinWrite for crate::space::Vector {
    fn write(&self, vec: &mut Vec<u8>) {
        let slice = self.as_slice();
        let array: [f64; 3] = slice.try_into().expect("Vector has exactly 3 elements");
        array.write(vec);
    }
}

impl ProtoType for crate::space::Matrix {
    const CHECKSUM: u128 = {
        #[derive(codegen::Gen)]
        struct _Matrix([f64; 16]);
        _Matrix::CHECKSUM
    };
}

impl BinRead for crate::space::Matrix {
    fn read(buf: &mut &[u8]) -> Result<Self> {
        let inner = <[f64; 16]>::read(buf)?;
        Ok(Self::from_column_slice(&inner[..]))
    }
}

impl BinWrite for crate::space::Matrix {
    fn write(&self, vec: &mut Vec<u8>) {
        let slice = self.as_slice();
        let array: [f64; 16] = slice.try_into().expect("Matrix has exactly 16 elements");
        array.write(vec);
    }
}

impl<A: smallvec::Array> ProtoType for SmallVec<A>
where
    A::Item: ProtoType,
{
    const CHECKSUM: u128 = Vec::<A::Item>::CHECKSUM;
}

impl<A: smallvec::Array> BinRead for SmallVec<A>
where
    A::Item: BinRead,
{
    #[inline(always)]
    fn read(buf: &mut &[u8]) -> Result<Self> {
        read_slice(buf, SmallVec::<A>::with_capacity)
    }
}

impl<A: smallvec::Array> BinWrite for SmallVec<A>
where
    A::Item: BinWrite,
{
    #[inline(always)]
    fn write(&self, vec: &mut Vec<u8>) {
        write_slice(self.as_slice(), vec)
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use super::*;

    fn test_write(t: &(impl Debug + BinWrite), expect: &[u8]) {
        let mut actual = Vec::new();
        t.write(&mut actual);
        assert_eq!(&actual[..], expect);
    }

    fn test_read<T: Debug + PartialEq + BinRead>(expect: &T, mut buf: &[u8]) {
        let actual = T::read(&mut buf);
        assert_eq!(actual.as_ref(), Ok(expect));
        assert_eq!(buf.len(), 0, "buf is not read fully");
    }

    macro_rules! test_rw {
        ($read:ident, $write:ident: $expr:expr, $bytes:expr) => {
            #[test]
            pub fn $read() {
                test_read(&$expr, $bytes);
            }

            #[test]
            pub fn $write() {
                test_write(&$expr, $bytes);
            }
        };
    }

    test_rw!(test_read_u8, test_write_u8: 0xa0_u8, b"\xa0");
    test_rw!(test_read_i8, test_write_i8: -0x60_i8, b"\xa0");
    test_rw!(test_read_u16, test_write_u16: 0xa0c0_u16, b"\xc0\xa0");
    test_rw!(test_read_i16, test_write_i16: -0x5f40_i16, b"\xc0\xa0");
    test_rw!(test_read_u32, test_write_u32: 0xa0c06080_u32, b"\x80\x60\xc0\xa0");
    test_rw!(test_read_i32, test_write_i32: -0x5f3f9f80_i32, b"\x80\x60\xc0\xa0");
    test_rw!(test_read_u64, test_write_u64: 0xabcdef1234567890_u64, b"\x90\x78\x56\x34\x12\xef\xcd\xab");
    test_rw!(test_read_i64, test_write_i64: -0x543210edcba98770_i64, b"\x90\x78\x56\x34\x12\xef\xcd\xab");
    test_rw!(test_read_u128, test_write_u128: 0xabcdef1234567890abcdef1234567890_u128, b"\x90\x78\x56\x34\x12\xef\xcd\xab\x90\x78\x56\x34\x12\xef\xcd\xab");
    test_rw!(test_read_i128, test_write_i128: -0x543210edcba98770543210edcba98770_i128, b"\x90\x78\x56\x34\x12\xef\xcd\xab\x8f\x78\x56\x34\x12\xef\xcd\xab");

    test_rw!(test_read_f32, test_write_f32: 0.0_f32, &(0.0_f32).to_le_bytes());
    test_rw!(test_read_f64, test_write_f64: 0.0_f64, &(0.0_f64).to_le_bytes());
    test_rw!(
        test_read_f64_inf,
        test_write_f64_inf: f64::INFINITY,
        &(f64::INFINITY).to_le_bytes()
    );

    test_rw!(test_read_bool_false, test_write_bool_false: false, b"\0");
    test_rw!(test_read_bool_true, test_write_bool_true: true, b"\x01");

    test_rw!(test_read_box_true, test_write_box_true: Box::new(true), b"\x01");
    test_rw!(
        test_read_vec_true,
        test_write_vec_true: vec![true],
        b"\x01\0\0\0\x01"
    );
    test_rw!(
        test_read_vec_empty,
        test_write_vec_empty: Vec::<bool>::new(),
        b"\0\0\0\0"
    );
    test_rw!(test_read_option_true, test_write_option_true: Some(true), b"\x01\x01");
    test_rw!(
        test_read_option_empty,
        test_write_option_empty: None::<bool>,
        b"\0"
    );
    test_rw!(test_read_string, test_write_string: String::from("Rust\u{2026}"), b"\x07\0\0\0Rust\xe2\x80\xa6");
}
