use byteorder::{BigEndian, ByteOrder};
use parse::Parse;
use std::marker::PhantomData;
use std::ops::{Add, Sub, Mul};

use font::Font;
macro_rules! newtype_unit_wrapper {
    ($(#[$attr:meta])* unit $name:ident) => {
        newtype_unit_wrapper!($(#[$attr])* () unit $name);
    };
    ($(#[$attr:meta])* pub unit $name:ident) => {
        newtype_unit_wrapper!($(#[$attr])* (pub) unit $name);
    };
    ($(#[$attr:meta])* ($($vis:tt)*) unit $name:ident) => {
        $(#[$attr])*
        #[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
        $($vis)* struct $name<T>(pub T);
        impl<'a, T: Parse<'a>> Parse<'a> for $name<T> {
            fn approx_file_size() -> usize {
                T::approx_file_size()
            }
            fn parse(buf: &'a [u8]) -> (&'a [u8], Self) {
                let (buf, val) = T::parse(buf);
                (buf, $name(val))
            }
        }
        impl<T> From<T> for $name<T> {
            fn from(val: T) -> Self {
                $name(val)
            }
        }
        impl<T> $name<T> {
            pub fn map<U, F: Fn(T) -> U>(self, f: F) -> $name<U> {
                $name(f(self.0))
            }
        }

        impl<T: Add<T>> Add for $name<T> {
            type Output = $name<T::Output>;

            fn add(self, rhs: $name<T>) -> $name<T::Output> {
                $name(self.0 + rhs.0)
            }
        }

        impl<T: Sub<T>> Sub for $name<T> {
            type Output = $name<T::Output>;

            fn sub(self, rhs: $name<T>) -> $name<T::Output> {
                $name(self.0 - rhs.0)
            }
        }

        impl<T: Mul<T>> Mul for $name<T> {
            type Output = $name<T::Output>;

            fn mul(self, rhs: $name<T>) -> $name<T::Output> {
                $name(self.0 * rhs.0)
            }
        }
    }
}

newtype_unit_wrapper!(
    /// A quantity in Font Units
    pub unit FontUnit);

impl<T: Into<f32>> FontUnit<T> {
    fn funits_to_pixels_rat<'a>(units_per_em: u16, point_size: usize) -> f32 {
        let resolution = 72.; // dpi
        (point_size as f32 * resolution) / (72. * units_per_em as f32)
    }

    pub fn to_pixels<'a>(self, units_per_em: u16, point_size: usize) -> f32 {
        let units: f32 = self.0.into();
        units * Self::funits_to_pixels_rat(units_per_em, point_size)
    }
    pub fn to_pixels_using_font<'a>(self, font: &Font<'a>, point_size: usize) -> f32 {
        use font::GetTable;
        use tables::head::Head;

        let head: Head = font.get_table().unwrap();

        self.to_pixels(head.units_per_em, point_size)
    }
}

newtype_unit_wrapper!(
    /// Wrapper for values in `em`
    pub unit Em);
newtype_unit_wrapper!(
    /// One-twentieth of a point (1440 per inch)
    pub unit TWIP);

pub type ShortFrac = i16;
pub type FWord = FontUnit<i16>;
pub type UFWord = FontUnit<u16>;
pub type LongDateTime = i64;

#[derive(Debug, PartialEq, PartialOrd)]
pub struct F2Dot14(pub f32);
impl<'a> Parse<'a> for F2Dot14 {
    fn approx_file_size() -> usize {
        i16::approx_file_size()
    }
    fn parse(buf: &[u8]) -> (&[u8], F2Dot14) {
        let (buf, num) = i16::parse(buf);
        // (1 << 14)  = 16384
        let frac_val = (num as f32) / ((1 << 14) as f32);
        (buf, F2Dot14(frac_val))
    }
}


// Represents the number (self.0).(self.1)
// e.g. 0.5 is (0x0000).(0x5000)
#[derive(Debug, Parse, PartialEq, Eq, PartialOrd, Ord)]
pub struct Fixed(pub i16, pub i16);

// impl<'a> Parse<'a> for &'a [u8] {
//     fn approx_file_size() -> usize {
//         0
//     }
//     fn parse(buf: &'a [u8]) -> (&'a [u8], &'a [u8]) {
//         (buf, buf)
//     }
// }


/// Type to be used when you have something that implements `Parse`
/// but doesn't use a lifetime. Just add a member of type `PhantomLifetime`.
pub type PhantomLifetime<'a> = PhantomData<&'a ()>;
impl<'a> Parse<'a> for PhantomLifetime<'a> {
    fn approx_file_size() -> usize {
        0
    }
    fn parse(buf: &[u8]) -> (&[u8], PhantomLifetime) {
        (buf, PhantomData)
    }
}

macro_rules! impl_primitives {
    ($($prim:ty : $parser:expr),*) => {
        $(
            impl<'a> Parse<'a> for $prim {
                fn approx_file_size() -> usize {
                    use std::mem::size_of;
                    size_of::<$prim>()
                }

                fn parse(buf: &'a [u8]) -> (&'a [u8], Self) {
                    use std::mem::size_of;
                    let len = size_of::<$prim>();
                    let val = $parser(buf);
                    (&buf[len..], val)
                }
            }
         )*
    }
}

impl_primitives! {
    u8: |buf: &[u8]| buf[0],
    u16: BigEndian::read_u16,
    u32: BigEndian::read_u32,
    u64: BigEndian::read_u64,

    i8: |buf: &[u8]| buf[0] as i8,
    i16: BigEndian::read_i16,
    i32: BigEndian::read_i32,
    i64: BigEndian::read_i64
}

macro_rules! impl_parse_arr {
    ($([$prim:ty; $len:expr]),*) => {
        $(
            impl<'a> Parse<'a> for [$prim; $len] {
                fn approx_file_size() -> usize {
                    $len * <$prim as Parse>::approx_file_size()
                }

                fn parse(mut buf: &'a [u8]) -> (&'a [u8], Self) {
                    let mut arr = [0; $len];
                    for i in 0..$len {
                        let res = <$prim as Parse>::parse(buf);
                        buf = res.0;
                        arr[i] = res.1;
                    }
                    (buf, arr)
                }
            }
         )*
    }
}

impl_parse_arr! {
    [u8; 4],
    [u8; 10],
    [u32; 4]
}

macro_rules! derive_parse_from_primitive {
    ($type:ty, $prim:ty, $parser:expr) => {
        impl<'a> Parse<'a> for $type {
            fn approx_file_size() -> usize {
                <$prim as Parse>::approx_file_size()
            }

            fn parse(buf: &'a [u8]) -> (&'a [u8], Self) {
                use num_traits::FromPrimitive;
                let (buf, prim_val) = <$prim as Parse>::parse(buf);
                let val = $parser(prim_val).unwrap();
                (buf, val)
            }
        }
    };
    ($type:ty,i16) => {
        derive_parse_from_primitive!($type, i16, <$type as FromPrimitive>::from_i16);
    };
}
