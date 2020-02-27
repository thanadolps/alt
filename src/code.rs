use crate::code::UnitMemLocation::Mem;
use crate::interpreter::{CodeBlock, MemUnit};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::hash::BuildHasherDefault;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum CodePoint {
    Set {
        dest: MemLocation,
        value: MemUnit,
    },
    Cpy {
        dest: MemLocation,
        source: MemLocation,
    },
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Cmp,
    BJmp {
        cond: MemLocation,
    },
    Land,
    // TODO: find a way to store hash instate?
    RAdd {
        name: String,
    },
    RSwp {
        name: String,
    },
    // TODO: make this custom print to impl Write (not just Stdout)?
    Print {
        source: ValMemLoc,
    },
    PrintC {
        source: ValMemLoc,
    },
    Term,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Routines {
    pub routine_map: HashMap<String, CodeBlock, BuildHasherDefault<seahash::SeaHasher>>,
}

impl Routines {
    pub fn new() -> Self {
        Routines {
            routine_map: HashMap::default(),
        }
    }

    pub fn add_routine(&mut self, name: String, routines: CodeBlock) -> Result<(), &'static str> {
        if let Some(_dup_rout) = self.routine_map.insert(name, routines) {
            Err("Routine with the same name already exist")
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Registry {
    I1,
    I2,
    O,
}

// literal memory location without indirection
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum UnitMemLocation {
    Registry(Registry),
    Mem(usize),
}

impl TryFrom<&str> for UnitMemLocation {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use crate::code::Registry::*;
        use UnitMemLocation::Registry;
        match value {
            "I1" => Ok(Registry(I1)),
            "I2" => Ok(Registry(I2)),
            "O" => Ok(Registry(O)),
            mem => match mem.split_at(1) {
                ("M", m) => Ok(Mem(m
                    .parse::<usize>()
                    .map_err(|_| "M follow by invalid memory location")?)),
                _ => Err("Unknown memory location"),
            },
        }
    }
}

// memory location that may be indirect
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemLocation {
    pub ptr_count: u8,
    pub unit_mem: UnitMemLocation,
}

impl TryFrom<&str> for MemLocation {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let unit_mem_str = value.trim_start_matches('*');
        let unit_mem = unit_mem_str.try_into()?;
        let ptr_count = (value.len() - unit_mem_str.len()) as u8;
        Ok(MemLocation {
            ptr_count,
            unit_mem,
        })
    }
}

// memory location or literal number
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum ValMemLoc {
    MemLoc(MemLocation),
    Value(MemUnit),
}

impl TryFrom<&str> for ValMemLoc {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use ValMemLoc::*;
        MemLocation::try_from(value).map(MemLoc).or_else(|_| {
            value
                .parse()
                .map(Value)
                .map_err(|_| "invalid literal/memory location")
        })
    }
}
