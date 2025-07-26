use std::time::{Duration, Instant};
use std::thread::sleep;

mod cpu;
use cpu::CPU;

fn main() {
    let mut cpu = CPU::new();

    cpu.load_rom("roms/pong.ch8").expect("Failed to load ROM");

    let sixty_hz_interval = Duration::from_millis(16);
    let mut last_timer_time = Instant::now();

    loop {
        if let Err(e) = cpu.tick() {
            eprintln!("Emulation error: {}", e);
            break;
        }

        if last_timer_time.elapsed() >= sixty_hz_interval {
            cpu.update_timers();
            last_timer_time = Instant::now();
        }

        sleep(Duration::from_millis(2));
    }
}
