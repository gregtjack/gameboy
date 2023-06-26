use std::{path::PathBuf, thread, time::Duration};

use clap::{arg, value_parser, Command};
use color_eyre::eyre::Result;
use gpu::{SCREEN_HEIGHT, SCREEN_WIDTH};
use pixels::{Pixels, SurfaceTexture};
use tracing::{debug, info};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

use crate::cpu::CPU;

mod cpu;
mod gpu;
mod memory;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let matches = cli().get_matches();

    info!("Starting Emulator");

    let mut cpu = CPU::new();

    // load game rom
    let bios = fs_read(
        matches
            .get_one::<PathBuf>("bios")
            .expect("Invalid BIOS path")
            .to_path_buf(),
    )?;

    cpu.load_bios(bios);

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

    let mut ticks = 0;

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => control_flow.set_exit(),
            _ => (),
        },
        Event::RedrawRequested(_) => {
            let cycles = cpu.step();
            for (i, pixel) in cpu.bus.gpu.screen.chunks(4).enumerate() {
                let offset = i * 4;
                pixels.frame_mut()[offset] = pixel[0];
                pixels.frame_mut()[offset + 1] = pixel[1];
                pixels.frame_mut()[offset + 2] = pixel[2];
                pixels.frame_mut()[offset + 3] = pixel[3];
            }
            thread::sleep(Duration::from_nanos(2))
        }
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        _ => (),
    });
}

fn fs_read(path: PathBuf) -> Result<Vec<u8>> {
    debug!("Loading bytes from path: {:?}", path);
    let bytes = std::fs::read(path)?;
    Ok(bytes)
}

fn cli() -> Command {
    Command::new("gameboy")
        .version("0.1.0")
        .author(clap::crate_authors!())
        .about("A Gameboy emulator written in Rust")
        .arg(
            arg!(-b --bios <FILE> "Path the the BIOS bin file")
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(arg!(-r --rom <FILE> "Path to the game ROM").value_parser(value_parser!(PathBuf)))
}
