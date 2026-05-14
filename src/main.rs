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
}

impl Default for Chip8 {
    fn default() -> Self {
        Self {
            memory: [0; 4096],
            pc: 0x200,
            display: [0; WIDTH * HEIGHT],
            v: [0; 16],
            i: 0
        }
    }
}

fn write_font_to_memory(
    chip: &mut Chip8
) {
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

fn write_test_program_to_memory(
    chip: &mut Chip8,
) {

    // 1010-0000
    // 0000-0000  0000-0000

    let bytes = vec![
        0x60, 0x69,
        0xA0, 0x00,
        0xA0, 0x00,
    ];

    chip.memory[0x200..0x200 + bytes.len()].copy_from_slice(&bytes);
}

fn tick(chip: &mut Chip8) {
    // 1010 0000 0000 0000
    let instruction = (chip.memory[chip.pc] as u16) << 8 | chip.memory[chip.pc + 1] as u16;

    // 0000 1010
    let opcode = ((instruction & 0xF000) >> 12) as u8;
    match opcode {
        // Clear the screen 00E0
        0x0 => {
            let data = ((instruction & 0x0F00) >> 8) as u8;
            if data == 0xE {
                for i in 0..WIDTH * HEIGHT {
                    chip.display[i] = 0;
                }
            }
        }

        // Jump 1NNN
        0x1 => {
            let data = instruction & 0x0FFF;
            chip.pc = data as usize;

            println!("Jumped to {:x}", data);
            return;
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

    let mut window = Window::new(
        "chip8",
        WIDTH * 10,
        HEIGHT * 10,
        WindowOptions::default(),
    )
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
