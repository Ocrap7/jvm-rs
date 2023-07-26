use std::{cell::Cell, ops::*, rc::Rc, sync::atomic::AtomicUsize};

use crate::{
    byte_stream::{ByteStream, ReaderContext},
    error::Result,
    frame::Frame,
    instructions::Instruction,
    rf::Rf,
    runtime::Runtime,
    value::{runtime_pool, Value},
};

pub struct Thread {
    runtime: Rf<Runtime>,

    pc: AtomicUsize,
    stack: Cell<Vec<Value>>,
    frames: Cell<Vec<Frame>>,
}

macro_rules! expand {
    (@phase2($($pat_final:pat => $body_final:tt),*) $op:expr;) => {
        match $op {
            $($pat_final => $body_final,)*
        }
    };
    (@phase2($($pat_final:pat => $body_final:tt),*) $op:expr; $pat:pat => $body:tt, $($rest:tt)*) => {
        expand!(@phase2($($pat_final => $body_final,)* $pat => $body) $op; $($rest)*)
    };
    (@phase2($($pat_final:pat => $body_final:tt),*) $op:expr; @checked $self:expr, $func:ident, $oper:expr, $($pat:pat = $i:ident, $ty:expr;)+, $($rest:tt)*) => {
        expand!(
            @phase2(
                $($pat_final => $body_final,)*
                $(
                    $pat => {
                        let right = $self.pop().$i();
                        let left = $self.pop().$i();

                        let (result, wrapped) = left.$func(right);

                        if wrapped {
                            tracing::warn!("{} {} overflow", $ty, $oper);
                        }

                        $self.push(result)
                    }
                ),*
            )

            $op; $($rest)*
        )
    };
    (@phase2($($pat_final:pat => $body_final:tt),*) $op:expr; @binop $self:expr, $func:ident, $($pat:pat = $i:ident;)+, $($rest:tt)*) => {
        expand!(
            @phase2(
                $($pat_final => $body_final,)*
                $(
                    $pat => {
                        let right = $self.pop().$i();
                        let left = $self.pop().$i();

                        $self.push(left.$func(right))
                    }
                ),*
            )

            $op; $($rest)*
        )
    };
    (@phase2($($pat_final:pat => $body_final:tt),*) $op:expr; @load $self:expr, $func:ident, $($pat:pat = $n:expr;)+, $($rest:tt)*) => {
        expand!(
            @phase2(
                $($pat_final => $body_final,)*
                $(
                    $pat => {
                        let value = $self.get_local($n);
                        value.$func();

                        $self.push(value);
                    }
                ),*
            )

            $op; $($rest)*
        )
    };
    (@phase2($($pat_final:pat => $body_final:tt),*) $op:expr; @store $self:expr, $func:ident, $($pat:pat = $n:expr;)+, $($rest:tt)*) => {
        expand!(
            @phase2(
                $($pat_final => $body_final,)*
                $(
                    $pat => {
                        let value = $self.pop();
                        value.$func();
                        $self.set_local($n, value);
                    }
                ),*
            )

            $op; $($rest)*
        )
    };
    (@phase2($($pat_final:pat => $body_final:tt),*) $op:expr; @cast $self:expr, $func:ident, $($pat:pat = $ty:ty;)+, $($rest:tt)*) => {
        expand!(
            @phase2(
                $($pat_final => $body_final,)*
                $(
                    $pat => {
                        let value = $self.pop().$func();
                        $self.push(value as $ty);
                    }
                ),*
            )

            $op; $($rest)*
        )
    };
    ($op:expr; $($t:tt)*) => {
        expand!(@phase2() $op; $($t)*)
    };
}

impl std::fmt::Debug for Thread {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let stack = self.stack.take();
        let frames = self.frames.take();

        f.debug_struct("Thread")
            // .field("runtime", &self.runtime)
            .field("pc", &self.pc)
            .field("stack", &stack)
            .field("frames", &frames)
            .finish()?;

        self.stack.set(stack);
        self.frames.set(frames);

        Ok(())
    }
}

