use std::env;

extern crate sdl2;
mod drivers;
use drivers::audio::AudioDriver;
use drivers::display::DisplayDriver;
use drivers::keyboard::KeyboardDriver;

mod processor;
use processor::Processor;

const RAM_SIZE: usize = 4 * 1024; // 4 KB
const GPR_SIZE: usize = 16;

const CHIP8_WIDTH: u32 = 64;
const CHIP8_HEIGHT: u32 = 32;

fn main() {
    let rom_name = env::args().nth(1).expect("Please provide a file name.");

    //drivers
    // event_pump is the key, need to poll on that.
    let sdl_context = sdl2::init().unwrap();
    let mut display_driver = DisplayDriver::new(&sdl_context);
    let mut audio_driver = AudioDriver::new(&sdl_context);
    let mut keyboard_driver = KeyboardDriver::new(&sdl_context);

    //
    // processor
    //
    let mut cpu = Processor::default();
    cpu.reset();
    cpu.load_rom(rom_name);

    // control unit & datapath inside processor?
    // keyboard/audio/display should be controlled by control unit

    // Peripheral driver? That has the event pump and knows how to interact with the other drivers?
    // And receives messages when to redraw the screen, beep, etc?
    // Captures keyboard state and stores it somewhere the CPU can get it?

    // CPU run
    'running: loop {
        //
        //
        // Every tick:
        //    get current kb state
        //    if update_display flag is set, draw then reset the flag
        //    if sound flag is set?
        //
        //    then perform instruction? Or instruction first?
        //

        let keep_going = keyboard_driver.poll(); // make this return the key set as a result

        if !keep_going {
            break 'running;
        }

        std::thread::sleep(std::time::Duration::from_millis(500));

        let mut pixels = [[0 as u8; CHIP8_WIDTH as usize]; CHIP8_HEIGHT as usize];
        for y in 0..CHIP8_HEIGHT {
            for x in 0..CHIP8_WIDTH {
                pixels[y as usize][x as usize] = (y as u8 + x as u8) % 2;
            }
        }

        // This is just spots in memory, right?
        display_driver.draw(&pixels);

        audio_driver.start_beep();
        std::thread::sleep(std::time::Duration::from_millis(100));
        audio_driver.stop_beep();

        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    cpu.run(); // need to put keyboard driver/poll into cpu run
}
