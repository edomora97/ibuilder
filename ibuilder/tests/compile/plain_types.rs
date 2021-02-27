use ibuilder::*;
use std::path::PathBuf;

#[derive(IBuilder)]
pub struct Foo {
    f_i8: i8,
    f_i16: i16,
    f_i32: i32,
    f_i64: i64,
    f_u8: u8,
    f_u16: u16,
    f_u32: u32,
    f_u64: u64,
    f_isize: isize,
    f_usize: usize,
    f_f32: f32,
    f_f64: f64,
    f_string: String,
    f_char: char,
    f_pathbuf: PathBuf,
    f_bool: bool,
}

fn main() {
    let _builder = Foo::builder();
}
