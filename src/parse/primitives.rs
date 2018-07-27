use byteorder::{BigEndian, ByteOrder};
use parse::Parse;

pub type ShortFrac = i16;
pub type FWord = i16;
pub type UFWord = u16;
pub type F2Dot14 = i16;
pub type LongDateTime = i64;

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
