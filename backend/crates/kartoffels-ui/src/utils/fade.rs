use crate::{Ui, UiWidget};
use ratatui::style::Color;
use std::time::Instant;

#[derive(Debug)]
pub struct Fade {
    dir: FadeDir,
    started_at: Instant,
}

impl Fade {
    const DURATION_MS: f32 = 200.0;

    pub fn new(dir: FadeDir) -> Self {
        Self {
            dir,
            started_at: Instant::now(),
        }
    }

    pub fn dir(&self) -> FadeDir {
        self.dir
    }

    pub fn is_completed(&self) -> bool {
        self.started_at.elapsed().as_millis() as f32 >= Fade::DURATION_MS
    }
}

impl<T> UiWidget<T> for &Fade {
    type Response = FadeStatus;

    fn render(self, ui: &mut Ui<T>) -> Self::Response {
        let alpha = {
            let alpha = self.started_at.elapsed().as_millis() as f32
                / Fade::DURATION_MS;

            let alpha = alpha.clamp(0.0, 1.0);

            match self.dir {
                FadeDir::In => alpha,
                FadeDir::Out => 1.0 - alpha,
            }
        };

        for y in 0..ui.area.height {
            for x in 0..ui.area.width {
                let cell = &mut ui.buf[(x, y)];

                if let Color::Rgb(r, g, b) = &mut cell.fg {
                    *r = ((*r as f32) * alpha) as u8;
                    *g = ((*g as f32) * alpha) as u8;
                    *b = ((*b as f32) * alpha) as u8;
                } else {
                    // Should be unreachable, since we rely on RGB colors
                    // everywhere, but let's avoid panicking just in case
                }
            }
        }

        if self.is_completed() {
            FadeStatus::Completed
        } else {
            FadeStatus::Pending
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FadeDir {
    In,
    Out,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FadeStatus {
    Pending,
    Completed,
}

impl FadeStatus {
    pub fn is_completed(&self) -> bool {
        matches!(self, FadeStatus::Completed)
    }
}
