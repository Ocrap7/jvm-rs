use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::{
    bytecode::{ClassFile, ConstantPool},
    error::{Result, VmError},
    frame::Frame,
    instructions,
    rf::Rf,
    thread::Thread,
    value::{runtime_pool, RuntimePool, Type, Value},
};

pub struct Runtime {
    initialized: HashSet<String>,

    class_files: HashMap<String, ClassFile>,

    code_pool: Vec<u8>,
    runtime_pool: HashMap<Rc<str>, RuntimePool>,

    funtions: HashMap<String, Box<dyn Fn(&[Value]) -> Option<Value>>>,
}

impl std::fmt::Debug for Runtime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Runtime")
            .field("initializeda", &self.initialized)
            .field("class_files", &self.class_files)
            .field("code_pool", &self.code_pool)
            .field("runtime_pool", &self.runtime_pool)
            .finish()
    }
}

impl Runtime {
    pub fn new(class_files: Vec<ClassFile>) -> Runtime {
        let class_files = HashMap::from_iter(
            class_files
                .into_iter()
                .map(|class| (class.class_name().to_string(), class)),
        );

        Runtime {
            initialized: HashSet::new(),
            class_files,
            code_pool: Vec::new(),
            runtime_pool: HashMap::new(),

            funtions: Self::setup_native_functions(),
        }
    }

    pub fn start(runtime: Rf<Self>, main_class: &str) -> (Result<usize>, Thread) {
        let (pc, name) = {
            let mut runtime = runtime.borrow_mut();

            runtime
                .load_class(main_class)
                .expect("Unable to load main class!");

            let name = runtime
                .link_class(&main_class)
                .expect("Unable to link main class!");

            println!("{}", instructions::Format::from(runtime.instructions()));

            match runtime.runtime_pool.get(&name) {
                Some(RuntimePool::Class(class)) => {
                    let method = class
                        .methods
                        .get("main")
                        .expect("Main class does not contain a main method!");

                    (method.as_method().code_index, name)
                }
                _ => panic!("Unable to get main class!"),
            }
        };

        let thread = Thread::new(runtime, pc).with_frame(Frame::new_main(name.clone()));

        (thread.run(), thread)
    }

    pub fn instructions(&self) -> &[u8] {
        &self.code_pool
    }

    pub fn load_class(&mut self, class: &str) -> Result<()> {
        {
            if let Some(class_file) = self.class_files.get(class) {
                // self.bootstrap.insert(class.to_string());
            } else {
                return Err(VmError::ClassNotFound(class.to_string()));
            }
        }

        Ok(())
    }

    pub fn link_class(&mut self, class_name: &str) -> Result<Rc<str>> {
        use crate::value::runtime_pool::*;

        let methods: Vec<_> = self
            .get_class(class_name)
            .methods()
            .into_iter()
            .cloned()
            .collect();

        let fields: Vec<_> = self
            .get_class(class_name)
            .fields()
            .into_iter()
            .cloned()
            .collect();

        let methods = HashMap::from_iter(methods.into_iter().map(|method| {
            let class_file = self.get_class(class_name);

            let ConstantPool::Utf8(nat) = class_file.pool(method.descriptor_index as usize) else {
                panic!("Expected method signature string!");
            };

            let (params, ret) = Type::parse_signature(nat.as_str());

            let res = (
                method.name(class_file).to_string(),
                method
                    .code()
                    .map(|code| {
                        let method = Method::Java(JavaMethod {
                            max_locals: code.max_locals,
                            max_stack: code.max_stack,
                            code_index: self.code_pool.len(),

                            params: params.clone(),
                            return_ty: ret.clone(),
                        });

                        // Add instructions to global code section
                        self.code_pool.extend(code.instructions.into_iter());

                        method
                    })
                    .unwrap_or(Method::Native(NativeMethod {
                        params,
                        return_ty: ret,
                    })), // If there is no code, it is a native method
            );

            res
        }));

        let static_fields = HashMap::from_iter(fields.into_iter().map(|field| {
            let class_file = self.get_class(class_name);

            let ConstantPool::Utf8(nat) = class_file.pool(field.descriptor_index as usize) else {
                panic!("Expected method signature string!");
            };

            let ty = Type::from(nat.as_str());

            (
                field.name(class_file).to_string(),
                Field {
                    ty,
                    value: Value::Uninit,
                },
            )
        }));

        let class = Class {
            methods,
            fields: static_fields,
        };

        let name: Rc<str> = class_name.into();
        self.runtime_pool
            .insert(name.clone(), RuntimePool::Class(class));

        Ok(name)
    }

    // pub fn initialize_class(&mut self, lass_name: &str) {
    //     if self.initialized.contains(class_name) {
    //         return;
    //     }

    //     let class_file = self.get_class(class_name);

    // class_file.
    // }

    fn get_class(&self, class: &str) -> &ClassFile {
        self.class_files
            .get(class)
            .expect(&format!("Class '{}' not found", class))
    }

    pub fn get_or_load_class(&mut self, class_name: &Rc<str>) -> Result<Rc<str>> {
        if let Some((k, _)) = self.runtime_pool.get_key_value(class_name) {
            return Ok(k.clone());
        } else {
            self.load_class(class_name)?;
            self.link_class(class_name)
        }
    }

