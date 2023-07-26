use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum Value {
    Uninit,
    Null,
    Boolean(bool),
    Char(i16),
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Reference,
    ReturnAdress,
}

impl Value {
    pub fn matches_type(&self, ty: &Type) -> bool {
        match (self, &ty.kind) {
            (Value::Boolean(_), TypeKind::Boolean)
            | (Value::Char(_), TypeKind::Char)
            | (Value::Short(_), TypeKind::Short)
            | (Value::Int(_), TypeKind::Int)
            | (Value::Long(_), TypeKind::Long)
            | (Value::Float(_), TypeKind::Float)
            | (Value::Double(_), TypeKind::Double)
            | (Value::Reference, TypeKind::Reference) => true,
            _ => false,
        }
    }
    //     pub fn set(&mut self, other: Value) -> Result<(), ()> {
    //         match (self, other) {
    //             (Value::Null, Value::Null) => (),
    //             (Value::Boolean())
    //         }

    //         Ok(())
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Uninit => write!(f, "null"),
            Value::Null => write!(f, "null"),
            Value::Boolean(b) => write!(f, "{b}"),
            Value::Char(b) => write!(f, "{b}"),
            Value::Byte(b) => write!(f, "{b}"),
            Value::Short(b) => write!(f, "{b}"),
            Value::Int(b) => write!(f, "{b}"),
            Value::Long(b) => write!(f, "{b}"),
            Value::Float(b) => write!(f, "{b}"),
            Value::Double(b) => write!(f, "{b}"),
            Value::Reference => write!(f, "@"),
            Value::ReturnAdress => write!(f, "@"),
        }
    }
}

macro_rules! impl_op {
    ($op:ident, $ty:ty, $trait_func:ident, $func:ident, $name:expr) => {
        impl std::ops::$op for $ty {
            type Output = Self;

            fn $trait_func(self, rhs: Self) -> Self::Output {
                let (value, wrapped): (Value, _) = match (self, rhs) {
                    (Value::Byte(left), Value::Byte(right)) => {
                        let (result, wrapped) = left.$func(right);
                        (result.into(), wrapped)
                    }
                    (Value::Short(left), Value::Short(right)) => {
                        let (result, wrapped) = left.$func(right);
                        (result.into(), wrapped)
                    }
                    (Value::Int(left), Value::Int(right)) => {
                        let (result, wrapped) = left.$func(right);
                        (result.into(), wrapped)
                    }
                    (Value::Long(left), Value::Long(right)) => {
                        let (result, wrapped) = left.$func(right);
                        (result.into(), wrapped)
                    }
                    (left, right) => {
                        panic!("Unable to add {} and {}", left.as_str(), right.as_str())
                    }
                };

                if wrapped {
                    tracing::warn!("{} {} overflowed", value.as_str(), $name)
                }

                value
            }
        }
    };
    (@float $op:ident, $ty:ty, $trait_func:ident, $func:ident, $name:expr) => {
        impl std::ops::$op for $ty {
            type Output = Self;

            fn $trait_func(self, rhs: Self) -> Self::Output {
                let (value, wrapped): (Value, _) = match (self, rhs) {
                    (Value::Byte(left), Value::Byte(right)) => {
                        let (result, wrapped) = left.$func(right);
                        (result.into(), wrapped)
                    }
                    (Value::Short(left), Value::Short(right)) => {
                        let (result, wrapped) = left.$func(right);
                        (result.into(), wrapped)
                    }
                    (Value::Int(left), Value::Int(right)) => {
                        let (result, wrapped) = left.$func(right);
                        (result.into(), wrapped)
                    }
                    (Value::Long(left), Value::Long(right)) => {
                        let (result, wrapped) = left.$func(right);
                        (result.into(), wrapped)
                    }
                    (Value::Float(left), Value::Float(right)) => {
                        let result = left.$trait_func(right);
                        (result.into(), false)
                    }
                    (Value::Double(left), Value::Double(right)) => {
                        let result = left.$trait_func(right);
                        (result.into(), false)
                    }
                    (left, right) => {
                        panic!("Unable to add {} and {}", left.as_str(), right.as_str())
                    }
                };

                if wrapped {
                    tracing::warn!("{} {} overflowed", value.as_str(), $name)
                }

                value
            }
        }
    }; // ($op:ident, $ty:ty, $trait_func:ident, $func:ident, $name:expr) => {
       //    impl std::ops::$op for $ty {
       //         type Output = Self;

       //         fn $trait_func(self, rhs: Self) -> Self::Output {
       //             let (value, wrapped): (Value, _) = match (self, rhs) {
       //                 (Value::Byte(left), Value::Byte(right)) => {
       //                     let (result, wrapped) = left.$func(right);
       //                     (result.into(), wrapped)
       //                 }
       //                 (Value::Short(left), Value::Short(right)) => {
       //                     let (result, wrapped) = left.$func(right);
       //                     (result.into(), wrapped)
       //                 }
       //                 (Value::Int(left), Value::Int(right)) => {
       //                     let (result, wrapped) = left.$func(right);
       //                     (result.into(), wrapped)
       //                 }
       //                 (Value::Long(left), Value::Long(right)) => {
       //                     let (result, wrapped) = left.$func(right);
       //                     (result.into(), wrapped)
       //                 }
       //                 (Value::Float(left), Value::Float(right)) => {
       //                     let result = left.$trait_func(right);
       //                     (result.into(), false)
       //                 }
       //                 (Value::Double(left), Value::Double(right)) => {
       //                     let result = left.$trait_func(right);
       //                     (result.into(), false)
       //                 }
       //                 (left, right) => panic!("Unable to add {} and {}", left.as_str(), right.as_str()),
       //             };

       //             if wrapped {
       //                 tracing::warn!("{} {} overflowed", value.as_str(), $name)
       //             }

       //             value
       //         }
       //     }
       // };
}

