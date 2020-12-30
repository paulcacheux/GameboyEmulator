use std::{
    rc::Rc,
    sync::{Mutex, RwLock},
};

use interrupt::InterruptController;
use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod cpu;
mod interrupt;
mod memory;
mod ppu;
mod utils;

use cpu::CPU;
use ppu::{PPU, SCREEN_HEIGHT, SCREEN_WIDTH};

const MULTIPLIER: u32 = 4;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * MULTIPLIER;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * MULTIPLIER;

const MACHINE_CYCLE_FREQ: u32 = 1 << 20;
const MACHINE_CYCLE_PER_FRAME: u32 = MACHINE_CYCLE_FREQ / 60;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let (bootstrap, rom) = match std::env::args().len() {
        2 => {
            let rom_path = std::env::args().nth(1).expect("Failed to get rom path");
            let rom_content = std::fs::read(&rom_path)?;
            (None, rom_content)
        }
        3 => {
            let bootstrap_path = std::env::args()
                .nth(1)
                .expect("Failed to get bootstrap path");
            let rom_path = std::env::args().nth(2).expect("Failed to get rom path");

            let bootstrap_content = std::fs::read(&bootstrap_path)?;
            let rom_content = std::fs::read(&rom_path)?;
            (Some(bootstrap_content), rom_content)
        }
        _ => panic!("Incorrect arguments"),
    };

    let interrupt_controller = Rc::new(Mutex::new(InterruptController::new()));

    let mbc = memory::build_mbc(&rom);
    let mut mmu = memory::MMU::new(mbc, interrupt_controller.clone());
    if let Some(bootstrap) = &bootstrap {
        mmu.write_bootstrap_rom(bootstrap);
    } else {
        mmu.unmount_bootstrap_rom();
    }

    let memory = Rc::new(RwLock::new(mmu));

    let mut cpu = CPU::new(memory.clone(), interrupt_controller.clone());
    if bootstrap.is_none() {
        cpu.pc = 0x100;
    }
    let mut ppu = PPU::new(memory.clone(), interrupt_controller.clone());

    let event_loop = EventLoop::new();

    let window = {
        let size = LogicalSize::new(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64);
        WindowBuilder::new()
            .with_title("GameBoy Emulator")
            .with_inner_size(size)
            .with_resizable(false)
            .build(&event_loop)
            .unwrap()
    };

    let mut framebuffer = {
        let window_physical_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(
            window_physical_size.width,
            window_physical_size.height,
            &window,
        );
        Pixels::new(SCREEN_WIDTH as _, SCREEN_HEIGHT as _, surface_texture)?
    };

    event_loop.run(move |event, _, control_flow| {
        use winit::event::{Event, WindowEvent};

        *control_flow = ControlFlow::Poll;

        match event {
            Event::RedrawRequested(_) => {
                ppu.draw_into_fb(framebuffer.get_frame());
                let _ = framebuffer.render();
            }
            Event::MainEventsCleared => {
                for _ in 0..MACHINE_CYCLE_PER_FRAME {
                    cpu.step();
                    ppu.step();
                }

                let mut int_cont = interrupt_controller.lock().unwrap();
                if int_cont.should_redraw {
                    window.request_redraw();
                    int_cont.should_redraw = false;
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}
