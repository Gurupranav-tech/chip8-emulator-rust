#[derive(Clone, Debug)]
pub struct Emulator {}

impl Emulator {
    pub fn new() -> Emulator {
        Emulator {}
    }

    pub fn run(&self) {
        println!("Hello, from the emulator");
    }
}
