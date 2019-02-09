use std::env;
use std::fs;
use std::io::Read;
use std::path::Path;

fn main() {
    if let Some(rom_name) = env::args().nth(1) {
        let rom = read_bin(rom_name);
        run(rom);
    } else {
        println!("No file provided.");
    }
}

fn run(rom: Vec<u8>) {
    let mut program_counter = 0;

    while program_counter < rom.len() {
        let op1 = rom[program_counter];
        let op2 = rom[program_counter + 1];

        let high_nibble = op1 >> 4;
        let lo_nibble = op1 & 0x0F;

        let high_nibble2 = op2 >> 4;
        let lo_nibble2 = op2 & 0x0F;

        match high_nibble {
            0x0 => match op2 {
                0xE0 => println!("CLS"),
                0xEE => println!("RET"),
                _ => println!("not supported ({:x?}{:x?})", op1, op2),
            },
            0x1 => println!("JP {:x?}{:x?}", lo_nibble, op2),
            0x2 => println!("CALL {:x?}{:x?}", lo_nibble, op2),
            0x3 => println!("SE V{:x?}, {:x?}", lo_nibble, op2),
            0x4 => println!("SNE V{:x?}, {:x?}", lo_nibble, op2),
            0x5 => println!("SE V{:x?}, V{:x?}", lo_nibble, high_nibble2),
            0x6 => println!("LD V{:x?}, {:x?}", lo_nibble, op2),
            0x7 => println!("ADD V{:x?}, {:x?}", lo_nibble, op2),
            0x8 => match lo_nibble2 {
                0x0 => println!("LD V{:x?}, V{:x?}", lo_nibble, high_nibble2),
                0x1 => println!("OR V{:x?}, V{:x?}", lo_nibble, high_nibble2),
                0x2 => println!("AND V{:x?}, V{:x?}", lo_nibble, high_nibble2),
                0x3 => println!("XOR V{:x?}, V{:x?}", lo_nibble, high_nibble2),
                0x4 => println!("ADD V{:x?}, V{:x?}", lo_nibble, high_nibble2),
                0x5 => println!("SUB V{:x?}, V{:x?}", lo_nibble, high_nibble2),
                0x6 => println!("SHR V{:x?} {{, V{:x?}}}", lo_nibble, high_nibble2),
                0x7 => println!("SUBN V{:x?}, V{:x?}", lo_nibble, high_nibble2),
                0xE => println!("SHL V{:x?} {{, V{:x?}}}", lo_nibble, high_nibble2),
                _ => println!("not supported ({:x?}{:x?})", op1, op2),
            },
            0xA => println!("LD I, {:x?}{:x?}", lo_nibble, op2),
            0xB => println!("JP V0, {:x?}{:x?}", lo_nibble, op2),
            0xC => println!("RND V{:x?}, {:x?}", lo_nibble, op2),
            0xD => println!(
                "DRW V{:x?}, V{:x?}, {:x?}",
                lo_nibble, high_nibble2, lo_nibble2
            ),
            0xE => match op2 {
                0x9E => println!("SKP V{:x?}", lo_nibble),
                0xA1 => println!("SKNP V{:x?}", lo_nibble),
                _ => println!("not supported ({:x?}{:x?})", op1, op2),
            },
            0xF => match op2 {
                0x07 => println!("LD V{:x?}, DT", lo_nibble),
                0x0A => println!("LD V{:x?}, K", lo_nibble),
                0x15 => println!("LD DT, V{:x?}", lo_nibble),
                0x18 => println!("LD ST, V{:x?}", lo_nibble),
                0x1E => println!("ADD I, V{:x?}", lo_nibble),
                0x29 => println!("LD F, V{:x?}", lo_nibble),
                0x33 => println!("LD B, V{:x?}", lo_nibble),
                0x55 => println!("LD [I], V{:x?}", lo_nibble),
                0x65 => println!("LD V{:x?}, [I]", lo_nibble),
                _ => println!("not supported ({:x?}{:x?})", op1, op2),
            },
            _ => println!("{:x?}", high_nibble),
        }

        program_counter += 2;
    }
}

fn read_bin<P: AsRef<Path>>(path: P) -> Vec<u8> {
    let mut file = fs::File::open(path).unwrap();
    let mut file_buf = Vec::new();
    file.read_to_end(&mut file_buf).unwrap();
    file_buf
}
