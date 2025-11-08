#[derive(Debug)]
pub enum Opcode {
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
