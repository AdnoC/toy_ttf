#[macro_use]
extern crate parse_derive;

trait Parse: Sized {
    /// Size of the object when serialized in the file
    fn file_size(&self) -> usize;
    // fn parse(buf: &[u8]) -> (Self, &[u8]);
}

impl Parse for u16 {
    fn file_size(&self) -> usize { 2 }
    // fn parse(buf: &[u8]) -> (Self, &[u8]) { (0, buf) }
}

#[derive(Default, Parse)]
struct Thing {
    a: u16,
    b: u16,
    #[arr_len_src = "a"]
    c: u16
}

#[test]
fn derive_works() {
    let t = Thing {
        a: 4,
        b: 1,
        c: 1,
    };
    assert_eq!(t.file_size(), 2 + 2 + 2*4);
}
