use crate::io;

pub struct Chip8 {}

impl Chip8 {
    pub fn new() -> Self {
        io::init();
        Chip8 {}
    }

    pub fn run(&mut self) -> () {
        ()
    }
}
