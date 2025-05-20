use std::path::Path;
use crate::{cpu::{register::Register, Cpu}, memory::Memory};

pub struct Vm{
    cpu: Cpu,
    memory: Memory,
}

impl Vm{
    pub fn new()->Self{
        Self{
            cpu: Cpu::new(),
            memory: Memory::new()
        }
    }

    pub fn load_image(&mut self, path: &Path) -> std::io::Result<()> {
        let mut file = std::fs::File::open(path)?;
        self.memory.load_img(&mut file)
    }

    pub fn loop_run(&mut self){
        self.cpu.running = true;
        crossterm::terminal::enable_raw_mode().unwrap();
        while self.cpu.running{
            let pc = self.cpu.get_reg(Register::PC);
            let ins = self.memory.read(pc);
            //println!("PC: 0x{:04X}, Ins: 0x{:04X}", pc, ins);
            self.cpu.add_pc();
            self.cpu.op(ins, &mut self.memory);
            //std::thread::sleep(std::time::Duration::from_secs(1));
        }

        crossterm::terminal::disable_raw_mode().unwrap();
    }
}