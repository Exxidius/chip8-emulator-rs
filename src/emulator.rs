use std::fs;
use std::thread;
use rand::Rng;

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
const TIMER_FREQ: u64 = 60;

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
    last_timer_update: std::time::Instant,

    running: bool,
    paused: bool,
    step_mode: bool,
    should_step: bool,
    debug_mode: bool,

    io: Option<io::IO>,
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
            last_timer_update: std::time::Instant::now(),
            running: true,
            debug_mode: debug,
            paused: debug,
            step_mode: false,
            should_step: false,
            pc: 0x200,
            i: 0x0,
            acc: 0,
            current_instruction: 0x0000,
            io: Some(io::IO::new(DISPLAY_WIDTH, DISPLAY_HEIGHT)?),
        })
    }

    pub fn run(&mut self) -> Result<(), Chip8Error> {
        while self.running {
            if !self.paused && (!self.step_mode || self.should_step) {
                self.handle_timer();

                if self.pc as usize >= MEMORY_SIZE - 1 {
                    return Err(Chip8Error::PCOutOfBounds(self.pc));
                }

                self.fetch();
            }

            self.decode_execute()?;

            thread::sleep(std::time::Duration::from_secs_f64(
                1_f64 / INSTRUCTION_FREQ as f64,
            ));

            if self.step_mode && self.should_step {
                self.draw()?;
                self.should_step = false;
            }

            if let Some(io) = &mut self.io {
                let result = io.poll()?;

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
        }
        Ok(())
    }

    fn draw(&mut self) -> Result<(), Chip8Error> {
        if let Some(io) = &mut self.io {
            println!("Drawing display");
            io.draw(&mut self.display)?;
        }
        Ok(())
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
        let opcode = self.decode()?;
        self.execute(opcode)?;
        Ok(())
    }

    fn decode(&self) -> Result<Opcode, Chip8Error> {
        let first_nibble = (self.current_instruction & 0xF000) >> 12;
        let x = ((self.current_instruction & 0x0F00) >> 8) as u8;
        let y = ((self.current_instruction & 0x00F0) >> 4) as u8;
        let n = (self.current_instruction & 0x000F) as u8;
        let nn = (self.current_instruction & 0x00FF) as u8;
        let nnn = self.current_instruction & 0x0FFF;

        match (first_nibble, x, y, n) {
            (0x0, 0x0, 0xE, 0x0) => Ok(Opcode::Clear),
            (0x0, 0x0, 0xE, 0xE) => Ok(Opcode::Return),
            (0x1, _, _, _) => Ok(Opcode::Jump(nnn)),
            (0x2, _, _, _) => Ok(Opcode::Call(nnn)),
            (0x3, _, _, _) => Ok(Opcode::SkipEqualVal(x, nn)),
            (0x4, _, _, _) => Ok(Opcode::SkipNotEqualVal(x, nn)),
            (0x5, _, _, 0x0) => Ok(Opcode::SkipEqual(x, y)),
            (0x6, _, _, _) => Ok(Opcode::SetVal(x, nn)),
            (0x7, _, _, _) => Ok(Opcode::AddVal(x, nn)),
            (0x8, _, _, 0x0) => Ok(Opcode::Set(x, y)),
            (0x8, _, _, 0x1) => Ok(Opcode::Or(x, y)),
            (0x8, _, _, 0x2) => Ok(Opcode::And(x, y)),
            (0x8, _, _, 0x3) => Ok(Opcode::Xor(x, y)),
            (0x8, _, _, 0x4) => Ok(Opcode::Add(x, y)),
            (0x8, _, _, 0x5) => Ok(Opcode::SubY(x, y)),
            (0x8, _, _, 0x6) => Ok(Opcode::ShiftRight(x)),
            (0x8, _, _, 0x7) => Ok(Opcode::SubX(x, y)),
            (0x8, _, _, 0xE) => Ok(Opcode::ShiftLeft(x)),
            (0x9, _, _, 0x0) => Ok(Opcode::SkipNotEqual(x, y)),
            (0xA, _, _, _) => Ok(Opcode::SetI(nnn)),
            (0xB, _, _, _) => Ok(Opcode::JumpV0(nnn)),
            (0xC, _, _, _) => Ok(Opcode::Random(x, nn)),
            (0xD, _, _, _) => Ok(Opcode::Draw(x, y, n)),
            (0xE, _, 0x9, 0xE) => Ok(Opcode::SkipKey(x)),
            (0xE, _, 0xA, 0x1) => Ok(Opcode::SkipNotKey(x)),
            (0xF, _, 0x0, 0x7) => Ok(Opcode::GetDelay(x)),
            (0xF, _, 0x0, 0xA) => Ok(Opcode::WaitKey(x)),
            (0xF, _, 0x1, 0x5) => Ok(Opcode::SetDelay(x)),
            (0xF, _, 0x1, 0x8) => Ok(Opcode::SetSound(x)),
            (0xF, _, 0x1, 0xE) => Ok(Opcode::AddI(x)),
            (0xF, _, 0x2, 0x9) => Ok(Opcode::SetSprite(x)),
            (0xF, _, 0x3, 0x3) => Ok(Opcode::StoreBCD(x)),
            (0xF, _, 0x5, 0x5) => Ok(Opcode::StoreRegs(x)),
            (0xF, _, 0x6, 0x5) => Ok(Opcode::LoadRegs(x)),
            _ => Err(Chip8Error::InvalidOpcode(self.current_instruction)),
        }
    }

    fn execute(&mut self, opcode: Opcode) -> Result<(), Chip8Error> {
        match opcode {
            Opcode::Clear => {
                self.display = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT];
                if let Some(io) = &mut self.io {
                    io.draw(&mut self.display)?;
                }
                Ok(())
            }
            Opcode::Return => {
                self.pc = self.stack_pop()?;
                Ok(())
            }
            Opcode::Jump(addr) => {
                self.pc = addr;
                Ok(())
            }
            Opcode::Call(addr) => {
                self.stack_push(self.pc)?;
                self.pc = addr;
                Ok(())
            }
            Opcode::SkipEqualVal(x, nn) => {
                if self.regs[x as usize] == nn {
                    self.pc += 2;
                }
                Ok(())
            }
            Opcode::SkipNotEqualVal(x, nn) => {
                if self.regs[x as usize] != nn {
                    self.pc += 2;
                }
                Ok(())
            }
            Opcode::SkipEqual(x, y) => {
                if self.regs[x as usize] == self.regs[y as usize] {
                    self.pc += 2;
                }
                Ok(())
            }
            Opcode::SetVal(x, nn) => {
                self.regs[x as usize] = nn;
                Ok(())
            }
            Opcode::AddVal(x, nn) => {
                self.regs[x as usize] = self.regs[x as usize].wrapping_add(nn);
                Ok(())
            }
            Opcode::Set(x, y) => {
                self.regs[x as usize] = self.regs[y as usize];
                Ok(())
            }
            Opcode::Or(x, y) => {
                self.regs[x as usize] |= self.regs[y as usize];
                Ok(())
            }
            Opcode::And(x, y) => {
                self.regs[x as usize] &= self.regs[y as usize];
                Ok(())
            }
            Opcode::Xor(x, y) => {
                self.regs[x as usize] ^= self.regs[y as usize];
                Ok(())
            }
            Opcode::Add(x, y) => {
                let (result, overflow) =
                    self.regs[x as usize].overflowing_add(self.regs[y as usize]);
                self.regs[x as usize] = result;
                self.regs[0xF] = overflow as u8;
                Ok(())
            }
            Opcode::SubY(x, y) => {
                let (result, underflow) =
                    self.regs[x as usize].overflowing_sub(self.regs[y as usize]);
                self.regs[x as usize] = result;
                self.regs[0xF] = !underflow as u8;
                Ok(())
            }
            Opcode::ShiftRight(x) => {
                let acc = self.regs[x as usize];
                self.regs[x as usize] >>= 1;
                self.regs[0xF] = acc & 0x1;
                Ok(())
            }
            Opcode::SubX(x, y) => {
                let (result, underflow) =
                    self.regs[y as usize].overflowing_sub(self.regs[x as usize]);
                self.regs[x as usize] = result;
                self.regs[0xF] = !underflow as u8;
                Ok(())
            }
            Opcode::ShiftLeft(x) => {
                let acc = self.regs[x as usize];
                self.regs[x as usize] <<= 1;
                self.regs[0xF] = (acc >> 7) & 0x1;
                Ok(())
            }
            Opcode::SkipNotEqual(x, y) => {
                if self.regs[x as usize] != self.regs[y as usize] {
                    self.pc += 2;
                }
                Ok(())
            }
            Opcode::SetI(addr) => {
                self.i = addr;
                Ok(())
            }
            Opcode::JumpV0(nnn) => {
                self.pc = (self.regs[0x0] + (nnn as u8)) as u16;
                Ok(())
            }
            Opcode::Random(x, nn) => {
                let mut rng = rand::rng();
                let random_number: u8 = rng.random();
                self.regs[x as usize] = random_number & nn;
                Ok(())
            }
            Opcode::Draw(x, y, n) => {
                let vx = self.regs[x as usize] as usize % DISPLAY_WIDTH;
                let vy = self.regs[y as usize] as usize % DISPLAY_HEIGHT;

                self.display(vx, vy, n);

                if let Some(io) = &mut self.io {
                    io.draw(&mut self.display)?;
                }
                Ok(())
            }
            // TODO: refactor SkipKey and SkipNotKey
            Opcode::SkipKey(x) => {
                if x > 0xF {
                    return Err(Chip8Error::StackUnderflow);
                }
                if let Some(io) = &mut self.io {
                    let result = io.check_key_pressed(self.regs[x as usize]);

                    if result == true {
                        self.pc += 2;
                    }
                }
                Ok(())
            }
            Opcode::SkipNotKey(x) => {
                if x > 0xF {
                    return Err(Chip8Error::StackOverflow);
                }
                if let Some(io) = &mut self.io {
                    let result = io.check_key_pressed(self.regs[x as usize]);

                    if result == false {
                        self.pc += 2;
                    }
                }
                Ok(())
            }
            Opcode::GetDelay(x) => {
                self.regs[x as usize] = self.delay_timer;
                Ok(())
            }
            Opcode::WaitKey(x) => {
                if let Some(io) = &mut self.io {
                    let result = io.get_key_pressed();
                    if result == NO_KEY_PRESSED {
                        self.pc -= 2;
                    }
                    else {
                        self.regs[x as usize] = result as u8;
                    }
                }
                Ok(())
            }
            Opcode::SetDelay(x) => {
                self.delay_timer = self.regs[x as usize];
                Ok(())
            }
            Opcode::SetSound(x) => {
                self.sound_timer = self.regs[x as usize];
                Ok(())
            }
            Opcode::AddI(x) => {
                self.i += self.regs[x as usize] as u16;
                Ok(())
            }
            Opcode::SetSprite(x) => {
                let value = (self.regs[(x as usize) & 0xF] * 5) as u16;
                self.i = MEMORY_SIZE as u16 + value;
                Ok(())
            }
            Opcode::StoreBCD(x) => {
                self.store_bcd(x);
                Ok(())
            }
            Opcode::StoreRegs(x) => {
                self.store_regs(x as u16);
                Ok(())
            }
            Opcode::LoadRegs(x) => {
                self.load_regs(x as u16);
                Ok(())
            }
        }
    }

    fn store_bcd(&mut self, x: u8) {
        self.memory[self.i as usize] = (self.regs[x as usize] / 100) % 10;
        self.memory[(self.i + 1) as usize] = (self.regs[x as usize] / 10) % 10;
        self.memory[(self.i + 2) as usize] = self.regs[x as usize] % 10;
    }

    fn store_regs(&mut self, x: u16) {
        for i in 0u16..=x {
            self.memory[(self.i + i) as usize] = self.regs[i as usize];
        }
    }

    fn load_regs(&mut self, x: u16) {
        for i in 0u16..=x {
            self.regs[i as usize] = self.memory[(self.i + i) as usize];
        }
    }

    fn handle_timer(&mut self) {
        if self.timer_60_hz() {
            if self.delay_timer > 0 {
                self.delay_timer -= 1;
            }
            if self.sound_timer > 0 {
                self.sound_timer -= 1;
            }
        }
    }

    fn timer_60_hz(&mut self) -> bool {
        let now = std::time::Instant::now();
        let diff = now.duration_since(self.last_timer_update);
        let update_rate_ms = 1000_f64 / TIMER_FREQ as f64;
        if diff.as_millis() > update_rate_ms as u128 {
            self.last_timer_update = now;
            return true;
        }
        false
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

    fn display(&mut self, vx: usize, vy: usize, n: u8) {
        self.regs[0xF] = 0;

        for byte_index in 0..n as usize {
            let byte = self.memory[self.i as usize + byte_index];
            for bit_index in (0..8).rev() {
                let bit = (byte >> bit_index) & 1;
                let screen_x = (vx + (7 - bit_index)) % DISPLAY_WIDTH;
                let screen_y = (vy + byte_index) % DISPLAY_HEIGHT;
                let screen_offset = screen_y * DISPLAY_WIDTH + screen_x;

                if bit == 1 && self.display[screen_offset] == 1 {
                    self.regs[0xF] = 1;
                }

                self.display[screen_offset] ^= bit;

                if screen_x == DISPLAY_WIDTH - 1 {
                    break;
                }
            }

            if vy + byte_index == DISPLAY_HEIGHT - 1 {
                break;
            }
        }
    }
}

