use std::{path::Path, fs::File, io::Read};

mod byte_stream;
mod bytecode;

fn main() {
    let path = Path::new("Main.class");

    let mut file = File::open(path).unwrap();
    let mut buf = Vec::with_capacity(1024);

    file.read_to_end(&mut buf).unwrap();

    let mut stream = byte_stream::ByteStream::new(&buf);
    let file = bytecode::ClassFile::read(&mut stream);
    
    println!("{:#?}", file);
}
