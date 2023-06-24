use std::path::PathBuf;

use clap::{arg, value_parser, Command};
use color_eyre::eyre::Result;
use cpu::CPU;
use gpu::{SCREEN_HEIGHT, SCREEN_WIDTH};
use tracing::{debug, info};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

mod cpu;
mod gpu;
mod instruction;
mod mmu;
mod registers;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let matches = cli().get_matches();

    info!("Starting Emulator");

    let mut cpu = CPU::new();

    // load game rom
    let rom = load_rom(
        matches
            .get_one::<PathBuf>("rom")
            .expect("Invalid ROM path")
            .to_path_buf(),
    )?;

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Gameboy Emulator")
        .with_inner_size(winit::dpi::LogicalSize::new(
            SCREEN_WIDTH as f64,
            SCREEN_HEIGHT as f64,
        ))
        .build(&event_loop)?;

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();
        control_flow.set_wait();

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => control_flow.set_exit(),
                _ => (),
            },
            _ => (),
        }
    });
}

fn load_rom(path: PathBuf) -> Result<Vec<u8>> {
    debug!("Loading game rom from path: {:?}", path);
    let rom = std::fs::read(path)?;
    Ok(rom)
}

fn cli() -> Command {
    Command::new("gameboy")
        .version("0.1.0")
        .author(clap::crate_authors!())
        .about("A Gameboy emulator written in Rust")
        .arg(arg!(-r --rom <FILE> "The game rom to load").value_parser(value_parser!(PathBuf)))
}
