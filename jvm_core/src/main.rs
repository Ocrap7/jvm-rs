#![feature(concat_idents)]
use std::{fs::File, io::Read, path::Path};

use crate::rf::Rf;

mod byte_stream;
mod bytecode;
mod error;
mod frame;
mod instructions;
mod rf;
mod runtime;
mod thread;
mod value;

fn main() {
    let path = Path::new("examples/Main.class");

    let mut file = File::open(path).unwrap();
    let mut buf = Vec::with_capacity(1024);

    file.read_to_end(&mut buf).unwrap();

    let mut stream = byte_stream::ByteStream::new(&buf);
    let file = bytecode::ClassFile::read(&mut stream);

    println!("{:#?}", file);

    let runtime = Rf::new(runtime::Runtime::new(vec![file]));
    let (status, thread) = runtime::Runtime::start(runtime.clone(), "Test/Main");


    // println!("{:#?}", rt);
    // println!("{:?} {:#?}", status, thread);
}
