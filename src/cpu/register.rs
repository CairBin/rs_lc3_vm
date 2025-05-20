
pub const PC_START:u16 = 0x3000;

#[derive(Debug, Copy, Clone)]
#[repr(usize)]
pub enum Register{
    R0 = 0,
    R1 = 1,
    R2 = 2,
    R3 = 3,
    R4 = 4,
    R5 = 5,
    R6 = 6,
    R7 = 7,
    PC = 8,
    COND = 9,
    COUNT = 10, // 寄存器数量
}

impl Register{
    pub fn from_usize(val:usize)->Option<Register>{
        match val{
            0 => Some(Register::R0),
            1 => Some(Register::R1),
            2 => Some(Register::R2),
            3 => Some(Register::R3),
            4 => Some(Register::R4),
            5 => Some(Register::R5),
            6 => Some(Register::R6),
            7 => Some(Register::R7),
            8 => Some(Register::PC),
            9 => Some(Register::COND),
            _ => None
        }
    }

    pub fn from_u16(val:u16)->Option<Register>{
        match val{
            0 => Some(Register::R0),
            1 => Some(Register::R1),
            2 => Some(Register::R2),
            3 => Some(Register::R3),
            4 => Some(Register::R4),
            5 => Some(Register::R5),
            6 => Some(Register::R6),
            7 => Some(Register::R7),
            8 => Some(Register::PC),
            9 => Some(Register::COND),
            _ => None
        }
    }
}

pub struct RegisterGroup{
    reg:[u16; Register::COUNT as usize],
}

impl RegisterGroup{
    pub fn new()->Self{
        let mut regs = Self{reg:[0; Register::COUNT as usize]};
        regs.write(Register::PC, PC_START);
        regs
    } 

    pub fn write(&mut self, reg:Register, value:u16){
        self.reg[reg as usize] = value;
    }

    pub fn read(&mut self, reg:Register)->u16{
        self.reg[reg as usize]
    }

    pub fn add_pc(&mut self){
        self.reg[Register::PC as usize] += 1;
    }
}