    pub fn get_name_and_type(&self, class: &str, index: u16) -> Option<(&str, &str)> {
        let Some(file) = self.class_files.get(class) else {
            return None;
        };

        let ConstantPool::NameAndType(nat) = file.pool(index as usize) else {
            return None;
        };

        let ConstantPool::Utf8(name) = file.pool(nat.name_index as usize) else {
            return None
        };

        let ConstantPool::Utf8(descriptor) = file.pool(nat.descriptor_index as usize) else {
            return None
        };

        Some((name.as_str(), descriptor.as_str()))
    }

    /// Load a class item if it hasn't been already.
    ///
    /// `class` is the name of the current class file. This is used to read the constant pool
    /// `index` is an index into the `class`'s constant pool
    pub fn get_or_load_class_item(&mut self, class: &str, index: u16) -> Option<Rc<str>> {
        let class_name = {
            let Some(file) = self.class_files.get(class) else {
                return None;
            };

            let (ConstantPool::MethodRef(cref) | ConstantPool::FieldRef(cref) | ConstantPool::InterfaceMethodRef(cref)) = file.pool(index as usize) else {
                return None;
            };

            let ConstantPool::Class(class_pool) = file.pool(cref.class_index as usize) else {
                return None
            };

            let class_name = file.get_str(class_pool.name_index as usize);
            Rc::from(class_name)
        };

        self.get_or_load_class(&class_name).ok()
    }

    pub fn get_field_by_index(
        &self,
        class: &str,
        index: u16,
    ) -> Option<(&str, &str, &runtime_pool::Field)> {
        let Some(file) = self.class_files.get(class) else {
            return None;
        };

        let (ConstantPool::MethodRef(cref) | ConstantPool::FieldRef(cref) | ConstantPool::InterfaceMethodRef(cref)) = file.pool(index as usize) else {
            return None;
        };

        let ConstantPool::Class(class_pool) = file.pool(cref.class_index as usize) else {
            return None
        };

        let class_name = file.get_str(class_pool.name_index as usize);

        let Some((name, _)) = self.get_name_and_type(class, cref.name_and_type_index) else {
            return None
        };

        self.runtime_pool
            .get(class_name)
            .map(|c| match c {
                RuntimePool::Class(class) => class.fields.get(name),
            })
            .flatten()
            .map(|method| (class_name, name, method))
    }

    pub fn get_field_by_index_mut(
        &mut self,
        class: &str,
        index: u16,
    ) -> Option<(Rc<str>, &mut runtime_pool::Field)> {
        let Some(file) = self.class_files.get(class) else {
            return None;
        };

        let (ConstantPool::MethodRef(cref) | ConstantPool::FieldRef(cref) | ConstantPool::InterfaceMethodRef(cref)) = file.pool(index as usize) else {
            return None;
        };

        let ConstantPool::Class(class_pool) = file.pool(cref.class_index as usize) else {
            return None
        };

        let class_name = file.get_str(class_pool.name_index as usize);

        let Some((name, _)) = self.get_name_and_type(class, cref.name_and_type_index) else {
            return None
        };
        // let name = class_name.to_string();
        let field_name = name.to_string();
        let (class_name, _) = self.runtime_pool.get_key_value(class_name)?;
        let class_name = class_name.clone();

        self.runtime_pool
            .get_mut(&class_name)
            .map(|c| match c {
                RuntimePool::Class(class) => class.fields.get_mut(&field_name),
            })
            .flatten()
            .map(|field| (class_name.clone(), field))
    }

    pub fn get_method_by_name(
        &self,
        class_name: &str,
        name: &str,
    ) -> Option<&runtime_pool::Method> {
        self.runtime_pool
            .get(class_name)
            .map(|c| {
                match c {
                    RuntimePool::Class(class) => class.methods.get(name), // _ => None
                }
            })
            .flatten()
    }

    pub fn get_method_by_index(
        &self,
        class: &str,
        index: u16,
    ) -> Option<(&str, &str, &runtime_pool::Method)> {
        let Some(file) = self.class_files.get(class) else {
            return None;
        };

        let (ConstantPool::MethodRef(cref) | ConstantPool::FieldRef(cref) | ConstantPool::InterfaceMethodRef(cref)) = file.pool(index as usize) else {
            return None;
        };

        let ConstantPool::Class(class_pool) = file.pool(cref.class_index as usize) else {
            return None
        };

        let class_name = file.get_str(class_pool.name_index as usize);

        let Some((name, _)) = self.get_name_and_type(class, cref.name_and_type_index) else {
            return None
        };

        self.get_method_by_name(class_name, name)
            .map(|method| (class_name, name, method))
    }

    pub fn invoke_native_function(&self, name: &str, params: &[Value]) -> Option<Value> {
        let Some(func) = self.funtions.get(name) else {
            panic!("Function '{}' not found", name)
        };

        func(params)
    }

    pub fn is_class_initialized(&self, class: &str) -> bool {
        self.initialized.contains(class)
    }

    pub fn set_class_initialized(&mut self, class: &str) {
        self.initialized.insert(class.to_string());
    }

    fn setup_native_functions() -> HashMap<String, Box<dyn Fn(&[Value]) -> Option<Value> + 'static>>
    {
        // let funcs = [("as".to_string(), Box::new(|| {}) as Box<dyn Fn()>)];
        let mut funcs = HashMap::new();

        let mut add_func = |name: &str, value: Box<dyn Fn(&[Value]) -> Option<Value>>| {
            funcs.insert(name.to_string(), value)
        };

        add_func(
            "Test/Main.out",
            Box::new(|params| {
                println!("{}", params[0]);
                None
            }),
        );

        funcs
    }
}
