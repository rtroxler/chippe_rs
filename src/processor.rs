use std::fmt;
use std::fs;
use std::io::Read;
use std::path::Path;

extern crate rand;

use crate::GPR_SIZE;
use crate::RAM_SIZE;

use crate::CHIP8_HEIGHT;
use crate::CHIP8_WIDTH;

use crate::font::FONT_SET;

use crate::drivers::audio::AudioDriver;
use crate::drivers::display::DisplayDriver;
use crate::drivers::keyboard::KeyboardDriver;

struct RamArray {
    pub memory: Box<[u8; RAM_SIZE]>,
}

impl fmt::Debug for RamArray {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        "Too big".fmt(formatter)
        //self.memory[0x200..0x201].fmt(formatter)
    }
}

impl RamArray {
    fn new() -> RamArray {
        RamArray {
            memory: Box::new([0; RAM_SIZE]),
        }
    }
}

//self.peripheral_driver.audio.start_beep();
//self.peripheral_driver.audio.stop_beep();
struct PeripheralDriver {
    audio: AudioDriver,
    display: DisplayDriver,
    keyboard: KeyboardDriver,
}

pub struct Processor {
    peripheral_driver: PeripheralDriver,
    clock_speed: u64,
    program_counter: u16,
    display_state: [[u8; CHIP8_WIDTH as usize]; CHIP8_HEIGHT as usize],
    keyboard_state: [bool; 16],
    gpr_v: [u8; GPR_SIZE], // General Purpose Registers (V0 - VF)
    reg_i: u16,
    delay_timer: u8,
    sound_timer: u8,
    stack_pointer: u16,
    ram: RamArray,
}

impl Processor {
    pub fn new(sdl: &sdl2::Sdl) -> Processor {
        Processor {
            peripheral_driver: PeripheralDriver {
                audio: AudioDriver::new(&sdl),
                display: DisplayDriver::new(&sdl),
                keyboard: KeyboardDriver::new(&sdl),
            },
            clock_speed: 1,
            program_counter: 0,
            keyboard_state: [false; 16],
            display_state: [[0 as u8; CHIP8_WIDTH as usize]; CHIP8_HEIGHT as usize],
            gpr_v: [0; GPR_SIZE],
            reg_i: 0,
            delay_timer: 0,
            sound_timer: 0,
            stack_pointer: 0,
            ram: RamArray::new(),
        }
    }

    pub fn load_rom<P: AsRef<Path>>(&mut self, path: P) {
        // Open file and dump contents into file_buf
        let mut file = fs::File::open(path).unwrap();
        let mut file_buf = Vec::new();
        file.read_to_end(&mut file_buf).unwrap();

        // Define temp ram array
        let mut ram = [0; 4096];

        // read font into ram
        for i in 0..FONT_SET.len() {
            ram[i] = FONT_SET[i];
        }
        // Copy file binary into ram, starting at 0x200
        println!("Loading file length: {} bytes", file_buf.len());
        ram[0x200..file_buf.len() + 0x200].copy_from_slice(&file_buf[..]);

        self.ram = RamArray {
            memory: Box::new(ram),
        }
    }

    pub fn reset(&mut self) {
        self.program_counter = 0x200;
        self.stack_pointer = 0xfa0;
        // screen is set to a slice of ram starting at 0xf00 ?
    }

