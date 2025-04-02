use std::fs::File;
use std::io::{self, Read};

use rand::Rng;

use crate::DISPLAY_WIDTH;
use crate::DISPLAY_HEIGHT;
use crate::DISPLAY_SIZE;

const MEMORY_SIZE: usize = 4096;
const PROGRAM_START: usize = 0x200;

pub struct Chip8 {
    ram: [u8; MEMORY_SIZE],
    stack: Vec<usize>,
    display_buffer: [u32; DISPLAY_SIZE],
    pub update_window: bool,
    v: [u8; 16],
    keys: [bool; 16],
    delay: u8,
    sound: u8,
    i: usize,
    pc: usize,
    waiting_key: Option<u8>,
}

impl Chip8 {
    pub fn new() -> Self {
        let mut cpu = Self {
            ram: [0; MEMORY_SIZE],
            stack: Vec::new(),
            display_buffer: [0; DISPLAY_SIZE],
            update_window: false,
            v: [0; 16],
            keys: [false; 16],
            delay: 0,
            sound: 0,
            i: 0,
            pc: PROGRAM_START,
            waiting_key: None,
        };

        cpu.load_font();

        cpu
    }

    pub fn load_font(&mut self) {
        let font: [u8; 80] = [
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
            0xF0, 0x80, 0xF0, 0x80, 0x80  // F
        ];

        // Font data is loaded between 0x50 and 0x9F as a convention
        self.ram[0x50..0x50 + font.len()].copy_from_slice(&font);
    }

