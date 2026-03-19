use chip_8::{SquareWave, emulator::Emulator};
use sdl2::{
    audio::AudioSpecDesired, event::Event, keyboard::Keycode, pixels::PixelFormatEnum, rect::Rect, render::TextureAccess
};
use std::{env, fs};

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
    let mut keycodes = [0 as u8; 16];

    let audio_subsystem = sdl.audio().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: None,
    };

    let device = audio_subsystem
        .open_playback(None, &desired_spec, |spec| {
            SquareWave::new(440.0 / spec.freq as f32, 0.0, 0.25)
        })
        .unwrap();

    'running: loop {
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(code),
                    ..
                } => {
                    let idx = match code {
                        Keycode::Num1 => Some(0x1),
                        Keycode::Num2 => Some(0x2),
                        Keycode::Num3 => Some(0x3),
                        Keycode::Num4 => Some(0x4),
                        Keycode::Num5 => Some(0x5),
                        Keycode::Num6 => Some(0x6),
                        Keycode::Num7 => Some(0x7),
                        Keycode::Num8 => Some(0x8),
                        Keycode::Num9 => Some(0x9),
                        Keycode::A => Some(0xa),
                        Keycode::B => Some(0xb),
                        Keycode::C => Some(0xc),
                        Keycode::D => Some(0xd),
                        Keycode::E => Some(0xe),
                        Keycode::F => Some(0xf),
                        _ => None,
                    }
                    .unwrap_or(0);
                    keycodes[idx] = 1;
                }
                Event::KeyUp {
                    keycode: Some(code),
                    ..
                } => {
                    let idx = match code {
                        Keycode::Num1 => Some(0x1),
                        Keycode::Num2 => Some(0x2),
                        Keycode::Num3 => Some(0x3),
                        Keycode::Num4 => Some(0x4),
                        Keycode::Num5 => Some(0x5),
                        Keycode::Num6 => Some(0x6),
                        Keycode::Num7 => Some(0x7),
                        Keycode::Num8 => Some(0x8),
                        Keycode::Num9 => Some(0x9),
                        Keycode::A => Some(0xa),
                        Keycode::B => Some(0xb),
                        Keycode::C => Some(0xc),
                        Keycode::D => Some(0xd),
                        Keycode::E => Some(0xe),
                        Keycode::F => Some(0xf),
                        _ => None,
                    }
                    .unwrap_or(0);
                    keycodes[idx] = 0;
                }
                _ => {}
            }
        }

        emulator.run(&mut texture, &keycodes);

        if emulator.need_sound() {
            device.resume();
        } else {
            device.pause();
        }

        canvas.copy(&texture, None, Some(target_rect)).unwrap();
        canvas.present();
    }
}
