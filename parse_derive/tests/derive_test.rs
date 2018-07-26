#[macro_use]
extern crate parse_derive;

trait ArrayBuffer<'a, T: Parse> {
    fn new(start: &'a [u8], len: usize) -> (&'a [u8], Self);
}
trait Parse {
    /// Size of the object when serialized in the file
    fn file_size(&self) -> usize;
    fn parse(buf: &[u8]) -> (&[u8], Self);
}

impl Parse for u16 {
    fn file_size(&self) -> usize { 2 }
    fn parse(buf: &[u8]) -> (&[u8], Self) {
        let val: u16 = ((buf[0] as u16) << 8) | ((buf[1] as u16));
        (&buf[2..], val)
    }
}
impl Parse for u8 {
    fn file_size(&self) -> usize { 1 }
    fn parse(buf: &[u8]) -> (&[u8], Self) {
        (&buf[1..], buf[0])
    }
}

#[derive(Debug, PartialEq, Eq, Default, Parse)]
struct Thing {
    a: u16,
    b: u16,
    c: u16
}
// #[derive(Debug, PartialEq, Eq, Default, Parse)]
// struct Other<'a> {
//     count: u16,
//     some_id: u8,
//     #[arr_len_src = "count"]
//     data: Something<'a>,
// }

// #[derive(Debug)]
// struct Something<'a>(&'a [u8]);
// impl<'a> ArrayBuffer<'a, u16> for Something<'a> {
//     fn new(start: &'a [u8], len: usize) -> (&'a [u8], Self) {
//         (&start[len..], Something(&start[0..len]))
//     }
// }

#[test]
fn derive_works() {
    let t = Thing {
        a: 4,
        b: 1,
        c: 1,
    };
    assert_eq!(t.file_size(), 2 + 2 + 2);

    let buf: &[u8] = &[0, 5, 0, 6, 0, 12];
    let t = Thing::parse(buf).1;
    assert_eq!(t, Thing { a: 5, b: 6, c: 12 });
}
