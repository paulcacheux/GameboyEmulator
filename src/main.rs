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
use memory::Memory;
use ppu::{PPU, SCREEN_HEIGHT, SCREEN_WIDTH};

const MULTIPLIER: u32 = 4;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * MULTIPLIER;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * MULTIPLIER;

const MACHINE_CYCLE_FREQ: u32 = 1 << 20;
const MACHINE_CYCLE_PER_FRAME: u32 = MACHINE_CYCLE_FREQ / 60;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let bootstrap_path = std::env::args()
        .nth(1)
        .expect("Failed to get bootstrap path");
    let rom_path = std::env::args().nth(2).expect("Failed to get rom path");

    let bootstrap_content = std::fs::read(&bootstrap_path)?;
    let rom_content = std::fs::read(&rom_path)?;

    let mut mmu = memory::MMU::new();
    mmu.write_slice(&rom_content, 0x0);
    mmu.write_slice(&bootstrap_content, 0x0);

    let memory = Rc::new(RwLock::new(mmu));

    let interrupt_controller = Rc::new(Mutex::new(InterruptController::new()));
    let mut cpu = CPU::new(memory.clone());
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
                if let Err(_) = framebuffer.render() {
                    println!("Failed to render framebuffer");
                    *control_flow = ControlFlow::Exit;
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

        for _ in 0..MACHINE_CYCLE_PER_FRAME {
            cpu.step();
            ppu.step();
            if cpu.pc == 0x00E0 {
                // ppu::dump_tiles_to_file(&memory, "tile_dump.ppm").expect("Failed to dump tiles");
                ppu::dump_frame_to_file(&ppu.previous_frame, "frame_dump.ppm")
                    .expect("Failed to dump frame");
                *control_flow = ControlFlow::Exit;
            }
        }

        let mut int_cont = interrupt_controller.lock().unwrap();
        if int_cont.should_redraw {
            window.request_redraw();
            int_cont.should_redraw = false;
        }
    });
}