    pub fn run(&mut self) {
        'running: loop {
            // set keyboard state and detect interrupt
            match self.peripheral_driver.keyboard.poll() {
                Ok(key_state) => self.keyboard_state = key_state,
                Err(_e) => break 'running,
            } // make this return the key set as a result

            std::thread::sleep(std::time::Duration::from_millis(self.clock_speed));

            // This is just spots in memory, right?
            // yep
            self.peripheral_driver.display.draw(&self.display_state);

            // Decrement DT and ST by 1 each 60 Hz?
            // just gonna be a clock cycle for now
            if self.delay_timer >= 1 {
                self.delay_timer -= 1;
            }
            if self.delay_timer >= 1 {
                // play sound
                self.peripheral_driver.audio.start_beep();
                self.sound_timer -= 1;
            } else {
                self.peripheral_driver.audio.stop_beep();
            }

            //
            // Instruction fetch and execute
            //
            let op1 = self.ram.memory[self.program_counter as usize];
            let op2 = self.ram.memory[self.program_counter as usize + 1];

            if op1 == 0x00 && op2 == 0x00 {
                // Break on 0 byte? Avoids the zeroed out end of RAM, not sure if
                // necessary or not though
                break 'running;
            }

            // display instructions for debugging
            let str_instruction = fetch_instruction_str(op1, op2);
            println!(
                "{:04x?} {:02x} {:02x} :: {}",
                self.program_counter, op1, op2, str_instruction
            );

            self.execute(op1, op2);
        }
    }

    fn execute(&mut self, byte1: u8, byte2: u8) {
        let high_nibble = byte1 >> 4;
        let lo_nibble = byte1 & 0x0F;

        let high_nibble2 = byte2 >> 4;
        let lo_nibble2 = byte2 & 0x0F;

        match high_nibble {
            0x0 => match byte2 {
                0xE0 => {
                    //CLS
                    self.display_state = [[0 as u8; CHIP8_WIDTH as usize]; CHIP8_HEIGHT as usize];
                    self.program_counter += 2;
                }
                0xEE => {
                    //RET
                    let pc_high = self.ram.memory[self.stack_pointer as usize];
                    let pc_lo = self.ram.memory[(self.stack_pointer + 1) as usize];

                    let target: u16 = (((pc_high as u16) << 8) | (pc_lo as u16)).into();
                    self.program_counter = target;
                    // I've been putting CALL on the stack then popping it and immediately calling again
                    self.program_counter += 2;

                    self.stack_pointer -= 2;
                }
                _ => {}
            },
            0x1 => {
                // JP nnn
                let target: u16 = (((lo_nibble as u16) << 8) | (byte2 as u16)).into();
                //println!("\tJP {:x?}", target);
                self.program_counter = target
            }
            0x2 => {
                //CALL addr

                // inc by 2 since we want to store a 16 bit addr and our ram is u8
                //println!("Current SP: {:?}", self.stack_pointer);
                self.stack_pointer += 2;

                let pc_high = (self.program_counter & 0xFF00) >> 8;
                let pc_lo = self.program_counter & 0x00FF;

                self.ram.memory[self.stack_pointer as usize] = pc_high as u8;
                self.ram.memory[(self.stack_pointer + 1) as usize] = pc_lo as u8;

                let target: u16 = (((lo_nibble as u16) << 8) | (byte2 as u16)).into();
                self.program_counter = target
            }
            0x3 => {
                // SE Vx, byte
                println!(
                    "\t SKIP IF if {:x?} == {:x?} ",
                    self.gpr_v[lo_nibble as usize], byte2
                );
                if self.gpr_v[lo_nibble as usize] == byte2 {
                    self.program_counter += 2;
                }
                self.program_counter += 2;
            }
            0x4 => {
                // SNE Vx, byte
                if self.gpr_v[lo_nibble as usize] != byte2 {
                    self.program_counter += 2;
                }
                self.program_counter += 2;
            }
            0x5 => {
                // SE Vx, Vy
                if self.gpr_v[lo_nibble as usize] == self.gpr_v[high_nibble2 as usize] {
                    self.program_counter += 2;
                }
                self.program_counter += 2;
            }
            0x6 => {
                //LD Vx, byte
                self.gpr_v[lo_nibble as usize] = byte2;

                self.program_counter += 2;
            }
            0x7 => {
                // ADD Vx, byte
                self.gpr_v[lo_nibble as usize] = self.gpr_v[lo_nibble as usize].wrapping_add(byte2);

                self.program_counter += 2;
            }
            0x8 => match lo_nibble2 {
                0x0 => {
                    // LD Vx, Vy
                    self.gpr_v[lo_nibble as usize] = self.gpr_v[high_nibble2 as usize];

                    self.program_counter += 2;
                }
                0x1 => {
                    // OR Vx, Vy
                    self.gpr_v[lo_nibble as usize] =
                        self.gpr_v[lo_nibble as usize] | self.gpr_v[high_nibble2 as usize];

                    self.program_counter += 2;
                }
                0x2 => {
                    // AND Vx, Vy
                    self.gpr_v[lo_nibble as usize] =
                        self.gpr_v[lo_nibble as usize] & self.gpr_v[high_nibble2 as usize];

                    self.program_counter += 2;
                }
                0x3 => {
                    // XOR Vx, Vy
                    self.gpr_v[lo_nibble as usize] =
                        self.gpr_v[lo_nibble as usize] ^ self.gpr_v[high_nibble2 as usize];

                    self.program_counter += 2;
                }
                0x4 => {
                    // ADD Vx, Vy, set VF to carry
                    let vx = self.gpr_v[lo_nibble as usize] as u16;
                    let vy = self.gpr_v[high_nibble2 as usize] as u16;
                    let result = vx + vy;

                    self.gpr_v[lo_nibble as usize] = result as u8;
                    self.gpr_v[0x0f] = if result > 0xFF { 1 } else { 0 };

                    self.program_counter += 2;
                }
                0x5 => {
                    //SUB Vx, Vy
                    if self.gpr_v[lo_nibble as usize] > self.gpr_v[high_nibble2 as usize] {
                        self.gpr_v[0xf] = 1
                    } else {
                        self.gpr_v[0xf] = 0
                    }

                    self.gpr_v[lo_nibble as usize] =
                        self.gpr_v[lo_nibble as usize] - self.gpr_v[high_nibble2 as usize];

                    self.program_counter += 2;
                }
                0x6 => {
                    // SHR Vx
                    if self.gpr_v[lo_nibble as usize] & 0x1 == 1 {
                        // I think this might also be wrong
                        self.gpr_v[0xf] = 1
                    } else {
                        self.gpr_v[0xf] = 0
                    }
                    self.gpr_v[lo_nibble as usize] = self.gpr_v[lo_nibble as usize] >> 1;

                    self.program_counter += 2;
                }
                0x7 => {
                    // SUBN Vx, Vy
                    if self.gpr_v[lo_nibble as usize] < self.gpr_v[high_nibble2 as usize] {
                        self.gpr_v[0xf] = 1
                    } else {
                        self.gpr_v[0xf] = 0
                    }

                    self.gpr_v[lo_nibble as usize] =
                        self.gpr_v[high_nibble2 as usize] - self.gpr_v[lo_nibble as usize];

                    self.program_counter += 2;
                }
                0xE => {
                    // SHL Vx
                    if self.gpr_v[lo_nibble as usize] & 0x8 == 1 {
                        // I think this is wrong
                        self.gpr_v[0xf] = 1
                    } else {
                        self.gpr_v[0xf] = 0
                    }
                    self.gpr_v[lo_nibble as usize] = self.gpr_v[lo_nibble as usize] << 1;

                    self.program_counter += 2;
                }
                _ => {}
            },
            0x9 => {
                // SNE Vx, Vy
                if self.gpr_v[lo_nibble as usize] != self.gpr_v[high_nibble2 as usize] {
                    // Extra +2 will skip the next instruction
                    self.program_counter += 2;
                }
                self.program_counter += 2;
            }
            0xA => {
                // LD I, addr
                let target: u16 = (((lo_nibble as u16) << 8) | (byte2 as u16)).into();
                self.reg_i = target;

                self.program_counter += 2;
            }
            0xB => {
                // JP V0, addr
                let target: u16 = (((lo_nibble as u16) << 8) | (byte2 as u16)).into();
                self.program_counter = target + self.gpr_v[0 as usize] as u16;
            }
            0xC => {
                // RND Vx, byte
                let random = rand::random::<u8>();

                self.gpr_v[lo_nibble as usize] = random & byte2;

                self.program_counter += 2;
            }
            0xD => {
                // DRW Vx, Vy, nibble

                // Get a slice of ram[reg_i..reg_i + lo_nib2]
                // each byte will occupy 8 rows
                // num of columns represented by number of bytes
                self.gpr_v[0x0f] = 0;
                for byte in 0..(lo_nibble2 as usize) {
                    let y = (self.gpr_v[high_nibble2 as usize] as usize + byte) % CHIP8_HEIGHT;
                    for bit in 0..8 {
                        let x = (self.gpr_v[lo_nibble as usize] as usize + bit) % CHIP8_WIDTH;
                        let color = (self.ram.memory[(self.reg_i + (byte as u16)) as usize]
                            >> (7 - bit))
                            & 1;
                        self.gpr_v[0x0f] |= color & self.display_state[y][x];
                        self.display_state[y][x] ^= color;
                    }
                }

                self.program_counter += 2;
            }
            0xE => match byte2 {
                0x9E => {
                    // SKP Vx, lo_nibble
                    if self.keyboard_state[self.gpr_v[lo_nibble as usize] as usize] {
                        self.program_counter += 2;
                    }

                    self.program_counter += 2;
                }
                0xA1 => {
                    // SKNP Vx, lo_nibble
                    if !self.keyboard_state[self.gpr_v[lo_nibble as usize] as usize] {
                        self.program_counter += 2;
                    }
                    self.program_counter += 2;
                }
                _ => {}
            },
            0xF => match byte2 {
                0x07 => {
                    // LD Vx, DT
                    self.gpr_v[lo_nibble as usize] = self.delay_timer;

                    self.program_counter += 2;
                }
                0x0A => {
                    println!("\n\n!!! TODO LD Vx, K !!!\n\n");
                    //
                    // TODO !!! not in pong !!!
                    // LD Vx, K
                    // OOO. Halt execution until a key is pressed, then load the value into Vx
                    //
                    // how does SDL handle this?
                    // or just fucking loop in here dawg
                    //
                    //format!("LD V{:x?}, K", lo_nibble),
                    self.program_counter += 2;
                }
                0x15 => {
                    // LD DT, Vx
                    self.delay_timer = self.gpr_v[lo_nibble as usize];
                    println!("\tDT: {:x?}", self.delay_timer);
                    self.program_counter += 2;
                }
                0x18 => {
                    //LD ST, Vx
                    self.sound_timer = self.gpr_v[lo_nibble as usize];

                    self.program_counter += 2;
                }
                0x1E => {
                    // ADD I, Vx
                    self.reg_i = self.reg_i + self.gpr_v[lo_nibble as usize] as u16;

                    self.program_counter += 2;
                }
                0x29 => {
                    // LD F, Vx

                    // since the fonts are loaded at the front of the memory, accessing them
                    // by this with no offset should get me what I want?
                    let maybe_font =
                        self.ram.memory[self.gpr_v[lo_nibble as usize] as usize] as u16;
                    println!("\tmaybe font? {:x?}", maybe_font);
                    self.reg_i = self.ram.memory[self.gpr_v[lo_nibble as usize] as usize] as u16;

                    self.program_counter += 2;
                }
                0x33 => {
                    // LD B, Vx

                    let mut value = self.gpr_v[lo_nibble as usize];
                    let ones = value % 10;
                    value = value / 10;
                    let tens = value % 10;
                    let hundreds = value / 10;
                    self.ram.memory[self.reg_i as usize] = hundreds;
                    self.ram.memory[(self.reg_i + 1) as usize] = tens;
                    self.ram.memory[(self.reg_i + 2) as usize] = ones;

                    self.program_counter += 2;
                }
                0x55 => {
                    // LD [I], Vx
                    //Store registers V0 through Vx in memory starting at location I.
                    for i in 0x0..=lo_nibble {
                        self.ram.memory[(self.reg_i + (i as u16)) as usize] =
                            self.gpr_v[i as usize];
                    }

                    self.program_counter += 2;
                }
                0x65 => {
                    // LD Vx, [I]
                    //Read registers V0 through Vx from memory starting at location I.
                    for i in 0x0..=lo_nibble {
                        self.gpr_v[i as usize] =
                            self.ram.memory[(self.reg_i + (i as u16)) as usize];
                    }
                    self.program_counter += 2;
                }
                _ => {}
            },
            _ => {}
        }
    }
}

