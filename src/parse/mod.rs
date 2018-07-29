mod name;
#[macro_use]
pub mod primitives;
pub(crate) mod font_directory;

use std::fmt;

pub trait Parse<'a> {
    /// Size of the object when serialized in the file
    fn approx_file_size() -> usize;
    fn parse(buf: &'a [u8]) -> (&'a [u8], Self);
}

/// Should be considered as just a view of the file starting at some point.
/// Does not have a defined endpoint, could go until the end of the file
/// Typed just so that the `at` function is convenient
pub(crate) struct BufView<'a, T>(pub &'a [u8], pub ::std::marker::PhantomData<T>);
impl<'a, T: Parse<'a>> Parse<'a> for BufView<'a, T> {
    fn approx_file_size() -> usize {
        0 // Just captures a view the whole buffer as it was passed in
    }
    fn parse(buf: &'a [u8]) -> (&'a [u8], Self) {
        use std::marker::PhantomData;
        (buf, BufView(buf, PhantomData))
    }
}
impl<'a, T: Parse<'a>> BufView<'a, T> {
    pub fn at(&self, idx: usize) -> T {
        let sized_idx = idx * T::approx_file_size();
        T::parse(&self.0[sized_idx..]).1
    }
    pub fn split_at(&self, idx: usize) -> (BufView<'a, T>, BufView<'a, T>) {
        use std::marker::PhantomData;
        let sized_idx = idx * T::approx_file_size();
        let (before, after) = self.0.split_at(sized_idx);
        (BufView(before, PhantomData), BufView(after, PhantomData))
    }
    pub fn cast<U>(&self) -> BufView<'a, U> {
        use std::marker::PhantomData;
        BufView(self.0, PhantomData)
    }
}
impl<'a, T: Parse<'a>> fmt::Debug for BufView<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let typed_len = self.0.len() / T::approx_file_size();
        f.debug_struct("BufView")
            .field("typed_len", &typed_len)
            .field("len", &self.0.len())
            .finish()
    }
}

/// Has a defined length.
/// It is known that the buffer it contains is made up of `sized_len` `T`s
pub(crate) struct DynArr<'a, T>(pub &'a [u8], pub ::std::marker::PhantomData<T>);
impl<'a, T: Parse<'a>> DynArr<'a, T> {
    pub fn at(&self, idx: usize) -> T {
        let sized_idx = idx * T::approx_file_size();
        T::parse(&self.0[sized_idx..]).1
    }
    pub fn iter(&self) -> DynArr<'a, T> {
        self.clone()
    }
    pub fn len(&self) -> usize {
        self.0.len() / T::approx_file_size()
    }
}
impl<'a, T: Parse<'a>> Parse<'a> for DynArr<'a, T> {
    fn approx_file_size() -> usize {
        T::approx_file_size()
    }
    fn parse(buf: &'a [u8]) -> (&'a [u8], Self) {
        use std::marker::PhantomData;
        assert!(buf.len() % <T as Parse>::approx_file_size() == 0);
        (buf, DynArr(buf, PhantomData))
    }
}
impl<'a, T: Parse<'a>> Iterator for DynArr<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let size: usize = T::approx_file_size();
        if self.0.len() < size {
            return None;
        }

        let (buf, val) = T::parse(self.0);
        self.0 = buf;
        Some(val)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.0.len(), Some(self.0.len()))
    }
}
impl<'a, T: Parse<'a>> ::std::iter::ExactSizeIterator for DynArr<'a, T> {}
impl<'a, T: Parse<'a>> ::std::iter::DoubleEndedIterator for DynArr<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let size: usize = T::approx_file_size();
        if self.0.len() < size {
            return None;
        }

        let start_point = self.0.len() - size;
        let (_, val) = T::parse(&self.0[start_point..]);
        self.0 = &self.0[..start_point];
        Some(val)
    }
}
impl<'a, T> Clone for DynArr<'a, T> {
    fn clone(&self) -> Self {
        DynArr(self.0.clone(), self.1.clone())
    }
}
impl<'a, T: Parse<'a>> fmt::Debug for DynArr<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let typed_len = self.0.len() / T::approx_file_size();
        f.debug_struct("DynArr")
            .field("typed_len", &typed_len)
            .field("len", &self.0.len())
            .finish()
    }
}

// TODO: Use to verify font tables
#[allow(dead_code)]
fn table_check_sum(table: &[u32]) -> u32 {
    table.iter().sum()
    // C version
    // uint32 CalcTableChecksum(uint32 *table, uint32 numberOfBytesInTable) {
    //     uint32 sum = 0;
    //     uint32 nLongs = (numberOfBytesInTable + 3) / 4;
    //     while (nLongs-- > 0)
    //         sum += *table++;
    //     return sum;
    // }
}

pub fn load_font(font_buf: &[u8]) {
    {
        // TESTING
        test_parse(font_buf).expect("Test parse failed");

        // let font_buf = &font_buf[0..1024];

        // let fd = parse_font_directory(font_buf).unwrap();
        // println!("fd = {:#?}", fd);
        // let off = parse_offset_subtable(font_buf);
        // let st = off.and_then(|inp| table_directory_entry(inp.0));

        // let st = parse_font_directory(font_buf);
        // match st {
        //     Ok(val) => println!("ok {:#?}", val.1),
        //     Err(e) => println!("err {:?}", e),
        // };

        // FIXME: WILL BLOW UP, just for debug
        // tables::tables_parse::parse_name_table(font_buf);
        // tables::tables_parse::parse_name_record(font_buf, 0);
    }
}

fn test_parse(i: &[u8]) -> ::nom::IResult<&[u8], ()> {
    unimplemented!()
    //
    // use tables::TableTag;
    //
    // use nom::Offset;
    // let (i1, fd) = try_parse!(i, font_directory::parse_font_directory);
    // let eaten = i.offset(i1);
    // let name_offset = fd.table_dirs.0.iter()
    //     .find(|tdr| tdr.tag == TableTag::Name)
    //     .map(|tdr| tdr.offset)
    //     .expect("Coulnd't find name table");
    //
    // let name_offset = name_offset - eaten as u32;
    // let (i_name, _) = try_parse!(i1, take!(name_offset));
    // let (i_fin, nt) = try_parse!(i_name, name::parse_name_table);
    //
    // println!("name_table = {:#?}", nt);
    //
    // Ok((i_fin, ()))
}
