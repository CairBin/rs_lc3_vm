use crossterm::{event::poll, execute, style::Print};
use std::time::Duration;
use std::io::{Read};
pub fn check_key()->bool{
    poll(Duration::from_secs(0)).unwrap_or(false)
}

pub fn getchar() -> Option<char>{
    let mut buffer = [0; 1];
    match std::io::stdin().read(&mut buffer) {
        Ok(1) => Some(buffer[0] as char),
        _ => None,
    }
}

pub fn putchar(c:char) -> std::io::Result<()>{
    execute!(std::io::stdout(), Print(c))?;
    Ok(())
}