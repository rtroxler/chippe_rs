use std::env;

extern crate sdl2;
mod drivers;
mod font;

mod processor;
use processor::Processor;

const RAM_SIZE: usize = 4 * 1024; // 4 KB
const GPR_SIZE: usize = 16;

const CHIP8_WIDTH: usize = 64;
const CHIP8_HEIGHT: usize = 32;

fn main() {
    let rom_name = env::args().nth(1).expect("Please provide a file name.");
    let sdl_context = sdl2::init().unwrap();
    let mut cpu = Processor::new(&sdl_context);

    cpu.reset();
    cpu.load_rom(rom_name);
    cpu.run();
}
