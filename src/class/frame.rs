use std::collections::VecDeque;

use broom::Handle;

use super::method::Argument;
use crate::class::method::MethodType;
use crate::class::ConstantsPoolInfo;
use crate::vastatrix::{VTXObject, Vastatrix};

pub trait Frame: core::fmt::Debug {
    fn exec(&mut self, args: Vec<Argument>, running_in: &mut Vastatrix) -> Argument;
}

#[derive(Debug)]
pub struct BytecodeFrame {
    pub class_handle: Handle<VTXObject>,
    pub method:       String,
    pub ip:           u32,
    pub code:         Vec<u8>,
    pub locals:       Vec<Argument>,
    pub stack:        VecDeque<Argument>,
}

impl Frame for BytecodeFrame {
    fn exec(&mut self, args: Vec<Argument>, running_in: &mut Vastatrix) -> Argument {
        // either its a 32 bit int or its a void, type checking should catch this (in
        // the future, for now i'm just relying on the compiler) would rather
        // not do JIT yet...
        trace!("Method: {}, locals len: {}", self.method, self.locals.len());
        for index in 0..args.len() {
            self.locals[index] = args[index].clone();
        }
        loop {
            let op = self.code[self.ip as usize];
            let class = running_in.get_class(self.class_handle);
            let this_class = &class.get_constant_pool()[class.get_this_class() as usize - 1];
            if let ConstantsPoolInfo::Class { name_index, } = this_class {
                let name = &class.get_constant_pool()[*name_index as usize - 1];
                if let ConstantsPoolInfo::Utf8 { bytes, .. } = name {
                    debug!("class: {}, method: {}, opcode: 0x{:x}, current stack:{:?}", bytes.to_string(), self.method, op, self.stack);
                }
            }
            drop(this_class);
            match op {
                0x2 => {
                    // iconst_m1
                    self.stack.push_back(Argument::new(-1, MethodType::Int));
                },
                0x3 => {
                    // iconst_0
                    self.stack.push_back(Argument::new(0, MethodType::Int));
                },
                0x4 => {
                    // iconst_1
                    self.stack.push_back(Argument::new(1, MethodType::Int));
                },
                0x5 => {
                    // iconst_2
                    self.stack.push_back(Argument::new(2, MethodType::Int));
                },
                0x6 => {
                    // iconst_3
                    self.stack.push_back(Argument::new(3, MethodType::Int));
                },
                0x7 => {
                    self.stack.push_back(Argument::new(4, MethodType::Int));
                },
                0x8 => {
                    self.stack.push_back(Argument::new(5, MethodType::Int));
                },
                0x12 => {
                    let index = self.code[self.ip as usize + 1];
                    let class = running_in.get_class(self.class_handle);
                    let constant = &class.get_constant_pool()[index as usize - 1];
                    match constant {
                        ConstantsPoolInfo::Integer { bytes, } => {
                            self.stack.push_back(Argument::new(*bytes as i32, MethodType::Int));
                        },
                        a => {
                            panic!("BAD! {:?}", a);
                        },
                    }
                },
                0x15 => {
                    self.ip += 1;
                    self.stack.push_back(self.locals[self.code[self.ip as usize] as usize].clone());
                },
                0x1A => {
                    // iload_0
                    self.stack.push_back(self.locals[0].clone());
                },
                0x1B => {
                    // iload_1
                    self.stack.push_back(self.locals[1].clone());
                },
                0x1C => {
                    self.stack.push_back(self.locals[2].clone());
                },
                0x1D => {
                    self.stack.push_back(self.locals[3].clone());
                },
                0x2A => {
                    self.stack.push_back(self.locals[0].clone());
                },
                0x36 => {
                    // istore     index
                    let value = self.stack.pop_front().unwrap();
                    self.ip += 1;
                    self.locals[self.code[self.ip as usize] as usize] = value;
                },
                0x3B => {
                    // istore_0
                    let value = self.stack.pop_front().unwrap();
                    self.locals[0] = value;
                },
                0x3C => {
                    let value = self.stack.pop_front().unwrap();
                    self.locals[1] = value;
                },
                0x3D => {
                    let value = self.stack.pop_front().unwrap();
                    self.locals[2] = value;
                },
                0x3E => {
                    let value = self.stack.pop_front().unwrap();
                    self.locals[3] = value;
                },
                0x4B => {
                    let value = self.stack.pop_front().unwrap();
                    trace!("LOCALS LENGTH: {}", self.locals.len());
                    self.locals[0] = value;
                },
                0x57 => {
                    self.stack.pop_front().unwrap();
                },
                0x59 => {
                    let value = &self.stack[0];
                    self.stack.push_back(value.clone());
                },
                0x60 => {
                    // iadd
                    let a = self.stack.pop_front().unwrap();
                    let b = self.stack.pop_front().unwrap();
                    self.stack.push_back(b.wrapping_iadd(a));
                },
                0x64 => {
                    // isub
                    let a = self.stack.pop_front().unwrap();
                    let b = self.stack.pop_front().unwrap();
                    self.stack.push_back(b.wrapping_isub(a));
                },
                0x68 => {
                    // imul
                    let a = self.stack.pop_front().unwrap();
                    let b = self.stack.pop_front().unwrap();
                    self.stack.push_back(b.wrapping_imul(a));
                },
                0x6C => {
                    // idiv
                    let a = self.stack.pop_front().unwrap();
                    let b = self.stack.pop_front().unwrap();
                    self.stack.push_back(b.wrapping_idiv(a));
                },
                0x84 => {
                    let index = self.code[(self.ip + 1) as usize];
                    let cons_t = self.code[(self.ip + 2) as usize];
                    self.locals[index as usize] += cons_t as i32;
                    self.ip += 2;
                },
                0xA7 => {
                    let branchbyte1 = self.code[(self.ip + 1) as usize];
                    trace!("{}", branchbyte1);
                    let branchbyte2 = self.code[(self.ip + 2) as usize];
                    trace!("{}", branchbyte2);
                    self.ip = self.ip.checked_add_signed((((((branchbyte1 as u16) << 8) | branchbyte2 as u16) - 1) as i16).into()).unwrap();
                    trace!("{:?}", self.code);
                },
                0xAC => {
                    // ireturn
                    let v = self.stack.pop_front().unwrap();
                    return v;
                },
                0xA2 => {
                    // if_icmpge    brancbyte1      branchbyte2
                    let value1 = self.stack.pop_front().unwrap();
                    let value2 = self.stack.pop_front().unwrap();
                    let branchbyte1 = self.code[(self.ip + 1) as usize];
                    let branchbyte2 = self.code[(self.ip + 2) as usize];
                    if value1 >= value2 {
                        self.ip += (((branchbyte1 as u32) << 8) | branchbyte2 as u32) - 1;
                    } else {
                        self.ip += 2;
                    }
                },
                0xB1 => {
                    return Argument::new(0, MethodType::Void);
                },
                0xB4 => {
                    let indexbyte1 = self.code[(self.ip + 1) as usize];
                    let indexbyte2 = self.code[(self.ip + 2) as usize];
                    let mut objectref = self.stack.pop_front().unwrap();
                    let this_class = running_in.get_class(self.class_handle).clone();
                    let instance = running_in.get_instance(objectref.value_ref() as usize);
                    let field_info = &this_class.get_constant_pool()[(((indexbyte1 as usize) << 8) | indexbyte2 as usize) - 1];
                    if let ConstantsPoolInfo::FieldRef { name_and_type_index, ..} = field_info {
                        //let class = &this_class.get_constant_pool()[*class_index as usize - 1];
                        /*if let ConstantsPoolInfo::Class { name_index } = class {
                            let class_name = &this_class.constant_pool[*name_index as usize - 1];
                            if let ConstantsPoolInfo::Utf8 { length, bytes } = class_name {
                                let class_handle = running_in.load_or_get_class_handle(bytes.to_string());
                                let that_class = running_in.get_class(class_handle); // don't know if i need this right now.
                            }
                        }*/
                        let name_and_type = &this_class.get_constant_pool()[*name_and_type_index as usize - 1];
                        if let ConstantsPoolInfo::NameAndType { name_index, .. } = name_and_type {
                            let name = &this_class.get_constant_pool()[*name_index as usize - 1];
                            if let ConstantsPoolInfo::Utf8 { bytes, .. } = name {
                                self.stack.push_back(instance.fields.get(&bytes.to_string()).expect("a").clone());
                            }
                        }
                    }
                    self.ip += 2;
                },
                0xB5 => {
                    let indexbyte1 = self.code[(self.ip + 1) as usize];
                    let indexbyte2 = self.code[(self.ip + 2) as usize];
                    let mut objectref = self.stack.pop_front().unwrap();
                    let value = self.stack.pop_front().unwrap();
                    let this_class = running_in.get_class(self.class_handle).clone();
                    let instance = running_in.get_instance(objectref.value_ref() as usize);
                    let field_info = &this_class.get_constant_pool()[(((indexbyte1 as usize) << 8) | indexbyte2 as usize) - 1];
                    if let ConstantsPoolInfo::FieldRef { name_and_type_index, .. } = field_info {
                        //let class = &this_class.get_constant_pool()[*class_index as usize - 1];
                        /*if let ConstantsPoolInfo::Class { name_index } = class {
                            let class_name = &this_class.constant_pool[*name_index as usize - 1];
                            if let ConstantsPoolInfo::Utf8 { length, bytes } = class_name {
                                let class_handle = running_in.load_or_get_class_handle(bytes.to_string());
                                let that_class = running_in.get_class(class_handle); // don't know if i need this right now.
                            }
                        }*/
                        let name_and_type = &this_class.get_constant_pool()[*name_and_type_index as usize - 1];
                        if let ConstantsPoolInfo::NameAndType { name_index, .. } = name_and_type {
                            let name = &this_class.get_constant_pool()[*name_index as usize - 1];
                            if let ConstantsPoolInfo::Utf8 { bytes, .. } = name {
                                instance.fields.insert(bytes.to_string(), value);
                            }
                        }
                    }
                    self.ip += 2;
                },
                0xB6 => {
                    let indexbyte1 = self.code[(self.ip + 1) as usize];
                    let indexbyte2 = self.code[(self.ip + 2) as usize];
                    let this_class = running_in.get_class(self.class_handle).clone();
                    let objectref = self.stack.pop_front().unwrap();
                    let method_info = &this_class.get_constant_pool()[(((indexbyte1 as usize) << 8) | indexbyte2 as usize) - 1];
                    if let ConstantsPoolInfo::MethodRef { .. } = method_info {
                        let (mut method, desc) = this_class.resolve_method(method_info.clone(), false, None, running_in);
                        let mut args: Vec<Argument> = vec![objectref];
                        for _ in desc.types {
                            args.push(self.stack.pop_front().unwrap());
                        }
                        let back = method.exec(args, running_in);
                        if !back.void() {
                            self.stack.push_back(back);
                        }
                    }

                    self.ip += 2;
                },
                0xB7 => {
                    let indexbyte1 = self.code[(self.ip + 1) as usize];
                    let indexbyte2 = self.code[(self.ip + 2) as usize];
                    let this_class = running_in.get_class(self.class_handle).clone();
                    let objectref = self.stack.pop_front().unwrap();
                    let method_info = &this_class.get_constant_pool()[(((indexbyte1 as usize) << 8) | indexbyte2 as usize) - 1];
                    if let ConstantsPoolInfo::MethodRef { .. } = method_info {
                        let (mut method, desc) = this_class.resolve_method(method_info.clone(), false, None, running_in);
                        let mut args: Vec<Argument> = vec![objectref];
                        for _ in desc.types {
                            args.push(self.stack.pop_front().unwrap());
                        }
                        let back = method.exec(args, running_in);
                        if !back.void() {
                            self.stack.push_back(back);
                        }
                    }
                    self.ip += 2;
                },
                0xB8 => {
                    let indexbyte1 = self.code[(self.ip + 1) as usize];
                    let indexbyte2 = self.code[(self.ip + 2) as usize];
                    trace!("byte1: {}, byte2: {}, final: {}", indexbyte1, indexbyte2, (((indexbyte1 as usize) << 8) | indexbyte2 as usize) - 1);
                    let this_class = running_in.get_class(self.class_handle).clone();
                    let method_info = &this_class.get_constant_pool()[(((indexbyte1 as usize) << 8) | indexbyte2 as usize) - 1]; // i have to asssume that indices in terms of the internals of the jvm start at 1, otherwise i have no idea why i'd have to subtract 1 here.
                    if let ConstantsPoolInfo::MethodRef { .. } = method_info {
                        let (mut method, desc) = this_class.resolve_method(method_info.clone(), false, None, running_in);
                        let mut args: Vec<Argument> = vec![];
                        for _ in desc.types {
                            args.push(self.stack.pop_front().unwrap());
                        }
                        let back = method.exec(args, running_in);
                        if !back.void() {
                            self.stack.push_back(back);
                        }
                    } else {
                        panic!("invokestatic was not a method reference!");
                    }
                    self.ip += 2;
                },
                0xBB => {
                    let indexbyte1 = self.code[(self.ip + 1) as usize];
                    let indexbyte2 = self.code[(self.ip + 2) as usize];
                    let this_class = running_in.get_class(self.class_handle).clone();
                    let class_info = &this_class.get_constant_pool()[(((indexbyte1 as usize) << 8) | indexbyte2 as usize) - 1];
                    if let ConstantsPoolInfo::Class { name_index, } = class_info {
                        let name = &this_class.get_constant_pool()[*name_index as usize - 1];
                        if let ConstantsPoolInfo::Utf8 { bytes, .. } = name {
                            let handle = running_in.load_or_get_class_handle(bytes.to_string());
                            let mut class = running_in.get_class(handle).clone();
                            self.stack.push_back(Argument::new(running_in.prepare_instance(&mut class),
                                                               MethodType::ClassReference { classpath: bytes.to_string(), }));
                        }
                    }
                    self.ip += 2;
                },
                _ => {
                    panic!("Unimplemented opcode: 0x{:x}", op);
                },
            }
            self.ip += 1;
        }
    }
}