// impl_op!(@float Add, Value, add, overflowing_add, "addition");
// impl_op!(@float Sub, Value, sub, overflowing_sub, "subtractoin");
// impl_op!(@float Mul, Value, mul, overflowing_mul, "multiplication");
// impl_op!(@float Div, Value, div, overflowing_div, "division");

// impl_op!(BitAnd, Value, bitand, bitand, "bitwise and");
// impl_op!(Add, Value, add, overflowing_add, "addition");

// impl std::ops::Add for Value {
//     type Output = Value;

//     fn add(self, rhs: Self) -> Self::Output {
//         let (value, wrapped) = match (self, rhs) {
//             (Value::Int(left), Value::Int(right)) => {
//                 let (result, wrapped) = left.overflowing_add(right);
//                 (result.into(), wrapped)
//             }
//             (left, right) => panic!("Unable to add {} and {}", left.as_str(), right.as_str()),
//         };

//         value
//     }
// }

impl Value {
    pub fn as_str(&self) -> &str {
        match self {
            Value::Uninit => "unitit",
            Value::Null => "null",
            Value::Boolean(_) => "boolean",
            Value::Char(_) => "char",
            Value::Byte(_) => "byte",
            Value::Short(_) => "short",
            Value::Int(_) => "integer",
            Value::Long(_) => "long",
            Value::Float(_) => "float",
            Value::Double(_) => "double",
            Value::Reference => "reference",
            Value::ReturnAdress => "return address",
        }
    }

    pub fn is_category1(&self) -> bool {
        match self {
            Value::Boolean(_)
            | Value::Byte(_)
            | Value::Char(_)
            | Value::Short(_)
            | Value::Int(_)
            | Value::Reference
            | Value::ReturnAdress => true,
            _ => false,
        }
    }

    pub fn is_category2(&self) -> bool {
        match self {
            Value::Long(_) | Value::Double(_) => true,
            _ => false,
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Self::Boolean(i) => *i,
            _ => panic!("Expected bool value!"),
        }
    }

    pub fn as_char(&self) -> char {
        match self {
            Self::Char(i) => char::from_u32(*i as u32).expect("Invalid char digit"),
            _ => panic!("Expected char value!"),
        }
    }

    pub fn as_byte(&self) -> i8 {
        match self {
            Self::Byte(i) => *i,
            _ => panic!("Expected byte value!"),
        }
    }

    pub fn as_short(&self) -> i16 {
        match self {
            Self::Short(i) => *i,
            _ => panic!("Expected short value!"),
        }
    }

    pub fn as_int(&self) -> i32 {
        match self {
            Self::Int(i) => *i,
            _ => panic!("Expected int value! Found {}", self),
        }
    }

    pub fn as_long(&self) -> i64 {
        // std::array::from
        match self {
            Self::Long(i) => *i,
            _ => panic!("Expected long value!"),
        }
    }

    pub fn as_float(&self) -> f32 {
        match self {
            Self::Float(i) => *i,
            _ => panic!("Expected float value!"),
        }
    }

