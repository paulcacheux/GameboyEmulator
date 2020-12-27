use std::{rc::Rc, sync::Mutex};

use super::ppu::Mode;

pub type InterruptControllerPtr = Rc<Mutex<InterruptController>>;

#[derive(Debug)]
pub struct InterruptController {
    last_ppu_mode: Mode,
    pub should_redraw: bool,
}

impl InterruptController {
    pub fn new() -> Self {
        InterruptController {
            last_ppu_mode: Mode::VBlank,
            should_redraw: false,
        }
    }

    pub fn ppu_mode_update(&mut self, mode: Mode) {
        if self.last_ppu_mode != mode {
            self.last_ppu_mode = mode;
            if mode == Mode::VBlank {
                self.should_redraw = true;
            }
        }
    }
}
