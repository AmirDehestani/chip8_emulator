use std::time::{Duration, Instant};
use std::thread::sleep;

mod cpu;
mod platform;
mod constants;
use cpu::CPU;
use platform::Display;
use platform::Input;

const SCALE: u32 = 10;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sdl_ctx = sdl2::init()?;
    let mut display = Display::new(&sdl_ctx, SCALE)?;
    let mut input = Input::new();
    let mut event_pump = sdl_ctx.event_pump()?;

    let mut cpu = CPU::new();
    cpu.load_rom("roms/test_opcode.ch8").expect("Failed to load ROM");

    let sixty_hz_interval = Duration::from_millis(16);
    let mut last_timer_time = Instant::now();

    loop {
        for event in event_pump.poll_iter() {
            if let sdl2::event::Event::Quit { .. } = event {
                return Ok(());
            }
            input.update(&event);
        }

        cpu.input = input.keys;

        if let Err(e) = cpu.tick() {
            eprintln!("Emulation error: {}", e);
            break Ok(());
        }

        if last_timer_time.elapsed() >= sixty_hz_interval {
            cpu.update_timers();
            last_timer_time = Instant::now();
        }

        display.render(&cpu.display);
        // sleep(Duration::from_millis(2));
    }
}
