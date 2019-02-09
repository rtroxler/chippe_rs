use std::env;
use std::fmt;
use std::fs;
use std::io::Read;
use std::path::Path;

const STACK_SIZE: usize = 16;
const RAM_SIZE: usize = 4 * 1024; // 4 KB
const GPR_SIZE: usize = 16;

struct Cpu {
    program_counter: usize,
    gpr_v: [u8; GPR_SIZE], // General Purpose Registers (V0 - VF)
    reg_i: u16,
    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; STACK_SIZE],
    stack_pointer: u8,
    ram: Box<[u8; RAM_SIZE]>,
}

impl Default for Cpu {
    fn default() -> Cpu {
        Cpu {
            program_counter: 0,
            gpr_v: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            reg_i: 0,
            delay_timer: 0,
            sound_timer: 0,
            stack: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            stack_pointer: 0,
            ram: Box::new([0; RAM_SIZE]),
        }
    }
}

impl Cpu {
    fn load_rom<P: AsRef<Path>>(&mut self, path: P) {
        // Open file and dump contents into file_buf
        let mut file = fs::File::open(path).unwrap();
        let mut file_buf = Vec::new();
        file.read_to_end(&mut file_buf).unwrap();

        // Define temp ram array
        let mut ram = [0; 4096];
        // Copy file binary into ram, starting at 0x200
        ram[0x200..file_buf.len() + 0x200].copy_from_slice(&file_buf[..]);

        self.ram = Box::new(ram);
    }

    fn run(&mut self) {
        self.program_counter = 0x200;

        while self.program_counter < self.ram.len() {
            let op1 = self.ram[self.program_counter];
            let op2 = self.ram[self.program_counter + 1];

            if op1 == 0x00 && op2 == 0x00 {
                // Break on 0 byte? Avoids the zeroed out end of RAM, not sure if
                // necessary or not though
                break;
            }

            let str_instruction = fetch_instruction_str(op1, op2);
            println!("{}", str_instruction);

            self.program_counter += 2;
        }
    }
}

fn main() {
    let rom_name = env::args().nth(1).expect("Please provide a file name.");

    let mut cpu = Cpu::default();
    cpu.load_rom(rom_name);

    cpu.run();
}

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
