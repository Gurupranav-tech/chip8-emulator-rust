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
    // delay timer
    pub timer1: u8,
    // sound timer
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
            display: [[0; 64]; 32],
            rnd_gen: rand::thread_rng(),
        };

        chip.memory[0x000..0x000 + FONTSET.len()].copy_from_slice(&FONTSET);
        chip.memory[0x200..0x200 + program.len()].copy_from_slice(&program);

        chip
    }

    pub fn execute(&mut self, keycodes: &[u8; 16]) {
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
                if self.vs[n2 as usize] != self.vs[n3 as usize] {
                    self.program_counter += 2;
                }
            }
            6 => {
                let val = ((n3 << 4) | n4) as u8;
                self.vs[n2 as usize] = val;
            }
            7 => {
                let val = ((n3 << 4) | n4) as u8;
                self.vs[n2 as usize] = self.vs[n2 as usize].wrapping_add(val);
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
                        let (res, carry) = self.vs[n2].overflowing_add(self.vs[n3]);
                        self.vs[n2] = res;
                        self.vs[0xF] = carry as u8;
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
            11 => self.program_counter = ((n2 << 8) | (n3 << 4) | n4) + self.vs[0] as u16,
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
            14 => {
                let code = self.vs[n2 as usize] as usize;
                if n3 == 9 && n4 == 14 && keycodes[code] == 1 {
                    self.program_counter += 2;
                } else if n3 == 10 && n4 == 1 && keycodes[code] == 0 {
                    self.program_counter += 2;
                }
            }
            15 => match n3 << 4 | n4 {
                0x55 => {
                    for i in 0..=n2 {
                        self.memory[(self.i + i) as usize] = self.vs[i as usize];
                    }
                }
                0x65 => {
                    for i in 0..=n2 {
                        self.vs[i as usize] = self.memory[(self.i + i) as usize];
                    }
                }
                0x1e => self.i += self.vs[n2 as usize] as u16,
                0x07 => self.vs[n2 as usize] = self.timer1,
                0x15 => self.timer1 = self.vs[n2 as usize],
                0x18 => self.timer2 = self.vs[n2 as usize],
                0x29 => {
                    let digit = self.vs[n2 as usize];
                    self.i = digit as u16 * 5;
                }
                0x0a => {
                    let mut key_pressed = None;
                    for (i, &key) in keycodes.iter().enumerate() {
                        if key != 0 {
                            key_pressed = Some(i as u8);
                            break;
                        }
                    }

                    match key_pressed {
                        Some(key) => {
                            self.vs[n2 as usize] = key;
                        }
                        None => {
                            self.program_counter -= 2;
                        }
                    }
                }
                0x33 => {
                    let val = self.vs[n2 as usize];
                    self.memory[self.i as usize] = val / 100;
                    self.memory[self.i as usize + 1] = (val / 10) % 10;
                    self.memory[self.i as usize + 2] = val % 10;
                }
                _ => println!("Unknown opcode in the instruction {inst:x}"),
            },
            _ => println!("Cannot decode unknown instruction {inst:x}"),
        };
        if self.timer2 != 0 {
            self.timer2 -= 1;
        }
        if self.timer1 != 0 {
            self.timer1 -= 1;
        }
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