#[derive(Debug)]
enum Opcode {
    Clear,                   // 00E0
    Return,                  // 00EE
    Jump(u16),               // 1NNN
    Call(u16),               // 2NNN
    SkipEqualVal(u8, u8),    // 3XNN
    SkipNotEqualVal(u8, u8), // 4XNN
    SkipEqual(u8, u8),       // 5XY0
    SetVal(u8, u8),          // 6XNN
    AddVal(u8, u8),          // 7XNN
    Set(u8, u8),             // 8XY0
    Or(u8, u8),              // 8XY1
    And(u8, u8),             // 8XY2
    Xor(u8, u8),             // 8XY3
    Add(u8, u8),             // 8XY4
    SubY(u8, u8),            // 8XY5
    ShiftRight(u8),          // 8XY6
    SubX(u8, u8),            // 8XY7
    ShiftLeft(u8),           // 8XYE
    SkipNotEqual(u8, u8),    // 9XY0
    SetI(u16),               // ANNN
    JumpV0(u16),             // BNNN
    Random(u8, u8),          // CXNN
    Draw(u8, u8, u8),        // DXYN
    SkipKey(u8),             // EX9E
    SkipNotKey(u8),          // EXA1
    GetDelay(u8),            // FX07
    WaitKey(u8),             // FX0A
    SetDelay(u8),            // FX15
    SetSound(u8),            // FX18
    AddI(u8),                // FX1E
    SetSprite(u8),           // FX29
    StoreBCD(u8),            // FX33
    StoreRegs(u8),           // FX55
    LoadRegs(u8),            // FX65
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_headless_chip8() -> Chip8 {
        let mut memory = [0; MEMORY_SIZE];
        memory[FONT_OFFSET..FONT_OFFSET + FONT.len()].copy_from_slice(&FONT);

        Chip8 {
            display: [0; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            memory,
            regs: [0; NUMBER_REGS],
            stack: Vec::with_capacity(STACK_SIZE),
            delay_timer: 0,
            sound_timer: 0,
            last_timer_update: std::time::Instant::now(),
            running: true,
            debug_mode: false,
            paused: false,
            step_mode: false,
            should_step: false,
            pc: PROGRAM_START as u16,
            i: 0x0,
            acc: 0,
            current_instruction: 0x0000,
            io: None,
        }
    }

    #[test]
    fn test_opcode_add() {
        let mut chip8 = new_headless_chip8();
        chip8.regs[0xF] = 200;
        chip8.current_instruction = 0x7F64;

        let opcode = chip8.decode().unwrap();
        chip8.execute(opcode).unwrap();

        // no overflow in this instruction
        assert_eq!(chip8.regs[0xF], 44);

        chip8.regs[0] = 20;
        chip8.current_instruction = 0x7064;

        let opcode = chip8.decode().unwrap();
        chip8.execute(opcode).unwrap();

        assert_eq!(chip8.regs[0], 120);
    }

    #[test]
    fn test_opcode_sub_y() {
        let mut chip8 = new_headless_chip8();
        chip8.regs[0xF] = 10;
        chip8.current_instruction = 0x8FF7;

        let opcode = chip8.decode().unwrap();
        chip8.execute(opcode).unwrap();

        assert_eq!(chip8.regs[0xF], 1);

        chip8.regs[0x1] = 10;
        chip8.regs[0x2] = 15;
        chip8.current_instruction = 0x8125;

        let opcode = chip8.decode().unwrap();
        chip8.execute(opcode).unwrap();

        assert_eq!(chip8.regs[1], 251);
        assert_eq!(chip8.regs[0xF], 0);
    }

    #[test]
    fn test_opcode_clear() {
        let mut chip8 = new_headless_chip8();
        chip8.display = [1; DISPLAY_WIDTH * DISPLAY_HEIGHT];
        chip8.current_instruction = 0x00E0;

        let opcode = chip8.decode().unwrap();
        chip8.execute(opcode).unwrap();

        assert_eq!(chip8.display, [0; DISPLAY_WIDTH * DISPLAY_HEIGHT]);
    }

    #[test]
    fn test_opcode_jump() {
        let mut chip8 = new_headless_chip8();
        chip8.current_instruction = 0x1234;

        let opcode = chip8.decode().unwrap();
        chip8.execute(opcode).unwrap();

        assert_eq!(chip8.pc, 0x234);
    }

    #[test]
    fn test_opcode_store_bcd() {
        let mut chip8 = new_headless_chip8();
        chip8.i = 0x300;
        chip8.regs[6] = 137;
        chip8.current_instruction = 0xF633;

        let opcode = chip8.decode().unwrap();
        chip8.execute(opcode).unwrap();

        assert_eq!(chip8.memory[0x300], 1);
        assert_eq!(chip8.memory[0x301], 3);
        assert_eq!(chip8.memory[0x302], 7);

        chip8.regs[6] = 65;
        chip8.current_instruction = 0xF633;

        let opcode = chip8.decode().unwrap();
        chip8.execute(opcode).unwrap();

        assert_eq!(chip8.memory[0x300], 0);
        assert_eq!(chip8.memory[0x301], 6);
        assert_eq!(chip8.memory[0x302], 5);

        chip8.regs[6] = 4;
        chip8.current_instruction = 0xF633;

        let opcode = chip8.decode().unwrap();
        chip8.execute(opcode).unwrap();

        assert_eq!(chip8.memory[0x300], 0);
        assert_eq!(chip8.memory[0x301], 0);
        assert_eq!(chip8.memory[0x302], 4);
    }

    #[test]
    fn test_opcode_store_regs_detailed() {
        let mut chip8 = new_headless_chip8();

        chip8.regs[0] = 0;
        chip8.regs[1] = 48;

        let scratchpad = 0x300;
        chip8.i = scratchpad;

        chip8.current_instruction = 0xF155;
        let opcode = chip8.decode().unwrap();
        chip8.execute(opcode).unwrap();

        assert_eq!(chip8.memory[scratchpad as usize], 0, "Memory[I] should be 0");
        assert_eq!(chip8.memory[(scratchpad + 1) as usize], 48, "Memory[I+1] should be 48");

        chip8.regs[0] = 0xFF;
        chip8.regs[1] = 0xFF;

        chip8.i = scratchpad;
        chip8.current_instruction = 0xF065;
        let opcode = chip8.decode().unwrap();
        chip8.execute(opcode).unwrap();

        let v1_temp = chip8.regs[0];

        chip8.i = scratchpad + 1;
        chip8.current_instruction = 0xF065;
        let opcode = chip8.decode().unwrap();
        chip8.execute(opcode).unwrap();

        assert_eq!(chip8.regs[0], 48, "v0 should be 48 after loading from scratchpad+1");
        assert_eq!(v1_temp, 0, "v1_temp should be 0 after loading from scratchpad");
    }

     #[test]
    fn test_opcode_load_regs() {
        let mut chip8 = new_headless_chip8();
        chip8.i = 0x300;
        for i in 0..=5 {
            chip8.memory[0x300 + i] = i as u8 * 10;
        }
        chip8.current_instruction = 0xF565;

        let opcode = chip8.decode().unwrap();
        chip8.execute(opcode).unwrap();

        for i in 0..=5 {
            assert_eq!(chip8.regs[i], i as u8 * 10);
        }
    }
}
