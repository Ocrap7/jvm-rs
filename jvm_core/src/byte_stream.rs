use crate::bytecode::ConstantPool;


pub struct ByteStream<'a> {
    pub index: usize,
    pub data: &'a [u8],
}

pub fn cast_type<T>(bytes: &[u8]) -> T {
    assert!(bytes.len() == std::mem::size_of::<T>());
    let ptr = bytes.as_ptr();

    unsafe { std::ptr::read_unaligned(ptr as *const T) }
    // unsafe { &*(ptr as *const T) }
}

pub fn cast_slice<T>(bytes: &[u8]) -> &[T] {
    let ptr = bytes.as_ptr();

    unsafe { std::slice::from_raw_parts(ptr as *const T, bytes.len() / std::mem::size_of::<T>()) }
}

pub fn cast_slice_unchecked<T>(bytes: &[u8]) -> &T {
    let ptr = bytes.as_ptr();

    unsafe { &*(ptr as *const T) }
}

impl<'a> ByteStream<'a> {
    pub fn new(data: &'a [u8]) -> ByteStream<'a> {
        ByteStream { index: 0, data }
    }

    pub fn reset(&mut self) {
        self.index = 0;
    }

    pub fn has_next(&self) -> bool {
        self.index < self.data.len()
    }

    pub fn current(&self) -> &[u8] {
        &self.data[self.index..]
    }

    pub fn read<B: StreamRead>(&mut self, ctx: &ReaderContext) -> B {
        B::read(self, ctx)
    }

    pub fn read_many<B: StreamRead>(&mut self, count: usize, ctx: &ReaderContext) -> Vec<B> {
        (0..count).map(|_| self.read(ctx)).collect()

        // let data = &self.data[self.index..self.index + count * B::SIZE];
        // let data_ptr = data.as_ptr();

        // let data_slice = unsafe { std::slice::from_raw_parts(data_ptr as *const B, count) };

        // self.index += count;

        // data_slice
    }

    pub fn read_vary<V: VaryingRead<'a>>(&mut self) -> V {
        let len = V::len(&self.data[self.index..]);

        println!("{}", len);
        let value = V::read(&self.data[self.index..self.index + len]);

        self.index += len;

        value
    }

    pub fn read_many_vary<B: VaryingRead<'a>>(&mut self, count: usize) -> Vec<B> {
        (0..count).inspect(|i| println!("Read MV {}", i)).map(|_| self.read_vary()).collect()
    }
}

pub trait ByteRead {
    /// Size in bytes;
    const SIZE: usize;

    fn read<'a>(buffer: &'a [u8]) -> Self;
}

impl ByteRead for u8 {
    const SIZE: usize = 1;

    fn read<'a>(buffer: &'a [u8]) -> Self {
        buffer[0]
    }
}

// impl <T: From<u8>> ByteRead for T {
//     const SIZE: usize = 1;

//     fn read<'a>(buffer: &'a [u8]) -> Self {
//         buffer[0].into()
//     }
// }

impl ByteRead for i8 {
    const SIZE: usize = 1;

    fn read<'a>(buffer: &'a [u8]) -> Self {
        i8::from_be_bytes([buffer[0]])
    }
}

impl ByteRead for u16 {
    const SIZE: usize = 2;

    fn read<'a>(buffer: &'a [u8]) -> Self {
        u16::from_be_bytes(buffer[0..2].try_into().unwrap())
    }
}

impl ByteRead for i16 {
    const SIZE: usize = 2;

    fn read<'a>(buffer: &'a [u8]) -> Self {
        i16::from_be_bytes(buffer[0..2].try_into().unwrap())
    }
}

impl ByteRead for u32 {
    const SIZE: usize = 4;

    fn read<'a>(buffer: &'a [u8]) -> Self {
        u32::from_be_bytes(buffer[0..4].try_into().unwrap())
    }
}

impl ByteRead for i32 {
    const SIZE: usize = 4;

    fn read<'a>(buffer: &'a [u8]) -> Self {
        i32::from_be_bytes(buffer[0..4].try_into().unwrap())
    }
}

impl ByteRead for u64 {
    const SIZE: usize = 8;

    fn read<'a>(buffer: &'a [u8]) -> Self {
        u64::from_be_bytes(buffer[0..8].try_into().unwrap())
    }
}

impl ByteRead for i64 {
    const SIZE: usize = 8;

    fn read<'a>(buffer: &'a [u8]) -> Self {
        i64::from_be_bytes(buffer[0..8].try_into().unwrap())
    }
}

pub trait VaryingRead<'a> {
    // fn valid(data: &'a [u8]) -> bool;
    fn len(data: &'a [u8]) -> usize;
    fn read(data: &'a [u8]) -> Self;
}

pub struct ReaderContext {
    pub constant_pool: Vec<ConstantPool>,
}

pub trait StreamRead {
    fn read<'a>(stream: &mut ByteStream<'a>, ctx: &ReaderContext) -> Self;
}

macro_rules! impl_stream_read {
    ($ty:ty) => {
        impl StreamRead for $ty {
            fn read<'a>(stream: &mut ByteStream<'a>, _ctx: &ReaderContext) -> Self {
                let value = Self::from_be_bytes(stream.current()[..std::mem::size_of::<Self>()].try_into().unwrap());

                stream.index += std::mem::size_of::<Self>();

                value
            }
        }
    };
}


impl_stream_read!(u8);
impl_stream_read!(i8);
impl_stream_read!(u16);
impl_stream_read!(i16);
impl_stream_read!(u32);
impl_stream_read!(i32);
impl_stream_read!(u64);
impl_stream_read!(i64);

// impl <'a> VaryingByteRead<'a> for &'a str {
// fn valid(data: &'a [u8]) -> bool {
//     data[0] != 0
// }

// fn len(data: &'a [u8]) -> usize {

// }

//     fn read(data: &'a [u8]) -> Self {
//         std::str::from_utf8(data).unwrap()
//     }
// }
