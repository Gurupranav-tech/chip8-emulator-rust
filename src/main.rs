use chip_8::emulator::Emulator;
use core::time;
use sdl2::{
    event::Event, keyboard::Keycode, pixels::PixelFormatEnum, rect::Rect, render::TextureAccess,
};
use std::{env, fs, thread::sleep};

extern crate sdl2;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("Invalid Usage of Emulator");
        return;
    }

    let program = fs::read(args[1].clone()).expect("Cannot open the file provided");
    let debug = args[2] == "debug";

    let mut emulator = Emulator::new(program, debug);

    let sdl = sdl2::init().expect("Cannot intialize SDL");
    let video_subsystem = sdl.video().expect("Cannot intialize video");

    let window = video_subsystem
        .window("CHIP-8", 1024, 512)
        .position_centered()
        .build()
        .expect("Cannot create window");

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture(PixelFormatEnum::RGB888, TextureAccess::Streaming, 64, 32)
        .expect("Cannot create texture");
    let target_rect = Rect::new(0, 0, 1024, 512);

    let mut event_pump = sdl.event_pump().unwrap();

    'running: loop {
        emulator.run(&mut texture);

        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        canvas.copy(&texture, None, Some(target_rect)).unwrap();
        canvas.present();

        sleep(time::Duration::from_millis(16));
    }
}
