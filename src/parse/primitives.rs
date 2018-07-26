use parse::Parse;
use byteorder::{BigEndian, ByteOrder};

pub type ShortFrac = i16;
pub type Fixed = (i16, i16);
pub type FWord = i16;
pub type UFWord = u16;
pub type F2Dot14 = i16;
pub type LongDateTime = i64;

impl<'a> Parse<'a> for &'a [u8] {
    fn file_size(&self) -> usize {
        self.len()
    }
    fn parse(buf: &'a [u8]) -> (&'a [u8], &'a [u8]) {
        (buf, buf)
    }
}

macro_rules! impl_primitives {
    ($($prim:ty : $parser:expr),*) => {
        $(
            impl<'a> Parse<'a> for $prim {
                fn file_size(&self) -> usize {
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

