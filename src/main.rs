use std::{path::PathBuf, thread, time::Duration};

use clap::{arg, value_parser, Command};
use color_eyre::eyre::Result;
use gpu::{SCREEN_HEIGHT, SCREEN_WIDTH};
use pixels::{Pixels, SurfaceTexture};
use tracing::{debug, info};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::{cpu::Cpu, emulator::Gameboy};

mod apu;
mod clock;
mod cpu;
mod emulator;
mod fs;
mod gpu;
mod joypad;
mod memory;
mod timer;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let matches = cli().get_matches();

    info!("Starting Gameboy");

    // load game rom
    let rom = fs::read(
        matches
            .get_one::<PathBuf>("rom")
            .expect("Invalid game rom path")
            .to_path_buf(),
    )?;

    let mut gb = Gameboy::new();
    gb.load_rom(rom);

    let event_loop = EventLoop::new();
    let window = {
        let size = LogicalSize::new(SCREEN_WIDTH as f64, SCREEN_HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Gameboy Emulator")
            .with_inner_size(size)
            .build(&event_loop)?
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, surface_texture)?
    };

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => control_flow.set_exit(),
                _ => (),
            },
            Event::MainEventsCleared => {
                gb.update();
                let frame = pixels.frame_mut();
                for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
                    let screen = gb.get_screen();
                    let r = screen[i];
                    let g = screen[i + 1];
                    let b = screen[i + 2];
                    pixel.copy_from_slice(&[r, g, b, 1]);
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
}
