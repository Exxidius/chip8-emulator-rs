use clap::Parser;

mod emulator;
mod error;
mod io;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Rom file to emulate
    #[arg(short, long, value_name = "ROM-FILE")]
    rom: String,

    /// Enables debug mode
    #[arg(short, long, default_value_t = false)]
    debug: bool,
}

fn main() -> Result<(), error::Chip8Error> {
    let args = Args::parse();
    emulator::Chip8::new(args.rom.as_str(), args.debug)?.run()?;
    Ok(())
}
