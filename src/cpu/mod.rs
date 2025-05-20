pub mod opcode;
pub mod register;
mod traps;

use std::io::Write;

use crate::io::{getchar, putchar};
use crate::memory::Memory;
use opcode::OpCode;
use register::{Register, RegisterGroup};
use traps::TrapCode;

pub struct Cpu {
    reg: RegisterGroup,
    pub running: bool,
}

#[derive(Debug, Copy, Clone)]
#[repr(u16)]
pub enum FlagBit {
    POS = 1 << 0,
    ZRO = 1 << 1,
    NEG = 1 << 2,
}

// 位扩展宏，正数填0，负数填充1
macro_rules! sign_extend {
    ($x:expr, $bit_count:expr) => {{
        let value = $x as u16;
        let shift = 16 - $bit_count;
        ((value << shift) as i16 >> shift) as u16
    }};
}

macro_rules! add_mod_u16 {
    ($a:expr, $b:expr) => {{
        let a_usize = $a as usize;
        let b_usize = $b as usize;
        let sum = (a_usize % 65536 + b_usize % 65536) % 65536;
        sum as u16
    }};
}

impl Cpu {
    pub fn new() -> Cpu {
        Self {
            reg: RegisterGroup::new(),
            running: false,
        }
    }

    pub fn set_reg(&mut self, reg: Register, value: u16) {
        self.reg.write(reg, value);
    }

    pub fn get_reg(&mut self, reg: Register) -> u16 {
        self.reg.read(reg)
    }

    pub fn add_pc(&mut self) {
        self.reg.add_pc();
    }

    fn update_flags(&mut self, reg: Register) {
        let val = self.reg.read(reg);
        if val == 0 {
            self.reg.write(Register::COND, FlagBit::ZRO as u16);
        } else if (val >> 15) != 0 {
            // 最高位不为0则负数
            self.reg.write(Register::COND, FlagBit::NEG as u16);
        } else {
            // 正数
            self.reg.write(Register::COND, FlagBit::POS as u16);
        }
    }

    /******************指令集处理************************ */
    pub fn br(&mut self, ins: u16) {
        let pc_offset = sign_extend!((ins & 0x1FF), 9);
        let cond_flag = (ins >> 9) & 0x7;
        let cond_reg = self.get_reg(Register::COND);

        if (cond_flag & cond_reg) != 0 {
            let mut val = self.reg.read(Register::PC);
            val = add_mod_u16!(val, pc_offset);

            self.set_reg(Register::PC, val);
        }
    }

    pub fn add(&mut self, ins: u16) {
        let dr = Register::from_u16((ins >> 9) & 0x7).unwrap();
        let sr1 = Register::from_u16((ins >> 6) & 0x7).unwrap();
        let imm_flag = (ins >> 5) & 0x1;

        if imm_flag != 0 {
            let imm5 = sign_extend!((ins & 0x1F), 5);
            let val = add_mod_u16!(self.reg.read(sr1), imm5);
            self.set_reg(dr, val);
        } else {
            let sr2 = Register::from_u16(ins & 0x7).unwrap();
            let val = add_mod_u16!(self.reg.read(sr1), self.reg.read(sr2));
            self.reg.write(dr, val);
        }

        self.update_flags(dr);
    }

    // load
    pub fn ld(&mut self, ins: u16, mem: &mut Memory) {
        let dr = Register::from_u16((ins >> 9) & 0x7).unwrap();
        let pc_offset = sign_extend!((ins & 0x1ff), 9);
        let pc = self.reg.read(Register::PC);
        let addr = add_mod_u16!(pc, pc_offset);
        let mem_r = mem.read(addr);
        self.reg.write(dr, mem_r);
        self.update_flags(dr);
    }

    // store
    pub fn st(&mut self, ins: u16, mem: &mut Memory) {
        let dr = Register::from_u16((ins >> 9) & 0x7).unwrap();
        let pc_offset = sign_extend!((ins & 0x1ff), 9);
        let addr = add_mod_u16!(self.reg.read(Register::PC), pc_offset);
        let val = self.reg.read(dr);
        mem.write(addr, val);
    }

