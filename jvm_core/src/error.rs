use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum VmError {
    ClassNotFound(String),
}

impl Display for VmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ClassNotFound(class_name) => write!(f, "Class '{}' not found!", class_name),
        }
    }
}

impl std::error::Error for VmError {}

pub type Result<T> = std::result::Result<T, VmError>;
