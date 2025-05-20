

#[derive(Debug, Copy, Clone)]
#[repr(u16)]
pub enum TrapCode{
    GETC = 0x20,
    OUT = 0x21,
    PUTS = 0x22,
    IN = 0x23,
    PUTSP = 0x24,
    HALT = 0x25
}

impl TrapCode{
    pub fn from_u16(val:u16)->Option<TrapCode>{
        match val{
            0x20 => Some(TrapCode::GETC),
            0x21 => Some(TrapCode::OUT),
            0x22 => Some(TrapCode::PUTS),
            0x23 => Some(TrapCode::IN),
            0x24 =>Some(TrapCode::PUTSP),
            0x25 => Some(TrapCode::HALT),
            _=>None
        }
    }
}