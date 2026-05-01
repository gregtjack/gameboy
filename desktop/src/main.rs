use clap::{arg, value_parser, Command};
use log::{debug, error, info};
use pixels::{Pixels, SurfaceTexture};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use gameboy::gameboy::{Gameboy, SCREEN_HEIGHT, SCREEN_WIDTH};
use gameboy::joypad::Key;
use gameboy::Theme;
mod utils;

const GAMEBOY_UPDATE_INTERVAL: Duration = Duration::from_nanos(1_000_000_000 / 60);
const MAX_ACCUMULATED_TIME: Duration = Duration::from_millis(250);

struct Emulator {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    gameboy: Gameboy,
    last_update: Instant,
    accumulated_time: Duration,
}

impl ApplicationHandler for Emulator {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = {
            let min_size = LogicalSize::new(SCREEN_WIDTH as f64 * 2.0, SCREEN_HEIGHT as f64 * 2.0);
            let size = LogicalSize::new(SCREEN_WIDTH as f64 * 3.0, SCREEN_HEIGHT as f64 * 3.0);
            Arc::new(
                event_loop
                    .create_window(
                        Window::default_attributes()
                            .with_title("Gameboy Emulator")
                            .with_inner_size(size)
                            .with_min_inner_size(min_size),
                    )
                    .unwrap(),
            )
        };

        self.window = Some(window.clone());
        self.pixels = {
            let window_size = window.inner_size();
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, window.clone());
            match Pixels::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, surface_texture) {
                Ok(pixels) => {
                    // Kick off the redraw loop
                    window.request_redraw();

                    Some(pixels)
                }
                Err(err) => {
                    error!("pixels::new {err}");
                    event_loop.exit();

                    None
                }
            }
        };
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                info!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Err(err) = self
                    .pixels
                    .as_mut()
                    .unwrap()
                    .resize_surface(size.width, size.height)
                {
                    error!("pixels.resize_surface {err}");
                    event_loop.exit();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(keycode) = event.physical_key {
                    // Theme switching shortcuts
                    if event.state == ElementState::Pressed {
                        match keycode {
                            KeyCode::Digit1 => {
                                self.gameboy.set_theme(Theme::Grayscale);
                                debug!("Theme switched to Grayscale");
                            }
                            KeyCode::Digit2 => {
                                self.gameboy.set_theme(Theme::Green);
                                debug!("Theme switched to Green");
                            }
                            KeyCode::Digit3 => {
                                self.gameboy.set_theme(Theme::PurpleYellow);
                                debug!("Theme switched to Purple/Yellow");
                            }
                            KeyCode::KeyP => {
                                self.gameboy.pause();
                                debug!("Emulation paused");
                            }
                            KeyCode::KeyR => {
                                self.gameboy.resume();
                                debug!("Emulation resumed")
                            }
                            _ => {}
                        }
                    }

                    // Game Boy joypad keys
                    if let Some(key) = map_key(keycode) {
                        match event.state {
                            ElementState::Pressed => self.gameboy.press_key(key),
                            ElementState::Released => self.gameboy.release_key(key),
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                self.advance_emulation();

                let frame = self.pixels.as_mut().unwrap().frame_mut();
                let screen = self.gameboy.screen();
                for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
                    let x = (i % SCREEN_WIDTH as usize) as i16;
                    let y = (i / SCREEN_WIDTH as usize) as i16;
                    let color = screen[x as usize][y as usize];
                    pixel.copy_from_slice(&self.gameboy.get_color_rgba(color));
                }

                if let Err(err) = self.pixels.as_ref().unwrap().render() {
                    error!("pixels.render {err}");
                    event_loop.exit();
                } else {
                    // Queue a redraw for the next frame
                    self.window.as_ref().unwrap().request_redraw();
                }
            }
            _ => (),
        }
    }
}

impl Emulator {
    fn new(gameboy: Gameboy) -> Self {
        Self {
            window: None,
            pixels: None,
            gameboy,
            last_update: Instant::now(),
            accumulated_time: Duration::ZERO,
        }
    }

    fn advance_emulation(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update);
        self.last_update = now;
        self.accumulated_time += elapsed.min(MAX_ACCUMULATED_TIME);

        while self.accumulated_time >= GAMEBOY_UPDATE_INTERVAL {
            self.gameboy.update();
            self.accumulated_time -= GAMEBOY_UPDATE_INTERVAL;
        }
    }
}

/// Maps keyboard keys to Game Boy joypad keys
fn map_key(keycode: KeyCode) -> Option<Key> {
    match keycode {
        KeyCode::ArrowUp => Some(Key::Up),
        KeyCode::ArrowDown => Some(Key::Down),
        KeyCode::ArrowLeft => Some(Key::Left),
        KeyCode::ArrowRight => Some(Key::Right),
        KeyCode::KeyZ => Some(Key::A),
        KeyCode::KeyX => Some(Key::B),
        KeyCode::Enter => Some(Key::Start),
        KeyCode::ShiftRight | KeyCode::ShiftLeft => Some(Key::Select),
        _ => None,
    }
}

fn main() {
    env_logger::init();
    let matches = cli().get_matches();
    let mut gameboy = Gameboy::new(matches.get_flag("debug"));

    // load game rom
    info!("Loading ROM");
    let rom = utils::fs::read(
        matches
            .get_one::<PathBuf>("rom")
            .expect("Should provide a game rom path")
            .to_path_buf(),
    )
    .expect("Should be able to read the game rom");

    gameboy.load_rom(rom);
    info!("ROM loaded");

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = Emulator::new(gameboy);

    event_loop.run_app(&mut app).unwrap()
}

fn cli() -> Command {
    Command::new("gameboy")
        .version("0.1.0")
        .author(clap::crate_authors!())
        .about("A Gameboy emulator written in Rust")
        .arg(arg!(-r --rom <FILE> "Path to the game ROM").value_parser(value_parser!(PathBuf)))
        .arg(arg!(-d --debug "Turn on debugging mode"))
}
