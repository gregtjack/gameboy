use anyhow::Result;
use clap::{arg, value_parser, Command};
use pixels::{Pixels, SurfaceTexture};
use std::path::PathBuf;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use libgb::gameboy::{Gameboy, SCREEN_HEIGHT, SCREEN_WIDTH};

mod utils;

fn main() {
    let matches = cli().get_matches();

    println!("Starting Gameboy");

    let mut gameboy = Gameboy::new(matches.get_flag("debug"));

    // load game rom
    let rom = utils::fs::read(
        matches
            .get_one::<PathBuf>("rom")
            .expect("Invalid game rom path")
            .to_path_buf(),
    )
    .unwrap();

    gameboy.load_rom(rom);

    println!("Loaded ROM");

    let event_loop = EventLoop::new();
    let window = {
        WindowBuilder::new()
            .with_title("Gameboy")
            .with_inner_size(LogicalSize::new(
                SCREEN_WIDTH as f64 * 2.0,
                SCREEN_HEIGHT as f64 * 2.0,
            ))
            .build(&event_loop)
            .expect("Window should have been initialized successfully")
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, surface_texture).unwrap()
    };

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        gameboy.update();
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => control_flow.set_exit(),
                WindowEvent::Resized(size) => {
                    pixels.resize_surface(size.width, size.height).unwrap();
                }
                _ => (),
            },
            Event::MainEventsCleared => {
                let frame = pixels.frame_mut();
                let screen = gameboy.frame();
                for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
                    let x = (i % SCREEN_WIDTH as usize) as i16;
                    let y = (i / SCREEN_WIDTH as usize) as i16;
                    let [r, g, b, a] = screen[x as usize][y as usize].rgba();
                    pixel.copy_from_slice(&[r, g, b, a]);
                }

                if let Err(err) = pixels.render() {
                    eprintln!("Error rendering pixels: {}", err);
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                    return;
                }
            }
            _ => (),
        }
    });
}

fn cli() -> Command {
    Command::new("gameboy")
        .version("0.1.0")
        .author(clap::crate_authors!())
        .about("A Gameboy emulator written in Rust")
        .arg(arg!(-r --rom <FILE> "Path to the game ROM").value_parser(value_parser!(PathBuf)))
        .arg(arg!(-d --debug "Turn on debugging mode"))
}
