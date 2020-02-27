use crate::code::{CodePoint, Routines, ValMemLoc};
use crate::interpreter::CodeBlock;
use itertools::Itertools;
use std::convert::TryInto;
use std::iter::empty;

impl Routines {
    pub fn parse<'a>(
        &mut self,
        source_code: impl Iterator<Item = &'a str>,
    ) -> Result<(), &'static str> {
        // sanitation
        let routine_source_iter = source_code
            .map(str::trim)
            .filter(|line| !line.is_empty() && !is_comment_source(line));

        let mut last_routine: Option<String> = None;
        for (is_routine, group) in &routine_source_iter.group_by(|elt| is_routine_source(elt)) {
            if is_routine {
                for rout in group.map(|x: &str| x.trim_end_matches(':')) {
                    if let Some(prev_routine) = &last_routine {
                        // there's prev routine but it don't contain any code
                        self.parse_routine(prev_routine.clone(), empty())?;
                    }
                    last_routine = Some(rout.to_string());
                }
            } else {
                // parse routine and use it up (by setting it to None)
                if let Some(prev_routine) = last_routine {
                    self.parse_routine(prev_routine, group)?;
                    last_routine = None;
                } else {
                    // detected code without routine
                    // TODO: make this proper error
                    return Err("code without routine");
                }
            }
        }

        if let Some(prev_routine) = &last_routine {
            self.parse_routine(prev_routine.clone(), empty())?;
        }

        Ok(())
    }

    pub(super) fn parse_routine<'a>(
        &mut self,
        name: String,
        source_code: impl Iterator<Item = &'a str>,
    ) -> Result<(), &'static str> {
        let code_block = parse_codeblock(source_code)?;
        self.add_routine(name, code_block)?;
        Ok(())
    }
}

fn parse_codeblock<'a>(
    source_code: impl Iterator<Item = &'a str>,
) -> Result<CodeBlock, &'static str> {
    let mut code_block = CodeBlock::new();

    for line in source_code.filter(|x| !x.is_empty()) {
        let code_point = parse_line(line)?;
        code_block.code.push(code_point);
    }

    Ok(code_block)
}

fn parse_line(line: &str) -> Result<CodePoint, &'static str> {
    use CodePoint::*;

    let token_gen = line.split_whitespace().collect::<Vec<_>>();

    match *token_gen.as_slice() {
        ["SET", dest, val] => Ok(Set {
            dest: dest.try_into()?,
            value: val.parse().map_err(|_| "Cannot parse value")?,
        }),
        ["CPY", dest, source] => Ok(Cpy {
            dest: dest.try_into()?,
            source: source.try_into()?,
        }),
        ["ADD"] => Ok(Add),
        ["SUB"] => Ok(Sub),
        ["MUL"] => Ok(Mul),
        ["DIV"] => Ok(Div),
        ["MOD"] => Ok(Mod),
        ["CMP"] => Ok(Cmp),
        ["BJMP", cond] => Ok(BJmp {
            cond: cond.try_into()?,
        }),
        ["LAND"] => Ok(Land),
        ["RADD", name] => Ok(RAdd {
            name: name.to_owned(),
        }),
        ["RSWP", name] => Ok(RSwp {
            name: name.to_owned(),
        }),
        ["TERM"] => Ok(Term),
        ["PRINT", source] => Ok(Print {
            source: source.try_into()?,
        }),
        ["PRINTC", source] => Ok(PrintC {
            source: {
                match *source.as_bytes() {
                    [b'\\', b'\\', b'n'] => ValMemLoc::Value(b'\n'),
                    [b'\\', b'\\', b's'] => ValMemLoc::Value(b' '),
                    [b'\\', c] => ValMemLoc::Value(c),
                    _ => source.try_into()?,
                }
            },
        }),
        _ => Err("Unknown keyword"),
    }
}

fn is_comment_source(line: &str) -> bool {
    line.starts_with('#')
}

fn is_routine_source(line: &str) -> bool {
    line.ends_with(':')
}
