use crate::io;

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const MEMORY_SIZE: usize = 4096;
const NUMBER_REGS: usize = 16;

pub struct Chip8 {
    display: [u8; DISPLAY_WIDTH * DISPLAY_HEIGHT],
    memory: [u8; MEMORY_SIZE],
    regs: [u8; NUMBER_REGS],
    pc: u16,
    i: u16,
    current_instruction: u16,
    acc: i32,

    delay_timer: u8,
    sound_timer: u8,

    running: bool,
    paused: bool,
    step_mode: bool,

    // TODO: we want the file
    rom_file_name: String,
    io: io::IO,
}

impl Chip8 {
    pub fn new() -> Self {
        let emu = Chip8 {
            display: [0; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            memory: [0; MEMORY_SIZE],
            regs: [0; NUMBER_REGS],
            delay_timer: 0,
            sound_timer: 0,
            running: true,
            paused: false,
            step_mode: false,
            rom_file_name: String::from("test"),
            pc: 0x0,
            i: 0x0,
            acc: 0,
            current_instruction: 0,
            io: io::IO::new()
        };

        emu
    }

    pub fn run(&mut self) -> () {
        ()
    }
}
