use crate::io;

pub struct Chip8 {}

impl Chip8 {
    pub fn new() -> Self {
        io::init();
        Chip8 {}
    }

    // Use run as loop is reserved (and i dont want to escape)
    pub fn run() -> () {
        ()
    }
}
