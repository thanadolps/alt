use crate::code::{CodePoint, MemLocation, ValMemLoc, Routines, UnitMemLocation};

use std::cmp::Ordering;
use serde::{Serialize, Deserialize};

pub type MemUnit = u8;

#[derive(Debug)]
pub struct Interpreter {
    pub memory: Memory,
    pub runtime_stack: Vec<CodeFrame>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            memory: Memory::new(),
            runtime_stack: Vec::new(),
        }
    }

    pub fn execute(&mut self, routines: &Routines) -> Result<(), &'static str> {
        self.execute_routine("Main", routines)?;
        Ok(())
    }

    pub fn execute_routine(&mut self, routine_name: &str, routines: &Routines) -> Result<(), &'static str> {
        let frame = Interpreter::make_code_frame(&routines, routine_name)?;

        self.runtime_stack.push(frame);
        self.run(routines);

        Ok(())
    }

    fn make_code_frame(
        routines: &Routines,
        routine_name: &str,
    ) -> Result<CodeFrame, &'static str> {
        let routine = routines.routine_map
            .get(routine_name)
            .ok_or("call of undeclared routine")?
            .clone();
        Ok(CodeFrame::new(routine))
    }

    // this is for internal use. for public use, use execute
    fn run(&mut self, routines: &Routines) {
        let Interpreter { runtime_stack, memory, .. } = self;

        // Main meat
        loop {
            let mut jumping = false;
            if let Some(CodeFrame { fetcher }) = runtime_stack.last_mut() {
                if fetcher.len() == 0 { runtime_stack.pop(); continue; }
                for code_point in fetcher {
                    match code_point {
                        CodePoint::Land if jumping => {
                            jumping = false;
                        }
                        _ if jumping => {}
                        CodePoint::Set { dest, value } => {
                            memory.set(dest, value);
                        }
                        CodePoint::Cpy { dest, source } => {
                            let data = memory.get(source);
                            memory.set(dest, data);
                        }
                        CodePoint::Add => memory.registry.add(),
                        CodePoint::Sub => memory.registry.sub(),
                        CodePoint::Mul => memory.registry.mul(),
                        CodePoint::Div => memory.registry.div(),
                        CodePoint::Mod => memory.registry.modulo(),
                        CodePoint::Cmp => {
                            let cmp_val = match memory.registry.cmp() {
                                Ordering::Less => 0,
                                Ordering::Equal => 1,
                                Ordering::Greater => 2,
                            };
                            memory.registry.o = cmp_val;
                        }
                        CodePoint::BJmp { cond } => jumping = memory.get(cond) != 0,
                        CodePoint::RAdd { name } => {
                            let frame =
                                Interpreter::make_code_frame(routines, &name).unwrap();
                            runtime_stack.push(frame);
                            break;
                        }
                        CodePoint::RSwp { name } => {
                            let frame =
                                Interpreter::make_code_frame(routines, &name).unwrap();
                            runtime_stack.pop();
                            runtime_stack.push(frame);
                            break;
                        },
                        CodePoint::Term => {
                            runtime_stack.pop();
                            break;
                        }
                        CodePoint::Print { source } => match source {
                            ValMemLoc::MemLoc(source) => print!("{}", memory.get(source)),
                            ValMemLoc::Value(lit) => print!("{}", lit),
                        },
                        CodePoint::PrintC { source } => match source {
                            ValMemLoc::MemLoc(source) => {
                                print!("{}", memory.get(source) as char)
                            }
                            ValMemLoc::Value(lit) => print!("{}", lit as char),
                        }
                        _ => eprintln!("WARNING: unknown code point"),
                    }
                }
            } else {
                return;
            }
        }
    }
}

#[derive(Debug)]
pub struct Registry {
    i1: MemUnit,
    i2: MemUnit,
    o: MemUnit,
}

impl Registry {
    pub const fn new() -> Self {
        Registry { i1: 0, i2: 0, o: 0 }
    }

    pub fn get(&self, reg: crate::code::Registry) -> MemUnit {
        match reg {
            crate::code::Registry::I1 => self.i1,
            crate::code::Registry::I2 => self.i2,
            crate::code::Registry::O => self.o,
        }
    }

    pub fn get_mut(&mut self, reg: crate::code::Registry) -> &mut MemUnit {
        match reg {
            crate::code::Registry::I1 => &mut self.i1,
            crate::code::Registry::I2 => &mut self.i2,
            crate::code::Registry::O => &mut self.o,
        }
    }

    pub fn set(&mut self, reg: crate::code::Registry, val: MemUnit) {
        *self.get_mut(reg) = val;
    }

    pub fn add(&mut self) {
        self.o = self.i1.wrapping_add(self.i2);
    }

    pub fn sub(&mut self) {
        self.o = self.i1.wrapping_sub(self.i2);
    }

    pub fn mul(&mut self) {
        self.o = self.i1.wrapping_mul(self.i2);
    }
    pub fn div(&mut self) {
        self.o = self.i1.wrapping_div(self.i2);
    }

    pub fn modulo(&mut self) {
        self.o = self.i1.wrapping_rem(self.i2);
    }

    pub fn cmp(&mut self) -> Ordering {
        self.i1.cmp(&self.i2)
    }
}

#[derive(Debug)]
pub struct Memory {
    registry: Registry,
    mem: Vec<MemUnit>,
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            registry: Registry::new(),
            mem: Vec::with_capacity(32),
        }
    }

    pub fn get(&self, loc: MemLocation) -> MemUnit {
        match self.pointer_chase(loc) {
            UnitMemLocation::Registry(reg) => self.registry.get(reg),
            UnitMemLocation::Mem(i) => { self.mem_get(i) },
        }
    }

    fn mem_get(&self, index: usize) -> u8 {
        self.mem.get(index).cloned().unwrap_or(0)
    }

    // chase down and deref the pointer to final memory location (only work on mem and not registry)
    fn pointer_chase(&self, mem_loc: MemLocation) -> UnitMemLocation {
        let MemLocation {ptr_count, unit_mem: unit_loc, } = mem_loc;

        // just return if there's no deref
        if ptr_count == 0 {
            return mem_loc.unit_mem;
        }

        // first deref (special case cause unit_loc can be registry)
        // since pointer cannot point to registry, after this the location will be on MEM
        // so we can just store an index (usize)
        let mut index_val =
            match unit_loc {
                UnitMemLocation::Registry(reg) => self.registry.get(reg),
                UnitMemLocation::Mem(i) => self.mem_get(i),
            } as usize;

        for _ in 1..ptr_count {
            index_val = self.mem_get(index_val) as usize;
        }
        UnitMemLocation::Mem(index_val)
    }

    // this will auto expand
    pub fn set(&mut self, loc: MemLocation, val: MemUnit) {
        match self.pointer_chase(loc) {
            UnitMemLocation::Registry(reg) => {
                self.registry.set(reg, val)
            },
            UnitMemLocation::Mem(index) => {
                if let Some(mem_ref) = self.mem.get_mut(index) {
                    *mem_ref = val;
                } else {
                    self.mem.resize(index + 1, 0);
                    self.mem[index] = val;
                }
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CodeBlock {
    pub code: Vec<CodePoint>,
}

impl CodeBlock {
    pub const fn new() -> Self {
        CodeBlock { code: Vec::new() }
    }
}

#[derive(Debug)]
pub struct CodeFrame {
    pub fetcher: std::vec::IntoIter<CodePoint>,
}

impl CodeFrame {
    pub fn new(code_block: CodeBlock) -> Self {
        CodeFrame {
            fetcher: code_block.code.into_iter(),
        }
    }
}
