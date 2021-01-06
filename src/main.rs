extern crate sdl2;

use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use ws::{connect, Message};

pub const WIDTH: usize = 1872;
pub const HEIGHT: usize = 1404;
pub const BYTES_PER_PIXEL: usize = 1;
pub const WINDOW_BYTES: usize = WIDTH * HEIGHT * BYTES_PER_PIXEL;

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("rust-sdl2 demo", WIDTH as u32 / 2, HEIGHT as u32 / 2)
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_scale(0.5, 0.5).unwrap();

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let (tx, rx) = channel();

    let arc_tx = Arc::new(Mutex::new(tx));

    thread::spawn(move || {
        connect("ws://192.168.1.41:4444", |_out| {
            let a = &arc_tx;
            move |data: Message| {
                println!("Received!");
                a.lock().unwrap().send(data.into_data()).unwrap();
                Result::Ok(())
            }
        }).unwrap();
    });

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                }
                _ => {}
            }
        }

        if let Ok(data) = rx.recv_timeout(Duration::from_millis(1000 / 60)) {
            println!("Arrived!");
            let mut window = [0u8; WINDOW_BYTES];
            canvas.set_draw_color(Color::RGB(255, 255, 255));
            canvas.clear();
            let mut iter = data.iter().peekable();
            while iter.peek().is_some() {
                let mut i = {
                    let a = *iter.next().unwrap() as i32;
                    let b = *iter.next().unwrap() as i32;
                    let c = *iter.next().unwrap() as i32;
                    a << 16 | b << 8 | c
                };
                while let Some(pixel) = iter.next() {
                    if *pixel == 255 { break; }
                    canvas.set_draw_color(Color::RGB(*pixel, *pixel, *pixel));
                    // canvas.set_draw_color(Color::RGB(0, 0, 0));

                    let h = i / WIDTH as i32;
                    let w = i % WIDTH as i32;
                    canvas.draw_point(Point::new( h, WIDTH as i32 - w)).unwrap();
                    window[i as usize] = *pixel;
                    i += 1;
                }
            }
            canvas.present();
            println!("Processed!");
        }

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
