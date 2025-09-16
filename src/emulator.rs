use std::fs;
use std::thread;

use crate::error::Chip8Error;
use crate::io;

type Memory = [u8; MEMORY_SIZE];
type Display = [u8; DISPLAY_WIDTH * DISPLAY_HEIGHT];
type Regs = [u8; NUMBER_REGS];
type Stack = Vec<u16>;

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const NUMBER_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const MEMORY_SIZE: usize = 4096;
const FONT_OFFSET: usize = 0x050;
const PROGRAM_START: usize = 0x200;
const INSTRUCTION_FREQ: u64 = 1000;

pub const PAUSE: u32 = 0x02;
pub const STEP_MODE: u32 = 0x04;
pub const SHOULD_STEP: u32 = 0x08;
pub const RESET: u32 = 0x10;
pub const QUIT: u32 = 0x20;
pub const NO_KEY_PRESSED: i32 = -2;

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
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Chip8 {
    display: Display,
    memory: Memory,
    regs: Regs,
    stack: Stack,
    pc: u16,
    i: u16,
    current_instruction: u16,
    acc: i32,

    delay_timer: u8,
    sound_timer: u8,

    running: bool,
    paused: bool,
    step_mode: bool,
    should_step: bool,
    debug_mode: bool,

    io: io::IO,
}

impl Chip8 {
    pub fn new(rom: &str, debug: bool) -> Result<Self, Chip8Error> {
        let mut memory = [0; MEMORY_SIZE];
        memory[FONT_OFFSET..FONT_OFFSET + FONT.len()].copy_from_slice(&FONT);

        let data = fs::read(rom)?;
        if (data.len() + PROGRAM_START) > MEMORY_SIZE {
            return Err(Chip8Error::RomTooLarge(data.len()));
        }
        memory[PROGRAM_START..PROGRAM_START + data.len()].copy_from_slice(&data);

        Ok(Self {
            display: [0; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            memory,
            regs: [0; NUMBER_REGS],
            stack: Vec::with_capacity(STACK_SIZE),
            delay_timer: 0,
            sound_timer: 0,
            running: true,
            debug_mode: debug,
            paused: debug,
            step_mode: false,
            should_step: false,
            pc: 0x200,
            i: 0x0,
            acc: 0,
            current_instruction: 0x0000,
            io: io::IO::new(DISPLAY_WIDTH, DISPLAY_HEIGHT)?,
        })
    }

    pub fn run(&mut self) -> Result<(), Chip8Error> {
        while self.running {
            if !self.paused && (!self.step_mode || self.should_step) {
                self.handle_timer();
                self.fetch();
            }

            self.decode_execute()?;

            thread::sleep(std::time::Duration::from_secs_f64(1_f64 / INSTRUCTION_FREQ as f64));

            if self.step_mode && self.should_step {
                self.draw()?;
                self.should_step = false;
            }

            let result = self.io.poll()?;

            if result == QUIT {
                self.running = false;
                continue;
            }

            if result & PAUSE != 0 && self.debug_mode {
                self.paused = !self.paused;
                self.draw()?;
            }

            if result & STEP_MODE != 0 && self.debug_mode {
                self.step_mode = !self.step_mode;
                self.draw()?;
            }

            if result & SHOULD_STEP != 0 {
                self.should_step = true;
            }

            if result & RESET != 0 {
                self.reset()?;
            }
        }
        Ok(())
    }

    fn draw(&mut self) -> Result<(), Chip8Error> {
        println!("Drawing display");
        self.io.draw(&mut self.display)
    }

    fn reset(&mut self) -> Result<(), Chip8Error> {
        self.display = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT];
        self.regs = [0; NUMBER_REGS];

        self.pc = 0x200;
        self.i = 0x0;
        self.acc = 0;
        self.current_instruction = 0x0000;

        self.delay_timer = 0;
        self.sound_timer = 0;

        self.running = true;
        self.paused = self.debug_mode;
        self.step_mode = false;
        self.should_step = false;

        self.draw()?;
        Ok(())
    }

    fn fetch(&mut self) {
        let high_byte = self.memory[self.pc as usize] as u16;
        let low_byte = self.memory[(self.pc + 1) as usize] as u16;

        self.current_instruction = (high_byte << 8) | low_byte;
        self.pc += 2;
    }

    fn decode_execute(&mut self) -> Result<(), Chip8Error> {
        // let opcode = self.decode()?;
        // self.execute()
        Ok(())
    }

    fn decode(&self) -> Result<(), Chip8Error> {
        let first_nibble = (self.current_instruction & 0xF000) >> 12;
        let x = ((self.current_instruction & 0x0F00) >> 8) as usize;
        let y = ((self.current_instruction & 0x00F0) >> 4) as usize;
        let n = (self.current_instruction & 0x000F) as u8;
        let nn = (self.current_instruction & 0x00FF) as u8;
        let nnn = self.current_instruction & 0x0FFF;

        Ok(())
    }

    fn execute(&mut self) -> Result<(), Chip8Error> {
        // ToDo: implement opcode struct and match on it
        Ok(())
    }
    

    fn handle_timer(&mut self) {
        if self.timer_60Hz() {
            if self.delay_timer > 0 {
                self.delay_timer -= 1;
            }
            if self.sound_timer > 0 {
                self.sound_timer -= 1;
            }
        }
    }

    fn timer_60Hz(&self) -> bool {
        let now = std::time::Instant::now();
        now.elapsed().as_millis() % (1000 / 60) == 0
    }

    fn stack_push(&mut self, value: u16) -> Result<(), Chip8Error> {
        if self.stack.len() >= STACK_SIZE {
            return Err(Chip8Error::StackOverflow);
        }
        self.stack.push(value);
        Ok(())
    }

    fn stack_pop(&mut self) -> Result<u16, Chip8Error> {
        self.stack.pop().ok_or(Chip8Error::StackUnderflow)
    }
}
