use minifb::{Key, Window, WindowOptions};
use std::fs;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;
const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

struct Chip8 {
    memory: [u8; 4096],
    pc: usize,
    display: [u32; 64 * 32],
    v: [u8; 16], // Registers
    i: u16,
    stack: Vec<u16>,
}

impl Default for Chip8 {
    fn default() -> Self {
        Self {
            memory: [0; 4096],
            pc: 0x200,
            display: [0; WIDTH * HEIGHT],
            v: [0; 16],
            i: 0,
            stack: Vec::new(),
        }
    }
}

fn write_font_to_memory(chip: &mut Chip8) {
    chip.memory[0x000..0x050].copy_from_slice(&FONT);
}

fn write_program_to_memory(
    filename: &str,
    chip: &mut Chip8,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::read(filename)?;
    chip.memory[0x200..0x200 + file.len()].copy_from_slice(&file);

    Ok(())
}

fn write_test_program_to_memory(chip: &mut Chip8) {
    // 1010-0000
    // 0000-0000  0000-0000

    let bytes = vec![0x60, 0x69, 0xA0, 0x00, 0xA0, 0x00];

    chip.memory[0x200..0x200 + bytes.len()].copy_from_slice(&bytes);
}

fn tick(chip: &mut Chip8) {
    // 1010 0000 0000 0000
    let instruction = (chip.memory[chip.pc] as u16) << 8 | chip.memory[chip.pc + 1] as u16;

    // 0000 1010
    let opcode = ((instruction & 0xF000) >> 12) as u8;
    match opcode {
        // No data instructions
        0x0 => {
            let rest = instruction & 0x0FFF;
            match rest {
                // Clear the screen 00E0
                0x0E0 => {
                    for i in 0..WIDTH * HEIGHT {
                        chip.display[i] = 0;
                    }
                }

                // Return from subroutine
                0x0EE => {
                    if let Some(return_pc) = chip.stack.pop() {
                        chip.pc = return_pc as usize;
                    }
                }
                _ => {}
            }
        }

        // Jump 1NNN
        0x1 => {
            let data = instruction & 0x0FFF;
            chip.pc = data as usize;

            println!("Jumped to {:x}", data);
            return;
        }

        // Call subroutine
        0x2 => {
            let data = instruction & 0x0FFF;
            chip.stack.push(chip.pc as u16);

            chip.pc = data as usize;
        }

        // Condition - Equal 3XNN
        0x3 => {
            let register_x = (instruction & 0x0F00) >> 8;
            let data = (instruction & 0x00FF) as u8;

            if chip.v[register_x as usize] == data {
                chip.pc += 2;
            }
        }

        // Condition - Not Equals 4XNN
        0x4 => {
            let register_x = (instruction & 0x0F00) >> 8;
            let data = (instruction & 0x00FF) as u8;

            if chip.v[register_x as usize] != data {
                chip.pc += 2;
            }
        }

        // Condition - Equals 5XY0
        0x5 => {
            let register_x = (instruction & 0x0F00) >> 8;
            let register_y = (instruction & 0x00F0) >> 4;

            if chip.v[register_x as usize] == chip.v[register_y as usize] {
                chip.pc += 2;
            }
        }

        // Condition - Equals 9XY0
        0x9 => {
            let register_x = (instruction & 0x0F00) >> 8;
            let register_y = (instruction & 0x00F0) >> 4;

            if chip.v[register_x as usize] != chip.v[register_y as usize] {
                chip.pc += 2;
            }
        }

        //  Set register to value 6XNN
        0x6 => {
            let register = (instruction & 0x0F00) >> 8;
            let data = (instruction & 0x00FF) as u8;

            chip.v[register as usize] = data;

            println!("Register {:x} set to {:x}", register, data);
        }

        // Add value to register 7XNN
        0x7 => {
            let register = (instruction & 0x0F00) >> 8;
            let data = (instruction & 0x00FF) as u8;

            chip.v[register as usize] += data;

            println!("Register {:x} incremented by {:x}", register, data);
        }

        // Arithmetic
        0x8 => {
            let register_x = ((instruction & 0x0F00) >> 8) as usize;
            let register_y = ((instruction & 0x00F0) >> 4) as usize;
            let operation = instruction & 0x000F;

            match operation {
                0x0 => {
                    chip.v[register_x] = chip.v[register_y];
                }
                0x1 => {
                    chip.v[register_x] |= chip.v[register_y];
                }
                0x2 => {
                    chip.v[register_x] &= chip.v[register_y];
                }
                0x3 => {
                    chip.v[register_x] ^= chip.v[register_y];
                }
                0x4 => {
                    let (result, overflow) = chip.v[register_x].overflowing_add(chip.v[register_y]);
                    chip.v[register_x] = result;
                    chip.v[0xF] = overflow as u8;
                }
                0x5 => {
                    let no_borrow = chip.v[register_x] >= chip.v[register_y];
                    let _ = chip.v[register_x].wrapping_sub(chip.v[register_y]);
                    chip.v[0xF] = no_borrow as u8;
                }
                0x6 => {
                    chip.v[register_x] = chip.v[register_y]; // This might break some programs
                    let shifted_out = chip.v[register_x] & 0x1;
                    chip.v[register_x] >>= 1;
                    chip.v[0xF] = shifted_out;
                }
                0xE => {
                    chip.v[register_x] = chip.v[register_y]; // This might break some programs
                    let shifted_out = chip.v[register_x] & 0x1;
                    chip.v[register_x] <<= 1;
                    chip.v[0xF] = shifted_out;
                }

                0x7 => {
                    let no_borrow = chip.v[register_y] >= chip.v[register_x];
                    let _ = chip.v[register_y].wrapping_sub(chip.v[register_x]);
                    chip.v[0xF] = no_borrow as u8;
                }
                _ => {}
            }
        }

        // Set index register ANNN
        0xA => {
            let data = instruction & 0x0FFF;

            chip.i = data;

            println!("Index register set to {:x}", data);
        }

        // Draw to display DXYN
        0xD => {
            // DXYN
            let register_x = (instruction & 0x0F00) >> 8;
            let register_y = (instruction & 0x00F0) >> 4;
            let n = (instruction & 0x000F) as usize;

            let vx = chip.v[register_x as usize] as usize;
            let vy = chip.v[register_y as usize] as usize;

            for row in 0..n {
                let byte = chip.memory[chip.i as usize + row];
                for bit in 0..8 {
                    let pixel = (byte >> (7 - bit)) & 1;
                    let x = (vx + bit) % WIDTH;
                    let y = (vy + row) % HEIGHT;
                    if pixel == 1 {
                        chip.display[y * WIDTH + x] ^= 0xFFFFFF;
                    }
                }
            }
        }
        _ => {}
    }

    chip.pc += 2;
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut chip = Chip8::default();

    if let Err(e) = write_program_to_memory("IBM Logo.ch8", &mut chip) {
        eprintln!("Warning: could not load ROM: {}", e);
    }
    write_font_to_memory(&mut chip);

    let mut window = Window::new("chip8", WIDTH * 10, HEIGHT * 10, WindowOptions::default())
        .unwrap_or_else(|e| panic!("{}", e));

    window.set_target_fps(60);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for _i in 0..10 {
            tick(&mut chip);
        }

        window.update_with_buffer(&chip.display, WIDTH, HEIGHT)?;
    }

    Ok(())
}
