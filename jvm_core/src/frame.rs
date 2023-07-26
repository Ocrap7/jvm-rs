use std::rc::Rc;

use bitflags::bitflags;

use crate::value::Value;

bitflags! {
    #[derive(Debug)]
    pub struct FrameFlags: u32 {
        const CLINIT = 1;
    }
}

#[derive(Debug)]
pub struct Frame {
    pub locals: Vec<Value>,
    pub base_pointer: usize,
    pub stack_pointer: usize,
    pub return_pc: usize,
    pub class_name: Rc<str>,
    pub flags: FrameFlags,
}

impl Frame {
    pub fn new_main(class_name: Rc<str>) -> Frame {
        Frame {
            locals: Vec::new(),
            base_pointer: 0,
            stack_pointer: 0,
            return_pc: 0,
            class_name,
            flags: FrameFlags::empty(),
        }
    }

    pub fn new(stack: usize, return_pc: usize, class_name: Rc<str>) -> Frame {
        Frame {
            locals: Vec::new(),
            base_pointer: stack,
            stack_pointer: stack,
            return_pc,
            class_name,
            flags: FrameFlags::empty(),
        }
    }

    pub fn new_clinit(stack: usize, return_pc: usize, class_name: Rc<str>) -> Frame {
        Frame {
            locals: Vec::new(),
            base_pointer: stack,
            stack_pointer: stack,
            return_pc,
            class_name,
            flags: FrameFlags::CLINIT,
        }
    }
}
