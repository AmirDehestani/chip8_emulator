use crate::constants::{DISPLAY_WIDTH, DISPLAY_HEIGHT};

pub struct Display {
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    scale: u32,
}

impl Display {
    pub fn new(sdl_ctx: &sdl2::Sdl, scale: u32) -> Result<Self, String> {
        let video = sdl_ctx.video()?;
        let window = video
            .window(
                "CHIP-8",
                DISPLAY_WIDTH as u32 * scale,
                DISPLAY_HEIGHT as u32 * scale
            )
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;

        let canvas = window.into_canvas()
            .present_vsync()
            .build()
            .map_err(|e| e.to_string())?;

        Ok(Self { canvas, scale })
    }

    pub fn render(&mut self, buffer: &[u8]) {
        self.canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 26, 0));
        self.canvas.clear();

        self.canvas.set_draw_color(sdl2::pixels::Color::RGB(57, 255, 20));
        for y in 0..DISPLAY_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                if buffer[y * DISPLAY_WIDTH + x] != 0 {
                    let rect = sdl2::rect::Rect::new(
                        (x as u32 * self.scale) as i32,
                        (y as u32 * self.scale) as i32,
                        self.scale,
                        self.scale,
                    );
                    self.canvas.fill_rect(rect).ok();
                }
            }
        }

        self.canvas.present();
    }
}