    pub fn jsr(&mut self, ins: u16) {
        let pc = self.reg.read(Register::PC);
        self.reg.write(Register::R7, pc);

        if ((ins >> 11) & 0x0001) == 1 {
            let pc_offset = sign_extend!((ins & 0x07FF), 11);
            let val = add_mod_u16!(pc_offset, pc);
            self.reg.write(Register::PC, val);
        } else {
            let base_reg = Register::from_u16((ins >> 6) & 0x0007).unwrap();
            let val = self.reg.read(base_reg);
            self.reg.write(Register::PC, val);
        }
    }

    pub fn and(&mut self, ins: u16) {
        // 目标寄存器
        let dis_reg = Register::from_u16((ins >> 9) & 0x0007).unwrap();
        let src1 = Register::from_u16((ins >> 6) & 0x0007).unwrap();

        if (ins & 0x0020) >= 1 {
            let imm5 = sign_extend!((ins & 0x001F), 5);
            let val = self.reg.read(src1) & imm5;
            self.reg.write(dis_reg, val);
        } else {
            let src2 = Register::from_u16(ins & 0x0007).unwrap();
            let val = self.reg.read(src1) & self.reg.read(src2);
            self.reg.write(dis_reg, val);
        }
    }

    // load register
    pub fn ldr(&mut self, ins: u16, mem: &mut Memory) {
        let dis_reg = Register::from_u16((ins >> 9) & 0x0007).unwrap();
        let base_reg = Register::from_u16((ins >> 6) & 0x0007).unwrap();

        let offset6 = sign_extend!((ins & 0x003F), 6);
        let addr = add_mod_u16!(self.reg.read(base_reg), offset6);
        let val = mem.read(addr);
        self.reg.write(dis_reg, val);
        self.update_flags(dis_reg);
    }

    // store register
    pub fn str(&mut self, ins: u16, mem: &mut Memory) {
        let sr_reg = Register::from_u16((ins >> 9) & 0x0007).unwrap();
        let base_reg = Register::from_u16((ins >> 6) & 0x0007).unwrap();

        let offset6 = sign_extend!((ins & 0x003F), 6);
        let value = self.reg.read(sr_reg);
        let addr = add_mod_u16!(self.reg.read(base_reg), offset6);
        mem.write(addr, value);
    }

    pub fn not(&mut self, ins: u16) {
        let dr = Register::from_u16((ins >> 9) & 0x7).unwrap();
        let sr = Register::from_u16((ins >> 6) & 0x7).unwrap();

        let sr_val = self.reg.read(sr);
        self.reg.write(dr, !sr_val);
        self.update_flags(dr);
    }

    pub fn ldi(&mut self, ins: u16, mem: &mut Memory) {
        let dr = Register::from_u16((ins >> 9) & 0x7).unwrap();
        let pc_offset = sign_extend!((ins & 0x1FF), 9);
        let addr = add_mod_u16!(self.reg.read(Register::PC), pc_offset);
        let mut value = mem.read(addr);
        value = mem.read(value);
        self.reg.write(dr, value);
        self.update_flags(dr);
    }

    pub fn sti(&mut self, ins: u16, mem: &mut Memory) {
        let sr = Register::from_u16((ins >> 9) & 0x7).unwrap();
        let pc_offset = sign_extend!((ins & 0x1FF), 9);
        let mut addr = add_mod_u16!(self.reg.read(Register::PC), pc_offset);
        addr = mem.read(addr);
        let value = self.reg.read(sr);
        mem.write(addr, value);
    }

    pub fn jmp(&mut self, ins: u16) {
        let br = Register::from_u16((ins >> 6) & 0x7).unwrap();
        let value = self.reg.read(br);
        self.reg.write(Register::PC, value);
    }

