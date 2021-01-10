use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex, RwLock,
};

use clap::{Arg, ArgAction, Command};
use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::ElementState,
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};

mod emu_thread;

use gbemu::{
    cpu::CPU,
    display::Display,
    interrupt::{InterruptController, Keys},
    mbc, memory,
    serial::StdoutSerialWrite,
    PPU, SCREEN_HEIGHT, SCREEN_WIDTH,
};

const MULTIPLIER: u32 = 4;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * MULTIPLIER;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * MULTIPLIER;

const TILE_WINDOW_WIDTH: u32 = 20 * 8;
const TILE_WINDOW_HEIGHT: u32 = 20 * 8;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let matches = Command::new("Gameboy Emulator")
        .version("0.1")
        .author("Paul Cacheux <paulcacheux@gmail.com>")
        .arg(
            Arg::new("TILES_WINDOW")
                .short('t')
                .long("tiles")
                .action(ArgAction::SetTrue)
                .help("Display the tiles data in a separate window"),
        )
        .arg(
            Arg::new("BOOTSTRAP_ROM")
                .short('b')
                .long("bootstrap")
                .value_name("BOOTSTRAP_ROM_PATH")
                .action(ArgAction::Set)
                .help("Sets the path to a bootstrap rom used to init the Gameboy emulator state."),
        )
        .arg(
            Arg::new("ROM_PATH")
                .required(true)
                .index(1)
                .action(ArgAction::Set)
                .help("Sets the path to the ROM to play on the Gameboy emulator."),
        )
        .get_matches();

    let bootstrap = if let Some(mut bootstrap_path) = matches.get_raw("BOOTSTRAP_ROM") {
        let path = bootstrap_path.next().unwrap();
        Some(std::fs::read(path)?)
    } else {
        None
    };

    let rom_path = matches.get_raw("ROM_PATH").unwrap().next().unwrap();
    let rom = std::fs::read(rom_path)?;

    let interrupt_controller = Arc::new(Mutex::new(InterruptController::new()));

    let mbc = mbc::read_cartridge(&rom);
    let mut mmu = memory::MMU::new(
        mbc,
        interrupt_controller.clone(),
        Box::new(StdoutSerialWrite),
    );
    if let Some(bootstrap) = &bootstrap {
        mmu.write_bootstrap_rom(bootstrap);
    } else {
        mmu.unmount_bootstrap_rom();
    }

    let memory = Arc::new(RwLock::new(mmu));
    let display = Arc::new(Mutex::new(Display::default()));

    let mut cpu = CPU::new(memory.clone(), interrupt_controller.clone());
    if bootstrap.is_none() {
        cpu.manual_bootstrap();
    }
    let ppu = PPU::new(
        memory.clone(),
        interrupt_controller.clone(),
        display.clone(),
    );

    let is_ended = Arc::new(AtomicBool::new(false));
    let is_ended_emu = is_ended.clone();
    let _ = std::thread::spawn(move || {
        emu_thread::run(cpu, ppu, is_ended_emu);
    });

    let event_loop = EventLoop::new()?;

    let mut main_window_data = {
        let window = {
            let size = LogicalSize::new(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64);
            WindowBuilder::new()
                .with_title("GameBoy Emulator")
                .with_inner_size(size)
                .with_resizable(false)
                .build(&event_loop)
                .unwrap()
        };

        let framebuffer = {
            let window_physical_size = window.inner_size();
            let surface_texture = SurfaceTexture::new(
                window_physical_size.width,
                window_physical_size.height,
                &window,
            );
            Pixels::new(SCREEN_WIDTH as _, SCREEN_HEIGHT as _, surface_texture)?
        };

        WindowData {
            window,
            framebuffer,
        }
    };

    let mut tiles_window_data = if matches.get_flag("TILES_WINDOW") {
        let window = {
            let size = LogicalSize::new(
                (TILE_WINDOW_WIDTH * MULTIPLIER) as f64,
                (TILE_WINDOW_HEIGHT * MULTIPLIER) as f64,
            );
            WindowBuilder::new()
                .with_title("GameBoy Emulator Tiles")
                .with_inner_size(size)
                .with_resizable(false)
                .build(&event_loop)
                .unwrap()
        };

        let framebuffer = {
            let window_physical_size = window.inner_size();
            let surface_texture = SurfaceTexture::new(
                window_physical_size.width,
                window_physical_size.height,
                &window,
            );
            Pixels::new(TILE_WINDOW_WIDTH, TILE_WINDOW_HEIGHT, surface_texture)?
        };

        Some(WindowData {
            window,
            framebuffer,
        })
    } else {
        None
    };

    event_loop.run(move |event, loop_proxy| {
        use winit::event::{Event, WindowEvent};

        loop_proxy.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent {
                window_id,
                event: WindowEvent::RedrawRequested,
            } if window_id == main_window_data.window.id() => {
                display
                    .lock()
                    .unwrap()
                    .draw_into_fb(main_window_data.framebuffer.frame_mut());
                let _ = main_window_data.framebuffer.render();
            }
            Event::WindowEvent {
                window_id,
                event: WindowEvent::RedrawRequested,
            } if Some(window_id) == tiles_window_data.as_ref().map(|d| d.window.id()) => {
                if let Some(data) = tiles_window_data.as_mut() {
                    Display::draw_tiles_into_fb(&memory, data.framebuffer.frame_mut());
                    let _ = data.framebuffer.render();
                }
            }
            Event::AboutToWait => {
                let mut int_cont = interrupt_controller.lock().unwrap();
                if int_cont.should_redraw {
                    main_window_data.window.request_redraw();
                    if let Some(data) = tiles_window_data.as_ref() {
                        data.window.request_redraw();
                    }
                    int_cont.should_redraw = false;
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                loop_proxy.exit();
            }

            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { event, .. },
                ..
            } => {
                if let PhysicalKey::Code(vkc) = event.physical_key {
                    // let pressed = input.state == ElementState::Pressed;
                    let pressed: bool = match event.state {
                        ElementState::Pressed => true,
                        ElementState::Released => false,
                    };
                    let mut int = interrupt_controller.lock().unwrap();

                    match vkc {
                        KeyCode::Escape => {
                            loop_proxy.exit();
                        }
                        KeyCode::KeyZ | KeyCode::ArrowUp => {
                            int.change_key_state(Keys::Up, pressed);
                        }
                        KeyCode::KeyQ | KeyCode::ArrowLeft => {
                            int.change_key_state(Keys::Left, pressed);
                        }
                        KeyCode::KeyS | KeyCode::ArrowDown => {
                            int.change_key_state(Keys::Down, pressed);
                        }
                        KeyCode::KeyD | KeyCode::ArrowRight => {
                            int.change_key_state(Keys::Right, pressed);
                        }

                        KeyCode::KeyO => {
                            int.change_key_state(Keys::A, pressed);
                        }
                        KeyCode::KeyP => {
                            int.change_key_state(Keys::B, pressed);
                        }

                        KeyCode::Enter => {
                            int.change_key_state(Keys::Start, pressed);
                        }
                        KeyCode::ControlLeft => {
                            int.change_key_state(Keys::Select, pressed);
                        }
                        _ => {}
                    }
                }
            }
            Event::LoopExiting => {
                is_ended.store(true, Ordering::Relaxed);
            }
            _ => {}
        }
    })?;

    Ok(())
}

struct WindowData {
    window: Window,
    framebuffer: Pixels,
}
