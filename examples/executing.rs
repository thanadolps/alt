use alt::prelude::*;
use std::fs::File;
use std::time::Instant;

fn main() {
    let compiled_file = File::open("examples/sample_compiled.bin").unwrap();
    let routines = bincode::deserialize_from::<_, Routines>(compiled_file).unwrap();
    let mut interpreter = Interpreter::new();

    let start_instant = Instant::now();
    interpreter.execute(&routines).unwrap();
    println!("Execution done in {:?}", Instant::now() - start_instant);
}
