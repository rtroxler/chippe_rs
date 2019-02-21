use std::fmt;
use std::fs;
use std::io::Read;
use std::path::Path;

extern crate rand;
use rand::prelude::*;

use crate::GPR_SIZE;
use crate::RAM_SIZE;

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

    fn len(&mut self) -> usize {
        self.memory.len()
    }
}

#[derive(Debug)]
pub struct Processor {
    program_counter: u16,
    gpr_v: [u8; GPR_SIZE], // General Purpose Registers (V0 - VF)
    reg_i: u16,
    delay_timer: u8,
    sound_timer: u8,
    stack_pointer: u16,
    ram: RamArray,
}

impl Default for Processor {
    fn default() -> Processor {
        Processor {
            program_counter: 0,
            gpr_v: [0; GPR_SIZE],
            reg_i: 0,
            delay_timer: 0,
            sound_timer: 0,
            stack_pointer: 0,
            ram: RamArray::new(),
        }
    }
}

impl Processor {
    pub fn load_rom<P: AsRef<Path>>(&mut self, path: P) {
        // Open file and dump contents into file_buf
        let mut file = fs::File::open(path).unwrap();
        let mut file_buf = Vec::new();
        file.read_to_end(&mut file_buf).unwrap();

        // Define temp ram array
        let mut ram = [0; 4096];
        // Copy file binary into ram, starting at 0x200
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
        while (self.program_counter as usize) < self.ram.len() {
            // Display if needed to redraw

            // Instruction fetch
            let op1 = self.ram.memory[self.program_counter as usize];
            let op2 = self.ram.memory[self.program_counter as usize + 1];

            if op1 == 0x00 && op2 == 0x00 {
                // Break on 0 byte? Avoids the zeroed out end of RAM, not sure if
                // necessary or not though
                break;
            }

            let str_instruction = fetch_instruction_str(op1, op2);
            println!(
                "{:04x?} {:02x} {:02x} :: {}",
                self.program_counter, op1, op2, str_instruction
            );

            let temp = self.program_counter.clone();
            self.execute(op1, op2);
            if temp == self.program_counter {
                println!("infinite loop detected");
                dbg!(self);
                break;
            }
        }
    }

    //fn load(dst: u8, src: u8) {
    //// TODO
    //}

    fn execute(&mut self, byte1: u8, byte2: u8) {
        // todo
        let high_nibble = byte1 >> 4;
        let lo_nibble = byte1 & 0x0F;

        let high_nibble2 = byte2 >> 4;
        let lo_nibble2 = byte2 & 0x0F;

        match high_nibble {
            0x0 => match byte2 {
                0xE0 => {
                    //CLS
                }
                0xEE => {
                    //RET
                    let pc_high = self.ram.memory[self.stack_pointer as usize];
                    let pc_lo = self.ram.memory[(self.stack_pointer + 1) as usize];

                    let target: u16 = (((pc_high as u16) << 8) | (pc_lo as u16)).into();
                    self.program_counter = target;

                    self.stack_pointer -= 2;
                }
                _ => {}
            },
            0x1 => {
                // JP nnn
                let target: u16 = (((lo_nibble as u16) << 8) | (byte2 as u16)).into();
                self.program_counter = target
            }
            0x2 => {
                //CALL addr
                let target: u16 = (((lo_nibble as u16) << 8) | (byte2 as u16)).into();

                // inc by 2 since we want to store a 16 bit addr and our ram is u8
                self.stack_pointer += 2;

                let pc_high = (self.program_counter & 0xFF00) >> 8;
                let pc_lo = self.program_counter & 0x00FF;

                // TODO Endianness ?
                self.ram.memory[self.stack_pointer as usize] = pc_high as u8;
                self.ram.memory[(self.stack_pointer + 1) as usize] = pc_lo as u8;

                self.program_counter = target
            }
            0x3 => {
                // SE Vx, byte
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
                self.gpr_v[lo_nibble as usize] += byte2;

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
                    // TODO Set VF
                    self.gpr_v[lo_nibble as usize] =
                        self.gpr_v[lo_nibble as usize] + self.gpr_v[high_nibble2 as usize];

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
                _ => {
                    //format!("not supported ({:x?}{:x?})", byte1, byte2),
                }
            },
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
                //"DRW V{:x?}, V{:x?}, {:x?}",
                // TODO

                self.program_counter += 2;
            }
            0xE => match byte2 {
                0x9E => {
                    //format!("SKP V{:x?}", lo_nibble),
                }
                0xA1 => {
                    //format!("SKNP V{:x?}", lo_nibble),
                }
                _ => {
                    //format!("not supported ({:x?}{:x?})", byte1, byte2),
                }
            },
            0xF => match byte2 {
                0x07 => {
                    //format!("LD V{:x?}, DT", lo_nibble),
                }
                0x0A => {
                    //format!("LD V{:x?}, K", lo_nibble),
                }
                0x15 => {
                    //format!("LD DT, V{:x?}", lo_nibble),
                }
                0x18 => {
                    //format!("LD ST, V{:x?}", lo_nibble),
                }
                0x1E => {
                    //format!("ADD I, V{:x?}", lo_nibble),
                }
                0x29 => {
                    //format!("LD F, V{:x?}", lo_nibble),
                }
                0x33 => {
                    //format!("LD B, V{:x?}", lo_nibble),
                    self.program_counter += 2;
                }
                0x55 => {
                    //format!("LD [I], V{:x?}", lo_nibble),
                }
                0x65 => {
                    //format!("LD V{:x?}, [I]", lo_nibble),
                }
                _ => {
                    //format!("not supported ({:x?}{:x?})", byte1, byte2),
                }
            },
            _ => {
                //format!("{:x?}", high_nibble),
            }
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
