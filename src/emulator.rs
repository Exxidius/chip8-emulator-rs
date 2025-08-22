use std::fs;
use std::error::Error;
use crate::io;

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const NUMBER_REGS: usize = 16;

const MEMORY_SIZE: usize = 4096;
const FONT_OFFSET: usize = 0x050;
const PROGRAM_START: usize = 0x200;
const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

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
    debug_mode: bool,

    rom_file_name: String,
    io: io::IO,
}

impl Chip8 {
    pub fn new(rom: &str, debug: bool) -> Result<Self, Box<dyn Error>> {
        let mut memory = [0; MEMORY_SIZE];

        memory[FONT_OFFSET..FONT_OFFSET + FONT.len()]
            .copy_from_slice(&FONT);

        let rom_data = fs::read(rom)?;

        if (rom_data.len() + PROGRAM_START) > MEMORY_SIZE {
            return Err("Rom is too large to fit in chip8 memory".into());
        }

        memory[PROGRAM_START..PROGRAM_START + rom_data.len()]
            .copy_from_slice(&rom_data);

        Ok(Chip8 {
            display: [0; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            memory,
            regs: [0; NUMBER_REGS],
            delay_timer: 0,
            sound_timer: 0,
            running: true,
            debug_mode: debug,
            paused: if debug { true } else { false },
            step_mode: false,
            rom_file_name: String::from(rom),
            pc: 0x200,
            i: 0x0,
            acc: 0,
            current_instruction: 0x0000,
            io: io::IO::new()?,
        })
    }

    pub fn run(&mut self) {}
}