// Chip-8 Disassembler
fn fetch_instruction_str(byte1: u8, byte2: u8) -> String {
    let high_nibble = byte1 >> 4;
    let lo_nibble = byte1 & 0x0F;

    let high_nibble2 = byte2 >> 4;
    let lo_nibble2 = byte2 & 0x0F;

    match high_nibble {
        0x0 => match byte2 {
            0xE0 => format!("CLS"),
            0xEE => format!("RET"),
            _ => format!("not supported ({:x?}{:x?})", byte1, byte2),
        },
        0x1 => format!("JP {:x?}{:x?}", lo_nibble, byte2),
        0x2 => format!("CALL {:x?}{:x?}", lo_nibble, byte2),
        0x3 => format!("SE V{:x?}, {:x?}", lo_nibble, byte2),
        0x4 => format!("SNE V{:x?}, {:x?}", lo_nibble, byte2),
        0x5 => format!("SE V{:x?}, V{:x?}", lo_nibble, high_nibble2),
        0x6 => format!("LD V{:x?}, {:x?}", lo_nibble, byte2),
        0x7 => format!("ADD V{:x?}, {:x?}", lo_nibble, byte2),
        0x8 => match lo_nibble2 {
            0x0 => format!("LD V{:x?}, V{:x?}", lo_nibble, high_nibble2),
            0x1 => format!("OR V{:x?}, V{:x?}", lo_nibble, high_nibble2),
            0x2 => format!("AND V{:x?}, V{:x?}", lo_nibble, high_nibble2),
            0x3 => format!("XOR V{:x?}, V{:x?}", lo_nibble, high_nibble2),
            0x4 => format!("ADD V{:x?}, V{:x?}", lo_nibble, high_nibble2),
            0x5 => format!("SUB V{:x?}, V{:x?}", lo_nibble, high_nibble2),
            0x6 => format!("SHR V{:x?} {{, V{:x?}}}", lo_nibble, high_nibble2),
            0x7 => format!("SUBN V{:x?}, V{:x?}", lo_nibble, high_nibble2),
            0xE => format!("SHL V{:x?} {{, V{:x?}}}", lo_nibble, high_nibble2),
            _ => format!("not supported ({:x?}{:x?})", byte1, byte2),
        },
        0x9 => format!("SNE V{:x?}, {:x?}", lo_nibble, byte2),
        0xA => format!("LD I, {:x?}{:x?}", lo_nibble, byte2),
        0xB => format!("JP V0, {:x?}{:x?}", lo_nibble, byte2),
        0xC => format!("RND V{:x?}, {:x?}", lo_nibble, byte2),
        0xD => format!(
            "DRW V{:x?}, V{:x?}, {:x?}",
            lo_nibble, high_nibble2, lo_nibble2
        ),
        0xE => match byte2 {
            0x9E => format!("SKP V{:x?}", lo_nibble),
            0xA1 => format!("SKNP V{:x?}", lo_nibble),
            _ => format!("not supported ({:x?}{:x?})", byte1, byte2),
        },
        0xF => match byte2 {
            0x07 => format!("LD V{:x?}, DT", lo_nibble),
            0x0A => format!("LD V{:x?}, K", lo_nibble),
            0x15 => format!("LD DT, V{:x?}", lo_nibble),
            0x18 => format!("LD ST, V{:x?}", lo_nibble),
            0x1E => format!("ADD I, V{:x?}", lo_nibble),
            0x29 => format!("LD F, V{:x?}", lo_nibble),
            0x33 => format!("LD B, V{:x?}", lo_nibble),
            0x55 => format!("LD [I], V{:x?}", lo_nibble),
            0x65 => format!("LD V{:x?}, [I]", lo_nibble),
            _ => format!("not supported ({:x?}{:x?})", byte1, byte2),
        },
        _ => format!("{:x?}", high_nibble),
    }
}
