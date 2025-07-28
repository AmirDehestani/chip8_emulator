use std::time::{Duration, Instant};
use std::thread::sleep;
use sdl2::event::Event;

mod cpu;
mod platform;
mod constants;
use cpu::CPU;
use platform::Display;
use platform::Input;

const SCALE: u32 = 20;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sdl_ctx = sdl2::init()?;
    let mut display = Display::new(&sdl_ctx, SCALE)?;
    let mut input = Input::new();
    let mut event_pump = sdl_ctx.event_pump()?;

    let mut cpu = CPU::new();
    cpu.load_rom("roms/pong.ch8").expect("Failed to load ROM");

    let sixty_hz_interval = Duration::from_millis(16);
    let mut last_timer_time = Instant::now();

    const INSTRUCTIONS_PER_FRAME: usize = 10;

    loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => return Ok(()),

                Event::KeyDown { keycode: Some(kc), repeat: false, .. } => {
                    if let Some(key) = Input::map_sdl_keycode(kc) {
                        input.set_key(key, true);
                    }
                }

                Event::KeyUp { keycode: Some(kc), .. } => {
                    if let Some(key) = Input::map_sdl_keycode(kc) {
                        input.set_key(key, false);
                    }
                }

                _ => {}
            }
        }
        
        cpu.input = input.keys;

        for _ in 0..INSTRUCTIONS_PER_FRAME {
            if let Err(e) = cpu.tick() {
                eprintln!("Emulation error: {}", e);
                return Ok(());
            }
        }

        if last_timer_time.elapsed() >= sixty_hz_interval {
            cpu.update_timers();
            last_timer_time = Instant::now();
        }

        display.render(&cpu.display);
        sleep(Duration::from_millis(2));
    }
}
