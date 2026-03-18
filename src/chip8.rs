use rand::{Rng, rngs::ThreadRng};

const FONTSET: [u8; 80] = [
    // 0
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 1
    0x20, 0x60, 0x20, 0x20, 0x70, // 2
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 3
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 4
    0x90, 0x90, 0xF0, 0x10, 0x10, // 5
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 6
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 7
    0xF0, 0x10, 0x20, 0x40, 0x40, // 8
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // A
    0xF0, 0x90, 0xF0, 0x90, 0x90, // B
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // C
    0xF0, 0x80, 0x80, 0x80, 0xF0, // D
    0xE0, 0x90, 0x90, 0x90, 0xE0, // E
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // F
    0xF0, 0x80, 0xF0, 0x80, 0x80,
];

#[derive(Clone, Debug)]
pub struct Chip8 {
    // general purpose registers
    pub vs: [u8; 16],
    pub i: u16,

    // Timer registers
    pub timer1: u8,
    pub timer2: u8,

    // Special Registers
    program_counter: u16,
    stack: Vec<u16>,

    // Memory
    memory: [u8; 4096],
    pub display: [[u8; 64]; 32],

    // random number generator
    rnd_gen: ThreadRng,
}

impl Chip8 {
    pub fn new(program: Vec<u8>) -> Chip8 {
        let mut chip = Chip8 {
            vs: [0; 16],
            i: 0,
            timer1: 0,
            timer2: 0,
            program_counter: 0x200,
            stack: Vec::new(),
            memory: [0; 4096],
            display: [[1; 64]; 32],
            rnd_gen: rand::thread_rng(),
        };

        chip.memory[0x000..0x000 + FONTSET.len()].copy_from_slice(&FONTSET);
        chip.memory[0x200..0x200 + program.len()].copy_from_slice(&program);

        chip
    }

    pub fn execute(&mut self) {
        let (inst, n1, n2, n3, n4) = self.instruction();
        self.program_counter += 2;

        match n1 {
            0 => {
                if n2 == 0 && n3 == 0xe && n4 == 0 {
                    self.cls();
                }
                if n2 == 0 && n3 == 0xe && n4 == 0xe {
                    self.program_counter =
                        self.stack.pop().expect("Empty stack, nowhere to return to")
                }
            }
            1 => self.program_counter = n2 << 8 | n3 << 4 | n4,
            2 => {
                if self.stack.len() >= 16 {
                    println!("Maximum stack size reached");
                    return;
                }
                self.stack.push(self.program_counter);
                self.program_counter = n2 << 8 | n3 << 4 | n4;
            }
            3 => {
                let val = n3 << 4 | n4;
                if self.vs[n2 as usize] as u16 == val {
                    self.program_counter += 2;
                }
            }
            4 => {
                let val = n3 << 4 | n4;
                if self.vs[n2 as usize] as u16 != val {
                    self.program_counter += 2;
                }
            }
            5 => {
                if self.vs[n2 as usize] == self.vs[n3 as usize] {
                    self.program_counter += 2;
                }
            }
            9 => {
                if self.vs[n2 as usize] == self.vs[n3 as usize] {
                    self.program_counter += 2;
                }
            }
            6 => {
                let val = ((n3 << 4) | n4) as u8;
                self.vs[n2 as usize] = val;
            }
            7 => {
                let val = ((n3 << 4) | n4) as u8;
                self.vs[n3 as usize] = ((self.vs[n3 as usize] as u32 + val as u32) % 0xff) as u8;
            }
            8 => {
                let n2 = n2 as usize;
                let n3 = n3 as usize;

                match n4 {
                    0 => self.vs[n2] = self.vs[n3],
                    1 => self.vs[n2] |= self.vs[n3],
                    2 => self.vs[n2] &= self.vs[n3],
                    3 => self.vs[n2] ^= self.vs[n3],
                    4 => {
                        self.vs[0xf] = (self.vs[n2] as u16 + self.vs[n3] as u16 > 0xff) as u8;
                        self.vs[n2] = ((self.vs[n2] as u16 + self.vs[n3] as u16) % 0xff) as u8;
                    }
                    5 => {
                        self.vs[n2] = self.vs[n2].wrapping_sub(self.vs[n3]);
                    }
                    7 => {
                        self.vs[0xf] = (self.vs[n2] >= self.vs[n3]) as u8;
                        self.vs[n2] = self.vs[n2].wrapping_sub(self.vs[n3]);
                    }
                    6 => {
                        self.vs[0xF] = self.vs[n2] & 0x1;
                        self.vs[n2] >>= 1;
                    }
                    14 => {
                        self.vs[0xF] = (self.vs[n2] & 0x80) >> 7;
                        self.vs[n2] <<= 1;
                    }
                    _ => println!("Unknown last opcode for instruction {inst:x}"),
                }
            }
            10 => self.i = (n2 << 8) | (n3 << 4) | n4,
            11 => self.program_counter = (n2 << 8) | (n3 << 4) | n4 + self.vs[n2 as usize] as u16,
            12 => {
                let val = (n3 << 4 | n4) as u8;
                self.vs[n2 as usize] = val & self.rnd_gen.gen_range(0..=0xff) as u8;
            }
            13 => {
                let vx = self.vs[n2 as usize] as usize;
                let vy = self.vs[n3 as usize] as usize;
                let height = n4 as usize;

                self.vs[0xF] = 0;
                for row in 0..height {
                    let sprite_byte = self.memory[(self.i + row as u16) as usize];

                    for col in 0..8 {
                        let sprite_pixel = (sprite_byte >> (7 - col)) & 1;

                        if sprite_pixel == 0 {
                            continue;
                        }

                        let x = (vx + col) % 64;
                        let y = (vy + row) % 32;

                        let screen_pixel = &mut self.display[y][x];

                        if *screen_pixel == 1 {
                            self.vs[0xF] = 1;
                        }

                        *screen_pixel ^= 1;
                    }
                }
            }
            

            _ => println!("Cannot decode unknown instruction {inst:x}"),
        };
    }

    pub fn cls(&mut self) {
        for y in 0..32 {
            for x in 0..64 {
                self.display[y][x] = 0;
            }
        }
    }

    pub fn instruction(&self) -> (u16, u16, u16, u16, u16) {
        let upper = self.memory[self.program_counter as usize] as u16;
        let lower = self.memory[self.program_counter as usize + 1] as u16;
        let inst: u16 = upper << 8 | lower;

        let n1 = (inst & 0xF000) >> 12;
        let n2 = (inst & 0x0F00) >> 8;
        let n3 = (inst & 0x00F0) >> 4;
        let n4 = inst & 0x000F;

        (inst, n1, n2, n3, n4)
    }

    pub fn pc(&self) -> u16 {
        self.program_counter
    }

    pub fn stack(&self) -> &Vec<u16> {
        &self.stack
    }
}
