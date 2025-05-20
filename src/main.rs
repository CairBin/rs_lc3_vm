use std::{env, path::Path};

use vm::Vm;

pub mod io;
pub mod memory;
pub mod cpu;
pub mod vm;

fn main() {
    let args:Vec<String> = env::args().collect();
    if args.len() < 2{
        println!("Usage: lc3 [image-file]...");
        return;
    }

    let mut vm = Vm::new();
    for arg in args.iter().skip(1){
        let path = Path::new(arg);
        if let Err(err) = vm.load_image(&path) {
            println!("Error loading image: {}", err);
            return;
        }
        println!("loading file {}", arg);
    }
    vm.loop_run();
}
