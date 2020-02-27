use std::time::Instant;
use asm_like_lang::prelude::Routines;
use std::fs::{File, OpenOptions};
use std::io::Read;

fn main() {
    let start_instant = Instant::now();
    let routines = {
        let mut routines = Routines::new();
        let mut file = File::open(r"examples/sample_code.txt").unwrap();
        // TODO: try to line by line?
        let mut code = String::new();
        file.read_to_string(&mut code).unwrap();
        routines.parse(code.lines()).unwrap();
        routines
    };
    println!("Parsing done in {:?} using {} bytes", Instant::now() - start_instant, bincode::serialized_size(&routines).unwrap());

    println!("Output to examples/sample_compiled.bin");
    let mut output_file =
        OpenOptions::new().write(true).create(true).open("examples/sample_compiled.bin").unwrap();
    bincode::serialize_into(output_file, &routines).unwrap();
}