    pub fn load_rom(&mut self, filename: &str) -> io::Result<()> {
        let mut file = match File::open(filename) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Error opening file '{}': {}", filename, e);
                return Err(e);
            }
        };

        let mut buffer = Vec::new();
        if let Err(e) = file.read_to_end(&mut buffer) {
            eprintln!("Error reading file '{}': {}", filename, e);
            return Err(e);
        }

        if buffer.len() > MEMORY_SIZE - PROGRAM_START {
            return Err(io::Error::new(io::ErrorKind::Other, "ROM too large"));
        }

        self.ram[PROGRAM_START..PROGRAM_START + buffer.len()].copy_from_slice(&buffer);
        Ok(())
    }

    pub fn get_display_buffer(&self) -> &[u32] {
        &self.display_buffer
    }

    pub fn set_keys(&mut self, new_keys: [bool; 16]) {
        self.keys = new_keys;
    }

    pub fn decrement_timers(&mut self) {
        if self.delay > 0 {
            self.delay -= 1;
        }
        if self.sound > 0 {
            self.sound -= 1;
        }
    }

    fn fetch_next_instruction(&self) -> u16 {
        let high_byte = self.ram[self.pc] as u16;
        let low_byte = self.ram[self.pc + 1] as u16;

        (high_byte << 8) | low_byte
    }

    /// Runs a single CHIP-8 instruction.
    pub fn cycle(&mut self) {
        let opcode = self.fetch_next_instruction();
    
        let first_nibble = (opcode & 0xF000) >> 12;
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let n = (opcode & 0x000F) as usize;
        let nn = (opcode & 0x00FF) as u8;
        let nnn = opcode & 0x0FFF;
        
        match first_nibble {
            0x0=> match opcode {
                0x00E0 => self.op_00e0(),
                0x00EE => self.op_00ee(),
                _ => self.pc += 2, // this instruction is ignored
            },
            0x1 => self.op_1nnn(nnn),
            0x2 => self.op_2nnn(nnn),
            0x3 => self.op_3xnn(x, nn),
            0x4 => self.op_4xnn(x, nn),
            0x5 => self.op_5xy0(x, y),
            0x6 => self.op_6xnn(x, nn),
            0x7 => self.op_7xnn(x, nn),
            0x8 => match n {
                0x0 => self.op_8xy0(x, y),
                0x1 => self.op_8xy1(x, y),
                0x2 => self.op_8xy2(x, y),
                0x3 => self.op_8xy3(x, y),
                0x4 => self.op_8xy4(x, y),
                0x5 => self.op_8xy5(x, y),
                0x6 => self.op_8xy6(x),
                0x7 => self.op_8xy7(x, y),
                0xE => self.op_8xye(x),
                _ => self.op_unknown(opcode),
            },
            0x9 => self.op_9xy0(x, y),
            0xA => self.op_annn(nnn),
            0xB => self.op_bnnn(nnn),
            0xC => self.op_cxnn(x, nn),
            0xD => self.op_dxyn(x, y, n),
            0xE => match nn {
                0x9E => self.op_ex9e(x),
                0xA1 => self.op_exa1(x),
                _ => self.op_unknown(opcode),
            },
            0xF => match nn {
                0x07 => self.op_fx07(x),
                0x0A => self.op_fx0a(x),
                0x15 => self.op_fx15(x),
                0x18 => self.op_fx18(x),
                0x1E => self.op_fx1e(x),
                0x29 => self.op_fx29(x),
                0x33 => self.op_fx33(x),
                0x55 => self.op_fx55(x),
                0x65 => self.op_fx65(x),
                _ => self.op_unknown(opcode),
            },
            _ => self.op_unknown(opcode),
        }
    }

    /// Clears the screen. Turns all pixels to 0.
    fn op_00e0(&mut self) {
        for pixel in 0..DISPLAY_HEIGHT * DISPLAY_WIDTH {
            self.display_buffer[pixel] = 0;
        }
        self.update_window = true;
        self.pc += 2;
    }

    /// Returns from a subrotine.
    fn op_00ee(&mut self) {
        self.pc = self.stack.pop().unwrap();
    }

    /// Jumps to memory location `nnn`
    fn op_1nnn(&mut self, nnn: u16) {
        self.pc = nnn as usize;
    }

    /// Calls subroutine at memory location `nnn`
    fn op_2nnn(&mut self, nnn: u16) {
        self.stack.push(self.pc + 2);
        self.pc = nnn as usize;
    }

    /// Skips next instruction if `VX == nn`
    fn op_3xnn(&mut self, x: usize, nn: u8) {
        if self.v[x] == nn {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    /// Skips next instruction if `VX != nn`
    fn op_4xnn(&mut self, x: usize, nn: u8) {
        if self.v[x] != nn {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    /// Skips next instruction if `VX == VY`
    fn op_5xy0(&mut self, x: usize, y: usize) {
        if self.v[x] == self.v[y] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    /// Sets `VX` to `nn`
    fn op_6xnn(&mut self, x: usize, nn: u8) {
        self.v[x] = nn;
        self.pc += 2;
    }

    /// Adds `VX` to `nn`. Does not set VF in case of overflow.
    fn op_7xnn(&mut self, x: usize, nn: u8) {
        let result = self.v[x].overflowing_add(nn);
        self.v[x] = result.0;
        self.pc += 2;
    }

    /// Sets `VX` to `VY`
    fn op_8xy0(&mut self, x: usize, y: usize) {
        self.v[x] = self.v[y];
        self.pc += 2;
    }

    /// Sets `VX` to `VX OR VY`
    fn op_8xy1(&mut self, x: usize, y: usize) {
        self.v[x] |= self.v[y];
        self.pc += 2;
    }

    /// Sets `VX` to `VX AND VY`
    fn op_8xy2(&mut self, x: usize, y: usize) {
        self.v[x] &= self.v[y];
        self.pc += 2;
    }

    /// Sets `VX` to `VX XOR VY`
    fn op_8xy3(&mut self, x: usize, y: usize) {
        self.v[x] ^= self.v[y];
        self.pc += 2;
    }

    /// Sets `VX` to `VX + VY`. Sets the overflow flag `VF` to `1` in case of overflow, `0` otherwise.
    fn op_8xy4(&mut self, x: usize, y: usize) {
        let (sum, carry) = self.v[x].overflowing_add(self.v[y]);
        self.v[x] = sum;
        self.v[0xF] = carry as u8;
        self.pc += 2;
    }

    /// `VX` is set to `VX - VY`. Sets `VF` to `0` in case of underflow and `1` otherwise.
    fn op_8xy5(&mut self, x: usize, y: usize) {
        let (diff, borrow) = self.v[x].overflowing_sub(self.v[y]);
        self.v[x] = diff;
        self.v[0xF] = if borrow { 0 } else { 1 };
        self.pc += 2;
    }

    /// Shifts LSB of `VX` to `VF`
    fn op_8xy6(&mut self, x: usize) {
        let carry = self.v[x] & 1;
        self.v[x] >>= 1;
        self.v[0xf] = carry;
        self.pc += 2;
    }

    /// `VX` is set to `VY - VX`. Sets `VF` to `0` in case of underflow and `1` otherwise.
    fn op_8xy7(&mut self, x: usize, y: usize) {
        let (diff, borrow) = self.v[y].overflowing_sub(self.v[x]);
        self.v[x] = diff;
        self.v[0xF] = if borrow { 0 } else { 1 };
        self.pc += 2;
    }

    /// Shifts MSB of `VX` to `VF`
    fn op_8xye(&mut self, x: usize) {
        let carry = (self.v[x] & 0b10000000) >> 7;
        self.v[x] <<= 1;
        self.v[0xf] = carry;
        self.pc += 2;
    }

    /// Skips next instruction if `VX != VY`
    fn op_9xy0(&mut self, x: usize, y: usize) {
        if self.v[x] != self.v[y] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    /// Sets register `i` to `nnn`
    fn op_annn(&mut self, nnn: u16) {
        self.i = nnn as usize;
        self.pc += 2;
    }

    /// Jumps to memory location `nnn + V0`
    fn op_bnnn(&mut self, nnn: u16) {
        let v0 = self.v[0] as u16;
        self.pc = (nnn + v0) as usize;
    }

    /// Sets `VX` to `random_value AND nn`
    fn op_cxnn(&mut self, x: usize, nn: u8) {
        let mut rng = rand::rng();
        let random: u8 = rng.random();
        self.v[x] = random & nn;
        self.pc += 2;
    }

    /// Draws sprite to the screen. Objects out of bounds wrap around the screen.
    fn op_dxyn(&mut self, x: usize, y: usize, n: usize) {
        let x = self.v[x] as usize & 63;
        let y = self.v[y] as usize & 31;
        self.v[0xf] = 0;

        for row in 0..n {
            // Reached bottom edge of the screen
            if y + row >= 32 {
                break;
            }

            // Check for overflow
            if self.i + row >= MEMORY_SIZE { break; }

            let sprite_byte = self.ram[self.i as usize + row];

            for col in 0..8 {
                // Reached right edge of the screen
                if x + col >= 64 {
                    break;
                }

                let sprite_pixel = (sprite_byte >> (7 - col)) & 1;

                if sprite_pixel == 1 {
                    let screen_index = (y + row) * 64 + (x + col);

                    if self.display_buffer[screen_index] == 0x0000FF {
                        self.display_buffer[screen_index] = 0x000000;
                        self.v[0xf] = 1;
                    } else {
                        self.display_buffer[screen_index] = 0x0000FF;
                    }
                }
            }
        }

        self.update_window = true;
        self.pc += 2;
    }

    /// Skips next instruction if a key is pressed.
    fn op_ex9e(&mut self, x: usize) {
        let key = self.v[x] & 0x0F;
        if self.keys[key as usize] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    /// Skips next instruction if a key is not pressed.
    fn op_exa1(&mut self, x: usize) {
        let key = self.v[x] & 0x0F;
        if !self.keys[key as usize] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    /// Sets `VX` to the delay timer.
    fn op_fx07(&mut self, x: usize) {
        self.v[x] = self.delay;
        self.pc += 2;
    }

    /// Waits for a key to be released. When it is released sets `VX` to it.
    fn op_fx0a(&mut self, x: usize) {
        if let Some(key) = self.waiting_key {
            if !self.keys[key as usize] {
                self.v[x] = key;
                self.waiting_key = None;
                self.pc += 2;
            }
        } 
        else if let Some(pressed_key) = self.keys.iter().position(|&state| state) {
            self.waiting_key = Some(pressed_key as u8);
        }
    }

    /// Sets delay timer to `VX`
    fn op_fx15(&mut self, x: usize) {
        self.delay = self.v[x];
        self.pc += 2;
    }

    /// Sets sound timer to `VX`
    fn op_fx18(&mut self, x: usize) {
        self.sound = self.v[x];
        self.pc += 2;
    }

    /// Sets `i` register to `i + VX`
    fn op_fx1e(&mut self, x: usize) {
        self.i += self.v[x] as usize;
        self.pc += 2;
    }

    /// Sets `i` register to font pointed to by `VX`
    fn op_fx29(&mut self, x: usize) {
        let digit = self.v[x] & 0x0F;
        self.i = 0x50 + (digit as usize * 5);
        self.pc += 2;
    }

    /// Splits `VX` into its constituent decimal digits and stores it into memory
    fn op_fx33(&mut self, x: usize) {
        self.ram[self.i] = self.v[x] / 100;
        self.ram[self.i + 1] = (self.v[x] % 100) / 10;
        self.ram[self.i + 2] = self.v[x] % 10;
        self.pc += 2;
    }

    /// Stores the contents of the `V` registers up to `x` into memory
    fn op_fx55(&mut self, x: usize) {
        for i in 0..x + 1 {
            self.ram[self.i + i] = self.v[i];
        }
        self.pc += 2;
    }

    /// Extracts memory onto the `V` registers up to `x`.
    fn op_fx65(&mut self, x: usize) {
        for i in 0..x + 1 {
            self.v[i] = self.ram[self.i + i];
        }
        self.pc += 2;
    }

    /// Unknown insruction. Does nothing.
    fn op_unknown(&mut self, opcode: u16) {
        println!("Spooky unknwon instruction with opcode: {:#X}", opcode);
        self.pc += 2;
    }
}