    pub fn as_double(&self) -> f64 {
        match self {
            Self::Double(i) => *i,
            _ => panic!("Expected float value!"),
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<i8> for Value {
    fn from(value: i8) -> Self {
        Self::Byte(value)
    }
}

impl From<i16> for Value {
    fn from(value: i16) -> Self {
        Self::Short(value)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Self::Int(value)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Self::Long(value)
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Self::Float(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Double(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind {
    Boolean,
    Char,
    Byte,
    Short,
    Int,
    Long,
    Float,
    Double,
    Reference,
    Class(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Type {
    array_dimensions: u8,
    kind: TypeKind,
}

impl Type {
    /// Parses method signature suchas "(Ljava/lang/String;)V"
    ///
    /// ##Example
    /// ```
    /// use crate::value::Type;
    ///
    /// let signature = "(I,Z,B)V";
    /// let (params, ty) = parse_signature(signature);
    ///
    /// assert_eq!(params, vec![Type::Int, Type::Boolean, Type::Byte]);
    /// assert_eq!(ret, None);
    /// ```
    pub fn parse_signature(str: impl AsRef<str>) -> (Vec<Type>, Option<Type>) {
        let s = str.as_ref();

        let mut iter = s.split_terminator(['(', ')']);

        let Some("") = iter.next() else { panic!("Malformed method signature!") };
        let params = iter.next().expect("Malformed method signature!");
        let params: Vec<_> = params
            .split(",")
            .take_while(|s| !s.is_empty())
            .map(Type::from)
            .collect();

        let ret_ty = iter.next().expect("Malformed method signature!");

        match ret_ty {
            "V" => (params, None),
            c => (params, Some(Type::from(c))),
        }
    }

    pub const fn boolean() -> Type {
        Type {
            array_dimensions: 0,
            kind: TypeKind::Boolean,
        }
    }

    pub const fn char() -> Type {
        Type {
            array_dimensions: 0,
            kind: TypeKind::Char,
        }
    }

    pub const fn byte() -> Type {
        Type {
            array_dimensions: 0,
            kind: TypeKind::Byte,
        }
    }

    pub const fn short() -> Type {
        Type {
            array_dimensions: 0,
            kind: TypeKind::Short,
        }
    }

    pub const fn int() -> Type {
        Type {
            array_dimensions: 0,
            kind: TypeKind::Int,
        }
    }

    pub const fn long() -> Type {
        Type {
            array_dimensions: 0,
            kind: TypeKind::Long,
        }
    }
}

impl<S: AsRef<str>> From<S> for Type {
    fn from(value: S) -> Self {
        let s = value.as_ref();
        let mut chars = s.chars();
        let mut arr = 0;

        while let Some(char) = chars.next() {
            let ty = match char {
                'B' => TypeKind::Byte,
                'C' => TypeKind::Char,
                'D' => TypeKind::Double,
                'F' => TypeKind::Float,
                'I' => TypeKind::Int,
                'S' => TypeKind::Short,
                'Z' => TypeKind::Boolean,
                'J' => TypeKind::Long,
                'L' => TypeKind::Class(chars.take_while(|c| *c != ';').collect()),
                '[' => {
                    arr += 1;
                    continue;
                }
                _ => panic!("Malformed type string!"),
            };

            return Type {
                array_dimensions: arr,
                kind: ty,
            };
        }

        panic!("Malformed type string! {}", s);
    }
}

#[derive(Debug)]
pub enum RuntimePool {
    Class(runtime_pool::Class),
}

pub mod runtime_pool {
    use std::collections::HashMap;

    use super::{Type, TypeKind, Value};

    #[derive(Debug)]
    pub struct Class {
        pub methods: HashMap<String, Method>,
        pub fields: HashMap<String, Field>,
    }

    #[derive(Debug)]
    pub struct Field {
        pub ty: Type,
        pub value: Value,
    }

    #[derive(Debug)]
    pub struct NativeMethod {
        pub params: Vec<Type>,
        pub return_ty: Option<Type>,
    }

    #[derive(Debug)]
    pub struct JavaMethod {
        pub max_locals: u16,
        pub max_stack: u16,
        pub code_index: usize,

        pub params: Vec<Type>,
        pub return_ty: Option<Type>,
    }

    #[derive(Debug)]
    pub enum Method {
        Native(NativeMethod),
        Java(JavaMethod),
    }

    impl Method {
        pub fn as_method(&self) -> &JavaMethod {
            match self {
                Method::Java(method) => method,
                _ => panic!("Expected Method but found Native Method!"),
            }
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_parse_sig() {
        use crate::value::Type;

        let signature = "(I,Z,B)V";
        let (params, ty) = Type::parse_signature(signature);

        assert_eq!(params, vec![Type::int(), Type::boolean(), Type::byte()]);
        assert_eq!(ty, None);
    }
}