impl Thread {
    pub fn new(runtime: Rf<Runtime>, pc: usize) -> Thread {
        Thread {
            runtime,
            pc: AtomicUsize::new(pc),
            stack: Cell::new(Vec::new()),
            frames: Cell::new(Vec::new()),
        }
    }

    pub fn with_frame(self, frame: Frame) -> Thread {
        self.frames.set(vec![frame]);
        self
    }

    pub fn run(&self) -> Result<usize> {
        let ctx = ReaderContext {
            constant_pool: Vec::new(),
        };

        loop {
            let ip = self.pc.load(std::sync::atomic::Ordering::Acquire);
            let bytes: [u8; 6] = {
                let rt = self.runtime.borrow();
                let mut instructions = rt.instructions()[ip..].iter();

                core::array::from_fn(|_| instructions.next().copied().unwrap_or(0))
            };

            let mut stream = ByteStream::new(&bytes[..]);

            let instructin_address = ip;
            let instruction = Instruction::from(stream.read::<u8>(&ctx));

            let mut ip_override = None;
            {
                let frames = self.frames.take();
                tracing::trace!("{:#?}", frames,);
                self.frames.set(frames);
            }

            tracing::trace!(
                "Execute Instructin: {:?} 0x{:x} @ {:x}",
                instruction,
                instruction as u8,
                ip
            );

            'outer: {
                expand! {
                    instruction;
                    @load self, as_int,
                        Instruction::ILoad0 = 0;
                        Instruction::ILoad1 = 1;
                        Instruction::ILoad2 = 2;
                        Instruction::ILoad3 = 3;
                    ,
                    @load self, as_long,
                        Instruction::LLoad0 = 0;
                        Instruction::LLoad1 = 1;
                        Instruction::LLoad2 = 2;
                        Instruction::LLoad3 = 3;
                    ,
                    @load self, as_float,
                        Instruction::FLoad0 = 0;
                        Instruction::FLoad1 = 1;
                        Instruction::FLoad2 = 2;
                        Instruction::FLoad3 = 3;
                    ,
                    @load self, as_double,
                        Instruction::DLoad0 = 0;
                        Instruction::DLoad1 = 1;
                        Instruction::DLoad2 = 2;
                        Instruction::DLoad3 = 3;
                    ,
                    @store self, as_int,
                        Instruction::IStore0 = 0;
                        Instruction::IStore1 = 1;
                        Instruction::IStore2 = 2;
                        Instruction::IStore3 = 3;
                    ,
                    @store self, as_long,
                        Instruction::LStore0 = 0;
                        Instruction::LStore1 = 1;
                        Instruction::LStore2 = 2;
                        Instruction::LStore3 = 3;
                    ,
                    @store self, as_float,
                        Instruction::FStore0 = 0;
                        Instruction::FStore1 = 1;
                        Instruction::FStore2 = 2;
                        Instruction::FStore3 = 3;
                    ,
                    @store self, as_double,
                        Instruction::DStore0 = 0;
                        Instruction::DStore1 = 1;
                        Instruction::DStore2 = 2;
                        Instruction::DStore3 = 3;
                    ,
                    @checked self, overflowing_add, "addition",
                        Instruction::IAdd = as_int, "Integer";
                        Instruction::LAdd = as_long, "Long";
                    ,
                    @checked self, overflowing_sub, "subtraction",
                        Instruction::ISub = as_int, "Integer";
                        Instruction::LSub = as_long, "Long";
                    ,
                    @checked self, overflowing_mul, "multiplication",
                        Instruction::IMul = as_int, "Integer";
                        Instruction::LMul = as_long, "Long";
                    ,
                    @checked self, overflowing_div, "division",
                        Instruction::IDiv = as_int, "Integer";
                        Instruction::LDiv = as_long, "Long";
                    ,
                    @checked self, overflowing_rem, "remainder",
                        Instruction::IRem = as_int, "Integer";
                        Instruction::LRem = as_long, "Long";
                    ,
                    @binop self, bitand,
                        Instruction::IAnd = as_int;
                        Instruction::LAnd = as_long;
                    ,
                    @binop self, bitor,
                        Instruction::IOr = as_int;
                        Instruction::LOr = as_long;
                    ,
                    @binop self, bitxor,
                        Instruction::IXOr = as_int;
                        Instruction::LXOr = as_long;
                    ,
                    @cast self, as_int,
                        Instruction::I2B = i8;
                        Instruction::I2S = i16;
                        Instruction::I2L = i64;
                        Instruction::I2F = f32;
                        Instruction::I2D = f64;
                    ,
                    @cast self, as_long,
                        Instruction::L2I = i8;
                        Instruction::L2F = f32;
                        Instruction::L2D = f64;
                    ,
                    Instruction::I2C => {
                        let value = self.pop().as_int();
                        self.push(Value::Char(value as i16));
                    },
                    // Floating
                    @binop self, add,
                        Instruction::FAdd = as_float;
                        Instruction::DAdd = as_double;
                    ,
                    @binop self, sub,
                        Instruction::FSub = as_float;
                        Instruction::DSub = as_double;
                    ,
                    @binop self, mul,
                        Instruction::FMul = as_float;
                        Instruction::DMul = as_double;
                    ,
                    @binop self, div,
                        Instruction::FDiv = as_float;
                        Instruction::DDiv = as_double;
                    ,
                    @binop self, rem,
                        Instruction::FRem = as_float;
                        Instruction::DRem = as_double;
                    ,
                    @cast self, as_float,
                        Instruction::F2I = i32;
                        Instruction::F2L = i64;
                        Instruction::F2D = f64;
                    ,
                    @cast self, as_double,
                        Instruction::D2I = i32;
                        Instruction::D2L = i64;
                        Instruction::D2F = f32;
                    ,

                    // Other instructions
                    Instruction::BiPush => {
                        let byte = stream.read::<i8>(&ctx) as i32;

                        self.push(byte);
                    },
                    Instruction::SiPush => {
                        let short = stream.read::<i16>(&ctx) as i32;

                        self.push(short);
                    },
                    Instruction::IConstM1 => {
                        self.push(-1i32);
                    },
                    Instruction::IConst0 => {
                        self.push(0i32);
                    },
                    Instruction::IConst1 => {
                        self.push(1i32);
                    },
                    Instruction::IConst2 => {
                        self.push(2i32);
                    },
                    Instruction::IConst3 => {
                        self.push(3i32);
                    },
                    Instruction::IConst4 => {
                        self.push(4i32);
                    },
                    Instruction::IConst5 => {
                        self.push(5i32);
                    },
                    Instruction::ILoad => {
                        let index = stream.read::<u8>(&ctx);
                        let value = self.get_local(index);
                        value.as_int();

                        self.push(value);
                    },
                    Instruction::IStore => {
                        let index = stream.read::<u8>(&ctx);
                        let value = self.pop();
                        value.as_int();

                        self.set_local(index, value);
                    },
                    Instruction::INeg => {
                        let value = self.pop().as_int();

                        self.push(-value)
                    },

                    Instruction::IShl => {
                        let right = self.pop().as_int();
                        let left = self.pop().as_int();

                        if right & !0x1F > 0 {
                            tracing::warn!(
                                "Integer left-shift attempted to shifr more than number of bits"
                            );
                        }

                        self.push(left << (right & 0x1f))
                    },
                    Instruction::IShr => {
                        let right = self.pop().as_int();
                        let left = self.pop().as_int();

                        if right & !0x1F > 0 {
                            tracing::warn!(
                                "Integer signed right-shift attempted to shifr more than number of bits"
                            );
                        }

                        self.push(left >> (right & 0x1f))
                    },
                    Instruction::IUShr => {
                        let right = self.pop().as_int();
                        let left = self.pop().as_int();

                        if right & !0x1F > 0 {
                            tracing::warn!(
                                "Integer unsigned right-shift attempted to shifr more than number of bits"
                            );
                        }

                        if right > 0 {
                            self.push(((left as u32) >> (right & 0x1f)) as i32)
                        } else {
                            self.push(right as i32)
                        }
                    },
                    Instruction::IInc => {
                        let index = stream.read::<u8>(&ctx) as usize;
                        let constant = stream.read::<i8>(&ctx) as i32;

                        let mut frames = self.frames.take();
                        let frame = frames.last_mut().expect("Unable to get current frame!");

                        let (result, wrapped) = frame.locals[index].as_int().overflowing_add(constant);

                        if wrapped {
                            tracing::warn!("Integer incrment overflow");
                        }

                        frame.locals[index] = result.into();

                        self.frames.set(frames);
                    },
                    Instruction::LConst0 => {
                        self.push(0i64);
                    },
                    Instruction::LConst1 => {
                        self.push(1i64);
                    },
                    Instruction::LLoad => {
                        let index = stream.read::<u8>(&ctx);
                        let value = self.get_local(index);
                        value.as_long();

                        self.push(value);
                    },
                    Instruction::LStore => {
                        let index = stream.read::<u8>(&ctx);
                        let value = self.pop();
                        value.as_long();

                        self.set_local(index, value);
                    },
                    Instruction::LNeg => {
                        let value = self.pop().as_long();

                        self.push(-value)
                    },
                    Instruction::LShl => {
                        let right = self.pop().as_long();
                        let left = self.pop().as_long();

                        if right & !0x3F > 0 {
                            tracing::warn!(
                                "Long left-shift attempted to shifr more than number of bits"
                            );
                        }

                        self.push(left << (right & 0x3f))
                    },
                    Instruction::LShr => {
                        let right = self.pop().as_long();
                        let left = self.pop().as_long();

                        if right & !0x3F > 0 {
                            tracing::warn!(
                                "Long signed right-shift attempted to shifr more than number of bits"
                            );
                        }

                        self.push(left >> (right & 0x3f))
                    },
                    Instruction::LUShr => {
                        let right = self.pop().as_long();
                        let left = self.pop().as_long();

                        if right & !0x3F > 0 {
                            tracing::warn!(
                                "Long unsigned right-shift attempted to shifr more than number of bits"
                            );
                        }

                        if right > 0 {
                            self.push(((left as u32) >> (right & 0x3f)) as i32)
                        } else {
                            self.push(right as i32)
                        }
                    },

                    // Floating
                    Instruction::FConst0 => {
                        self.push(0f32);
                    },
                    Instruction::FConst1 => {
                        self.push(1f32);
                    },
                    Instruction::FConst2 => {
                        self.push(2f32);
                    },
                    Instruction::FLoad => {
                        let index = stream.read::<u8>(&ctx);
                        let value = self.get_local(index);
                        value.as_float();

                        self.push(value);
                    },
                    Instruction::FStore => {
                        let index = stream.read::<u8>(&ctx);
                        let value = self.pop();
                        value.as_float();

                        self.set_local(index, value);
                    },
                    Instruction::FNeg => {
                        let value = self.pop().as_float();

                        self.push(-value)
                    },

                    Instruction::DConst0 => {
                        self.push(0f32);
                    },
                    Instruction::DConst1 => {
                        self.push(1f32);
                    },
                    Instruction::DLoad => {
                        let index = stream.read::<u8>(&ctx);
                        let value = self.get_local(index);
                        value.as_double();

                        self.push(value);
                    },
                    Instruction::DStore => {
                        let index = stream.read::<u8>(&ctx);
                        let value = self.pop();
                        value.as_double();

                        self.set_local(index, value);
                    },
                    Instruction::DNeg => {
                        let value = self.pop().as_float();

                        self.push(-value)
                    },


                    Instruction::Dup => {
                        let value = self.pop();

                        if !value.is_category1() {
                            panic!("Dup: Expected value to be category 1!")
                        }

                        self.push(value.clone());
                        self.push(value);
                    },
                    Instruction::DupX1 => {
                        let value1 = self.pop();
                        let value2 = self.pop();

                        if !value1.is_category1() || !value2.is_category2() {
                            panic!("DupX1: Expected values to be category 1!")
                        }

                        self.push(value1.clone());
                        self.push(value2);
                        self.push(value1);
                    },
                    Instruction::DupX2 => {
                        let value1 = self.pop();
                        let value2 = self.pop();
                        let value3 = self.pop();

                        if !value1.is_category1() || !value2.is_category2() || !value3.is_category2(){
                            panic!("DupX2: Expected values to be category 1!")
                        }

                        self.push(value1.clone());
                        self.push(value3);
                        self.push(value2);
                        self.push(value1);
                    },

                    Instruction::Dup2 => {
                        let value = self.pop();

                        if value.is_category1() {
                            let value2 = self.pop();

                            self.push(value2.clone());
                            self.push(value.clone());
                            self.push(value2);
                            self.push(value);
                        } else if value.is_category2() {
                            self.push(value.clone());
                            self.push(value);
                        }
                    },
                    Instruction::Dup2X1 => {
                        let value1 = self.pop();

                        if value1.is_category1() {
                            let value2 = self.pop();
                            let value3 = self.pop();

                            self.push(value2.clone());
                            self.push(value1.clone());
                            self.push(value3);
                            self.push(value2);
                            self.push(value1);
                        } else if value1.is_category2() {
                            let value2 = self.pop();

                            self.push(value1.clone());
                            self.push(value2);
                            self.push(value1);
                        }
                    },
                    Instruction::Dup2X2 => {
                        let value1 = self.pop();

                        if value1.is_category1() {
                            let value2 = self.pop();
                            if value2.is_category1() {
                                // Form 1
                                let value3 = self.pop();

                                if value3.is_category1() {
                                    // Form 1
                                    let value4 = self.pop();

                                    self.push(value2.clone());
                                    self.push(value1.clone());
                                    self.push(value3);
                                    self.push(value4);
                                    self.push(value2);
                                    self.push(value1);

                                } else if value3.is_category2() {
                                    // Form 3

                                    self.push(value2.clone());
                                    self.push(value1.clone());
                                    self.push(value3);
                                    self.push(value2);
                                    self.push(value1);
                                } else {
                                    panic!()
                                }
                            } else {
                                panic!()
                            }

                        } else if value1.is_category2() {
                            let value2 = self.pop();

                            if value2.is_category1() {
                                let value3 = self.pop();

                                self.push(value2.clone());
                                self.push(value1.clone());
                                self.push(value3);
                                self.push(value2);
                                self.push(value1);

                            } else if value2.is_category2() {
                                self.push(value1.clone());
                                self.push(value2);
                                self.push(value1);
                            } else {
                                panic!();
                            }
                        } else {
                            panic!();
                        }
                    },

                    Instruction::Goto => {
                        let offset = stream.read::<i16>(&ctx);

                        ip_override = Some(instructin_address.checked_add_signed(offset as isize).expect("Program counter overflow!"))
                    },
                    Instruction::GotoW => {
                        let offset = stream.read::<i32>(&ctx);

                        ip_override = Some(instructin_address.checked_add_signed(offset as isize).expect("Program counter overflow!"))
                    },
                    Instruction::ICmpEq => {
                        let value2 = self.pop().as_int();
                        let value1 = self.pop().as_int();

                        if value1 == value2 {
                            let offset = stream.read::<i16>(&ctx);
                            ip_override = Some(instructin_address.checked_add_signed(offset as isize).expect("Program counter overflow!"))
                        } else {
                            stream.index += 2;
                        }
                    },
                    Instruction::ICmpNe => {
                        let value2 = self.pop().as_int();
                        let value1 = self.pop().as_int();

                        if value1 != value2 {
                            let offset = stream.read::<i16>(&ctx);
                            ip_override = Some(instructin_address.checked_add_signed(offset as isize).expect("Program counter overflow!"))
                        } else {
                            stream.index += 2;
                        }
                    },
                    Instruction::ICmpLt => {
                        let value2 = self.pop().as_int();
                        let value1 = self.pop().as_int();

                        if value1 < value2 {
                            let offset = stream.read::<i16>(&ctx);
                            ip_override = Some(instructin_address.checked_add_signed(offset as isize).expect("Program counter overflow!"))
                        } else {
                            stream.index += 2;
                        }
                    },
                    Instruction::ICmpGt => {
                        let value2 = self.pop().as_int();
                        let value1 = self.pop().as_int();

                        if value1 > value2 {
                            let offset = stream.read::<i16>(&ctx);
                            ip_override = Some(instructin_address.checked_add_signed(offset as isize).expect("Program counter overflow!"))
                        } else {
                            stream.index += 2;
                        }
                    },
                    Instruction::ICmpLe => {
                        let value2 = self.pop().as_int();
                        let value1 = self.pop().as_int();

                        if value1 <= value2 {
                            let offset = stream.read::<i16>(&ctx);
                            ip_override = Some(instructin_address.checked_add_signed(offset as isize).expect("Program counter overflow!"))
                        } else {
                            stream.index += 2;
                        }
                    },
                    Instruction::ICmpGe => {
                        let value2 = self.pop().as_int();
                        let value1 = self.pop().as_int();

                        if value1 >= value2 {
                            let offset = stream.read::<i16>(&ctx);
                            ip_override = Some(instructin_address.checked_add_signed(offset as isize).expect("Program counter overflow!"))
                        } else {
                            stream.index += 2;
                        }
                    },
                    Instruction::LCmp => {
                        let value2 = self.pop().as_long();
                        let value1 = self.pop().as_long();

                        if value1 == value2 {
                            self.push(0i32);
                        } else if value1 > value2 {
                            self.push(1i32);
                        } else if value1 < value2 {
                            self.push(-1i32);
                        }
                    },
                    Instruction::FCmpl => {
                        let value2 = self.pop().as_float();
                        let value1 = self.pop().as_float();

                        if value1 == value2 {
                            self.push(0i32);
                        } else if value1 > value2 {
                            self.push(1i32);
                        } else if value1 < value2 {
                            self.push(-1i32);
                        } else {
                            self.push(-1i32); // NaN
                        }
                    },
                    Instruction::FCmpg => {
                        let value2 = self.pop().as_float();
                        let value1 = self.pop().as_float();

                        if value1 == value2 {
                            self.push(0i32);
                        } else if value1 > value2 {
                            self.push(1i32);
                        } else if value1 < value2 {
                            self.push(-1i32);
                        } else {
                            self.push(1i32); // NaN
                        }
                    },
                    Instruction::DCmpl => {
                        let value2 = self.pop().as_float();
                        let value1 = self.pop().as_float();

                        if value1 == value2 {
                            self.push(0i32);
                        } else if value1 > value2 {
                            self.push(1i32);
                        } else if value1 < value2 {
                            self.push(-1i32);
                        } else {
                            self.push(-1i32); // NaN
                        }
                    },
                    Instruction::DCmpg => {
                        let value2 = self.pop().as_float();
                        let value1 = self.pop().as_float();

                        if value1 == value2 {
                            self.push(0i32);
                        } else if value1 > value2 {
                            self.push(1i32);
                        } else if value1 < value2 {
                            self.push(-1i32);
                        } else {
                            self.push(1i32); // NaN
                        }
                    },
                    Instruction::IEq => {
                        let value1 = self.pop().as_int();

                        if value1 == 0 {
                            let offset = stream.read::<i16>(&ctx);
                            ip_override = Some(instructin_address.checked_add_signed(offset as isize).expect("Program counter overflow!"))
                        } else {
                            stream.index += 2;
                        }
                    },
                    Instruction::INe => {
                        let value1 = self.pop().as_int();

                        if value1 != 0 {
                            let offset = stream.read::<i16>(&ctx);
                            ip_override = Some(instructin_address.checked_add_signed(offset as isize).expect("Program counter overflow!"))
                        } else {
                            stream.index += 2;
                        }
                    },
                    Instruction::ILt => {
                        let value1 = self.pop().as_int();

                        if value1 < 0 {
                            let offset = stream.read::<i16>(&ctx);
                            ip_override = Some(instructin_address.checked_add_signed(offset as isize).expect("Program counter overflow!"))
                        } else {
                            stream.index += 2;
                        }
                    },
                    Instruction::IGt => {
                        let value1 = self.pop().as_int();

                        if value1 > 0 {
                            let offset = stream.read::<i16>(&ctx);
                            ip_override = Some(instructin_address.checked_add_signed(offset as isize).expect("Program counter overflow!"))
                        } else {
                            stream.index += 2;
                        }
                    },
                    Instruction::ILe => {
                        let value1 = self.pop().as_int();

                        if value1 <= 0 {
                            let offset = stream.read::<i16>(&ctx);
                            ip_override = Some(instructin_address.checked_add_signed(offset as isize).expect("Program counter overflow!"))
                        } else {
                            stream.index += 2;
                        }
                    },
                    Instruction::IGe => {
                        let value1 = self.pop().as_int();

                        if value1 >= 0 {
                            let offset = stream.read::<i16>(&ctx);
                            ip_override = Some(instructin_address.checked_add_signed(offset as isize).expect("Program counter overflow!"))
                        } else {
                            stream.index += 2;
                        }
                    },
                    Instruction::IfNull => {
                        let value1 = self.pop();

                        match value1 {
                            Value::Null => {
                                let offset = stream.read::<i16>(&ctx);
                                ip_override = Some(instructin_address.checked_add_signed(offset as isize).expect("Program counter overflow!"))
                            }
                            _ => ()
                        }
                        stream.index += 2;
                    },
                    Instruction::IfNotNull => {
                        let value1 = self.pop();

                        match value1 {
                            Value::Reference => {
                                let offset = stream.read::<i16>(&ctx);
                                ip_override = Some(instructin_address.checked_add_signed(offset as isize).expect("Program counter overflow!"))
                            }
                            _ => ()
                        }

                        stream.index += 2;
                    },

                    Instruction::Pop => {
                        let mut stack = self.stack.take();

                        if !stack.last().unwrap().is_category1() {
                            panic!("Expected category 1 value!")
                        }

                        stack.truncate(stack.len() - 1);
                        self.stack.set(stack);
                    },
                    Instruction::Pop2 => {
                        let mut stack = self.stack.take();

                        if stack.last().unwrap().is_category1() {
                            stack.truncate(stack.len() - 2);
                        } else {
                            stack.truncate(stack.len() - 1);
                        }

                        self.stack.set(stack);
                    },

                    Instruction::Swap => {
                        let mut stack = self.stack.take();
                        let value1 = stack.pop().expect("Expected value to pop");

                        if !value1.is_category1() {
                            panic!("Expected category 1 value!");
                        }

                        let value2 = stack.pop().expect("Expected value to pop");
                        if !value2.is_category1() {
                            panic!("Expected category 1 value!");
                        }

                        stack.push(value2);
                        stack.push(value1);

                        self.stack.set(stack);
                    },
                    Instruction::Return => {
                        let frames = self.frames.take();
                        if frames.len() == 1 {
                            self.frames.set(frames);
                            return Ok(0);
                        }

                        ip_override = Some(self.restore_popped(frames));
                    },
                    Instruction::PutStatic => {
                        let index = stream.read::<u16>(&ctx);
                        tracing::debug!("{}", index);
                        let value = self.pop();

                        let frames = self.frames.take();
                        let frame = frames.last().expect("Unable to retrieve current frame!");

                        let (class, is_init) = {
                            let mut rt = self.runtime.borrow_mut();
                            let (class, _) = rt.get_field_by_index_mut(&frame.class_name, index).expect("Unable to get static field!");
                            let is_init = rt.is_class_initialized(&class);

                            (class, is_init)
                        };

                        if !is_init {
                            // We want to return to this instruction when done initializing.
                            if let Some(ip) = self.initialize_class(ip, &class, index) {
                                self.frames.set(frames);
                                ip_override = Some(ip);
                                break 'outer;
                            }
                        }

                        let mut rt = self.runtime.borrow_mut();
                        let (class, field) = rt.get_field_by_index_mut(&frame.class_name, index).expect("Unable to get static field!");

                        value.matches_type(&field.ty)
                            .then_some(())
                            .expect("Value does not match type!");

                        field.value = value;

                        self.frames.set(frames);
                    },
                    Instruction::InvokeStatic => {
                        let index = stream.read::<u16>(&ctx);

                        let mut frames = self.frames.take();
                        let frame = frames.last().expect("Unable to retrieve current call frame!");

                        let mut rt = self.runtime.borrow_mut();
                        let class = {
                            rt.get_or_load_class_item(&frame.class_name, index).expect("Class not found!")
                        };

                        let (class_name, method_name, method) = rt.get_method_by_index(&frame.class_name, index).expect("Method not found!");

                        match method {
                            runtime_pool::Method::Native(method) => {
                                // TODO; check types
                                let param_len = method.params.len();
                                let mut stack = self.stack.take();
                                let params = &stack[stack.len() - param_len..];

                                let value = rt.invoke_native_function(&format!("{}.{}", class_name, method_name), params);

                                stack.truncate(stack.len() - param_len);
                                self.stack.set(stack);

                                if method.return_ty.is_some() {
                                    self.push(value.expect("Expected return value!"));
                                }
                            }
                            runtime_pool::Method::Java(method) => {
                                // TODO: check types
                                let param_len = method.params.len();
                                let mut stack = self.stack.take();
                                let params = &stack[stack.len() - param_len..];

                                let mut new_frame = Frame::new(stack.len() - param_len, instructin_address + 1, class.clone());
                                new_frame.locals.extend(params.iter().cloned());
                                frames.push(new_frame);

                                ip_override = Some(method.code_index);

                                stack.truncate(stack.len() - param_len);
                                self.stack.set(stack);
                            },
                            _ => (),
                        }

                        self.frames.set(frames);
                    },
                    Instruction::Nop => {},
                    _ => {unimplemented!()},
                };
            }

            self.pc.store(
                ip_override.unwrap_or(ip + stream.index),
                std::sync::atomic::Ordering::Release,
            );
        }
    }

    fn push<V: Into<Value>>(&self, value: V) {
        let mut stack = self.stack.take();
        stack.push(value.into());
        self.stack.set(stack);
    }

    fn pop(&self) -> Value {
        let mut stack = self.stack.take();
        let value = stack.pop().unwrap();
        self.stack.set(stack);

        value
    }

    fn get_local(&self, index: u8) -> Value {
        let mut frames = self.frames.take();
        let current = frames.last_mut().unwrap();

        // if index as usize > current.locals.len() {
        //     current.locals.resize(index as usize + 1, Value::Null);
        // }

        let value = current.locals[index as usize].clone();

        self.frames.set(frames);

        value
    }

    fn set_local<V: Into<Value> + Clone>(&self, index: u8, value: V) {
        let mut frames = self.frames.take();
        let current = frames.last_mut().unwrap();

        if index as usize > current.locals.len() {
            current.locals.resize(index as usize + 1, Value::Null);
        }

        current.locals[index as usize] = value.into();

        self.frames.set(frames);
    }

    fn stack_index(&self) -> usize {
        let frames = self.frames.take();

        let ptr = frames.last().unwrap().stack_pointer;

        self.frames.set(frames);

        ptr
    }

    fn initialize_class(&self, ip: usize, class_name: &Rc<str>, index: u16) -> Option<usize> {
        let stack = self.stack.take();
        let mut frames = self.frames.take();

        let mut rt = self.runtime.borrow_mut();
        {
            rt.get_or_load_class_item(&class_name, index)
                .expect("Class not found!")
        };

        let Some(method) = rt
            .get_method_by_name(&class_name, "<clinit>") else {
                // If no clinit, there is nothing to run.
                rt.set_class_initialized(&class_name);
                self.frames.set(frames);
                self.stack.set(stack);
                return None;
        };

        let new_frame = Frame::new(stack.len(), ip, class_name.clone());
        frames.push(new_frame);

        self.frames.set(frames);
        self.stack.set(stack);

        Some(method.as_method().code_index)
    }

    // fn set_stack_index(&self, index: usize) -> {

    // }

    fn restore_popped(&self, mut frames: Vec<Frame>) -> usize {
        let frame = frames.pop().unwrap();

        let mut stack = self.stack.take();
        stack.truncate(frame.base_pointer);
        self.stack.set(stack);

        self.frames.set(frames);

        frame.return_pc
    }

    fn restore(&self) -> Option<()> {
        let mut frames = self.frames.take();
        let frame = frames.pop()?;

        self.pc
            .store(frame.return_pc, std::sync::atomic::Ordering::SeqCst);

        let mut stack = self.stack.take();
        stack.truncate(frame.base_pointer);
        self.stack.set(stack);

        self.frames.set(frames);

        Some(())
    }
}
