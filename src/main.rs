use clap::Parser;

mod emulator;
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

fn main() {
    let args = Args::parse();

    println!("{:?}", args);

    emulator::Chip8::new()
        .run();
}
