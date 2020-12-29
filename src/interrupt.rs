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

    pub fn request_redraw(&mut self) {
        self.should_redraw = true;
    }
}
