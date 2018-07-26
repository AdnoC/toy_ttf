#[macro_use]
extern crate parse_derive;

trait Parse<'a> {
    /// Size of the object when serialized in the file
    fn file_size(&self) -> usize;
    fn parse(buf: &'a [u8]) -> (&'a [u8], Self);
}

impl<'a> Parse<'a> for u16 {
    fn file_size(&self) -> usize {
        2
    }
    fn parse(buf: &[u8]) -> (&[u8], Self) {
        let val: u16 = ((buf[0] as u16) << 8) | (buf[1] as u16);
        (&buf[2..], val)
    }
}
impl<'a> Parse<'a> for u8 {
    fn file_size(&self) -> usize {
        1
    }
    fn parse(buf: &[u8]) -> (&[u8], Self) {
        (&buf[1..], buf[0])
    }
}

#[derive(Debug, PartialEq, Eq, Default, Parse)]
struct Thing {
    a: u16,
    b: u16,
    c: u16,
}
#[derive(Debug, Parse)]
struct Other<'o> {
    count: u16,
    some_id: u8,
    #[arr_len_src = "count"]
    data: &'o [u8],
}
// #[derive(Parse)]
// struct O<'a>(Other<'a>);

// impl<'o> Parse<'o> for Other<'o> {
//     fn file_size(&self) -> usize {
//         match *self {
//             Other {
//                 count: ref __binding_0,
//                 some_id: ref __binding_1,
//                 data: ref __binding_2,
//             } => 0 + __binding_0.file_size() + __binding_1.file_size() + __binding_2.file_size(),
//         }
//     }
//     fn parse(buf: &'o [u8]) -> (&'o [u8], Self) {
//         let res = <u16 as Parse>::parse(buf);
//         let buf = res.0;
//         let count = res.1;
//         let res = <u8 as Parse>::parse(buf);
//         let buf = res.0;
//         let some_id = res.1;
//         let res = {
//             let len = count as usize;
//             let buf = &buf[0..len];
//             let res = <&'o [u8] as Parse>::parse(buf);
//             (&buf[len..], res.1)
//         };
//         let buf = res.0;
//         let data = res.1;
//         let val = Other {
//             count: count,
//             some_id: some_id,
//             data: data,
//         };
//         (buf, val)
//     }
// }
// trait ArrayBuffer<'a, T: Parse> {
//     fn new(start: &'a [u8], len: usize) -> (&'a [u8], Self);
//     fn len(&self) -> usize;
// }
// impl<'a, T: Parse, AB: ArrayBuffer<'a, T>> Parse for AB {
//     fn file_size(&self) -> usize {
//         self.len()
//     }
//     fn parse(buf: &[u8]) -> (&[u8], Self) {
//         unimplemented!()
//     }
// }

// #[derive(Debug)]
// struct Something<'a>(&'a [u8]);
// impl<'a> Parse for Something<'a> {
//     fn file_size(&self) -> usize { self.0.len() }
//     fn parse(buf: &[u8]) -> (&[u8], Something<'a>) {
//         let b1 = buf;
//         let b2 = buf;
//         (b1, Something(b2))
//         // (buf, Something(buf))
//     }
// }

impl<'a> Parse<'a> for &'a [u8] {
    fn file_size(&self) -> usize {
        self.len()
    }
    fn parse(buf: &'a [u8]) -> (&'a [u8], &'a [u8]) {
        (buf, buf)
    }
}
// impl<'a> ArrayBuffer<'a, u16> for Something<'a> {
//     fn new(start: &'a [u8], len: usize) -> (&'a [u8], Self) {
//         (&start[len..], Something(&start[0..len]))
//     }
//     fn len(&self) -> usize { self.0.len() }
// }

#[test]
fn derive_works() {
    let t = Thing { a: 4, b: 1, c: 1 };
    assert_eq!(t.file_size(), 2 + 2 + 2);

    let buf: &[u8] = &[0, 5, 0, 6, 0, 12];
    let t = Thing::parse(buf).1;
    assert_eq!(t, Thing { a: 5, b: 6, c: 12 });
}