    pub fn lea(&mut self, ins: u16) {
        let dr = Register::from_u16((ins >> 9) & 0x7).unwrap();
        let pc_offset = sign_extend!((ins & 0x1FF), 9);
        let addr = add_mod_u16!(pc_offset, self.reg.read(Register::PC));
        self.reg.write(dr, addr);
        self.update_flags(dr);
    }

    pub fn trap(&mut self, ins: u16, mem: &mut Memory) {
        //println!("{:?}",TrapCode::from_u16(ins & 0xFF));
        match TrapCode::from_u16(ins & 0xFF) {
            Some(TrapCode::GETC) => self.trap_getc(),
            Some(TrapCode::OUT) => self.trap_out(),
            Some(TrapCode::IN) => self.trap_in(),
            Some(TrapCode::PUTS) => self.trap_puts(mem),
            Some(TrapCode::PUTSP) => self.trap_putsp(mem),
            Some(TrapCode::HALT) => {
                println!("HALT");
                self.running = false;
            }
            _ => {
                panic!("Unexpected trap code!");
            }
        }
    }

    fn trap_getc(&mut self) {
        let c = getchar().unwrap();
        self.reg.write(Register::R0, c as u16);
    }

    fn trap_out(&mut self) {
        let val = self.reg.read(Register::R0) as u8 as char;
        putchar(val).unwrap();
    }

    fn trap_puts(&mut self, mem: &mut Memory) {
        let mut addr = self.reg.read(Register::R0); // 起始地址
        loop {
            let ch = mem.read(addr); // 读取内存中的字符
            if ch == 0 {
                break; // 遇到 NULL 终止符时停止
            }
            print!("{}", ch as u8 as char); // 输出字符
            addr += 1;
        }
        std::io::stdout().flush().unwrap(); // 强制刷新输出缓冲区
    }

    fn trap_putsp(&mut self, mem: &mut Memory) {
        let mut addr = self.reg.read(Register::R0); // 起始地址
        loop {
            let word = mem.read(addr); // 读取一个 u16
            if word == 0 {
                break; // 遇到 NULL 终止符时停止
            }

            // 拆解低字节和高字节
            let low_byte = (word & 0xFF) as u8;
            let high_byte = (word >> 8) as u8;

            // 输出低字节字符（必须非零）
            print!("{}", low_byte as char);

            // 如果高字节非零，也输出
            if high_byte != 0 {
                print!("{}", high_byte as char);
            }

            addr += 1;
        }
        std::io::stdout().flush().unwrap(); // 强制刷新输出缓冲区
    }

    fn trap_in(&mut self) {
        print!("Enter a character: ");
        std::io::stdout().flush().unwrap();
        let c = getchar().unwrap();
        self.reg.write(Register::R0, c as u16);
    }

    pub fn rti(&mut self) {
        panic!("Instruction is not implemented!");
    }

    pub fn res(&mut self) {
        return;
    }

    pub fn op(&mut self, ins: u16, mem: &mut Memory) {
        // opcode selector
        //println!("{:?}",OpCode::from_u16(ins >> 12));
        match OpCode::from_u16(ins >> 12) {
            Some(OpCode::BR) => self.br(ins),
            Some(OpCode::ADD) => self.add(ins),
            Some(OpCode::LD) => self.ld(ins, mem),
            Some(OpCode::ST) => self.st(ins, mem),
            Some(OpCode::JSR) => self.jsr(ins),
            Some(OpCode::AND) => self.and(ins),
            Some(OpCode::LDR) => self.ldr(ins, mem),
            Some(OpCode::STR) => self.str(ins, mem),
            Some(OpCode::RTI) => self.rti(),
            Some(OpCode::NOT) => self.not(ins),
            Some(OpCode::LDI) => self.ldi(ins, mem),
            Some(OpCode::STI) => self.sti(ins, mem),
            Some(OpCode::JMP) => self.jmp(ins),
            Some(OpCode::RES) => self.res(),
            Some(OpCode::LEA) => self.lea(ins),
            Some(OpCode::TRAP) => self.trap(ins, mem),
            None => {
                panic!("Unexpected operation code")
            }
        };
    }
}
