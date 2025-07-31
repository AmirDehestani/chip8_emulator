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

    println!("Select a rom from the list below:");
    let roms_dir = "./roms";
    let roms = std::fs::read_dir(roms_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .map(|entry| entry.file_name().into_string().unwrap_or_default())
        .collect::<Vec<String>>();

    if roms.is_empty() {
        println!("No ROMs found");
        return Ok(());
    }

    for (i, rom) in roms.iter().enumerate() {
        println!("{}: {}", i + 1, rom);
    }

    let mut selected_rom = String::new();
    std::io::stdin().read_line(&mut selected_rom)?;
    let selected_rom = selected_rom.trim().parse::<usize>().ok();

    let rom_path = match selected_rom.and_then(|index| roms.get(index - 1)) {
        Some(rom) => format!("{}/{}", roms_dir, rom),
        None => {
            println!("Invalid selection.");
            return Ok(());
        }
    };

    let mut cpu = CPU::new();
    let _ = cpu.load_rom(&rom_path);

    let sdl_ctx = sdl2::init()?;
    let mut display = Display::new(&sdl_ctx, SCALE)?;
    let mut input = Input::new();
    let mut event_pump = sdl_ctx.event_pump()?;

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
