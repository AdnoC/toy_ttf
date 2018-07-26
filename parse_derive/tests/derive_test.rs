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

#[derive(Parse)]
struct O<'a, T: Parse<'a>>(T, Other<'a>, T);


impl<'a> Parse<'a> for &'a [u8] {
    fn file_size(&self) -> usize {
        self.len()
    }
    fn parse(buf: &'a [u8]) -> (&'a [u8], &'a [u8]) {
        (buf, buf)
    }
}

#[test]
fn derive_works() {
    let t = Thing { a: 4, b: 1, c: 1 };
    assert_eq!(t.file_size(), 2 + 2 + 2);

    let buf: &[u8] = &[0, 5, 0, 6, 0, 12];
    let t = Thing::parse(buf).1;
    assert_eq!(t, Thing { a: 5, b: 6, c: 12 });

    let mess = "Hello, World!";
    let sized_buf: &[u8] = &[0, mess.len() as u8, 2];
    let buf: Vec<u8> = sized_buf.into_iter().cloned().chain(mess.bytes()).collect();
    let (remain, o) = Other::parse(&buf);

    use std::str;

    assert_eq!(o.count as usize, mess.len());
    assert_eq!(o.some_id, 2);
    assert_eq!(str::from_utf8(o.data), Ok(mess));
    assert_eq!(remain.len(), 0);
}
