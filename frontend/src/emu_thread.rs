use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Instant,
};

use gbemu::{memory::Memory, CPU, PPU};

const FREQUENCY: u64 = 1 << 20;
const NANOS_IN_SECOND: u64 = 1_000_000_000;
const NANOS_IN_CYCLE: u64 = NANOS_IN_SECOND / FREQUENCY;

pub fn run<M: Memory + Clone>(mut cpu: CPU<M>, mut ppu: PPU<M>, is_ended: Arc<AtomicBool>) {
    let mut last_instant = Instant::now();
    let mut nano_counter: u64 = 0;

    while !is_ended.load(Ordering::Relaxed) {
        let now = Instant::now();
        let elapsed = now - last_instant;
        assert_eq!(elapsed.as_secs(), 0);
        nano_counter += elapsed.subsec_nanos() as u64;
        last_instant = now;

        while nano_counter >= NANOS_IN_CYCLE {
            cpu.one_or_two_steps();
            ppu.step();

            nano_counter -= NANOS_IN_CYCLE;
        }
    }
}
