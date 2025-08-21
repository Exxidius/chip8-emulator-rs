use clap::Parser;

mod emulator;
mod io;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    rom: String,

    #[arg(short, long, default_value_t = false)]
    debug: bool,
}

fn main() {
    let args = Args::parse();

    println!("{:?}", args);

    let _emu = emulator::Chip8::new();
}
