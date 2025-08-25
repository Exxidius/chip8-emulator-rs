use sdl3::video::WindowBuildError;

#[derive(Debug)]
pub enum Chip8Error {
    RomTooLarge(usize),
    RomNotFound(String),
    InvalidOpcode(u16),
    StackOverflow,
    StackUnderflow,
    IoError(std::io::Error),
}

impl std::error::Error for Chip8Error {}

impl From<std::io::Error> for Chip8Error {
    fn from(err: std::io::Error) -> Self {
        Chip8Error::IoError(err)
    }
}

impl From<sdl3::Error> for Chip8Error {
    fn from(err: sdl3::Error) -> Self {
        Chip8Error::IoError(std::io::Error::other(err))
    }
}

impl From<WindowBuildError> for Chip8Error {
    fn from(err: WindowBuildError) -> Self {
        Chip8Error::IoError(std::io::Error::other(err))
    }
}

impl std::fmt::Display for Chip8Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Chip8Error::RomTooLarge(size) => {
                write!(f, "ROM is too large to fit in memory (size: {})", size)
            }
            Chip8Error::RomNotFound(name) => write!(f, "ROM file not found: {}", name),
            Chip8Error::InvalidOpcode(opcode) => write!(f, "Invalid opcode: {:#X}", opcode),
            Chip8Error::StackOverflow => write!(f, "Stack overflow"),
            Chip8Error::StackUnderflow => write!(f, "Stack underflow"),
            Chip8Error::IoError(err) => write!(f, "IO Error: {}", err),
        }
    }
}
