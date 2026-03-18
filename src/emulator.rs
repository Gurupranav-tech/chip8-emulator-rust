use std::io::stdin;

use sdl2::render::Texture;

use crate::chip8::Chip8;

#[derive(Clone, Debug)]
pub struct Emulator {
    chip: Chip8,
    debug: bool,
}

impl Emulator {
    pub fn new(program: Vec<u8>, debug: bool) -> Emulator {
        Emulator {
            chip: Chip8::new(program),
            debug,
        }
    }

    pub fn run(&mut self, texture: &mut Texture) {
        if self.debug {
            let (inst, _, _, _, _) = self.chip.instruction();

            println!("Cur Instruction: {inst:x}");
            println!("general registers {:x?}", self.chip.vs);
            println!("index register {:x}", self.chip.i);
            println!("program counter {:x}", self.chip.pc());
            println!("stack {:?}", self.chip.stack());
        }

        self.chip.execute();

        texture
            .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                for y in 0..32 {
                    for x in 0..64 {
                        let pixel = self.chip.display[y][x];
                        let color = if pixel == 1 { 255 } else { 0 };

                        let offset = y * pitch + x * 4;

                        buffer[offset] = color;
                        buffer[offset + 1] = color;
                        buffer[offset + 2] = color;
                        buffer[offset + 3] = 0;
                    }
                }
            })
            .expect("Error updating the texture");

        if self.debug {
            let _ = stdin().read_line(&mut "".to_string());
        }
    }
}
