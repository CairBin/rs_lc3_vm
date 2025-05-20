use std::{fs::File, io::Read};

use crate::io::{check_key, getchar};

const KBSR: u16 = 0xFE00;
const KBDR: u16 = 0xFE02;

pub const MEMORY_SIZE: usize = 65536;

pub struct Memory {
    data: [u16; MEMORY_SIZE as usize],
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            data: [0; MEMORY_SIZE as usize],
        }
    }

    pub fn write(&mut self, addr: u16, value: u16) {
        self.data[addr as usize] = value;
    }

    pub fn read(&mut self, addr: u16) -> u16 {
        if addr == KBSR {
            if check_key() {
                self.write(KBSR, (1 << 15) as u16);
                if let Some(ch) = getchar() {
                    self.write(KBDR, ch as u16);
                }
            }
        } else {
            self.write(KBSR, 0);
        }

        return self.data[addr as usize];
    }

    pub fn load_img(&mut self, file: &mut File) -> std::io::Result<()> {
        let mut origin_bytes = [0u8; 2];
        file.read_exact(&mut origin_bytes)?;
        let origin = u16::from_be_bytes(origin_bytes) as usize;

        // 检查地址是否合法
        if origin >= MEMORY_SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Origin address 0x{:X} exceeds memory size", origin),
            ));
        }

        // 读取剩余数据（按大端序解析）
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        // 确保字节数是偶数
        if buffer.len() % 2 != 0 {
            buffer.push(0);
        }

        // 按字（2字节）写入内存
        for (i, chunk) in buffer.chunks_exact(2).enumerate() {
            let addr = origin + i;
            if addr >= MEMORY_SIZE {
                break; // 超出内存范围时停止
            }
            let word = u16::from_be_bytes([chunk[0], chunk[1]]);
            self.data[addr] = word;
        }

        Ok(())
    }
}
