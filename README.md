# Chip8 Emulator (Rust Version)

A Chip8 emulator (re)written in rust to learn/practice rust.

> **_NOTE:_**  In the rust version there is no debug UI. If the debug flag is passed the 
debug buttons still work. Whenever I find the time I will include the debug UI aswell.

## Controlling the emulator
You can pause and continue the emulation with the P button.
The M button toggles step mode.
With N you can advance one instruction when in step mode.
Pause mode and step mode are only available when debug mode is active.

The 0 button resets the emulator and the loaded rom will start from the beginning.

When launching the emulator in debug mode, the pause mode is activated by default.

## Resources (Thanks to the authors for providing these!)
   + [Tobias V. Langhoff's high level Chip8 guide](https://tobiasvl.github.io/blog/write-a-chip-8-emulator)
   + [Timendus Chip8 test roms](https://github.com/Timendus/chip8-test-suite?tab=readme-ov-file)
   + [Corax Chip8 test roms](https://github.com/corax89/chip8-test-rom)

