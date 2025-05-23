use crate::{Ui, UiWidget};
use kartoffels_store::Store;
use ratatui::style::Color;
use std::task::Poll;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct Fade<T> {
    stage: Option<FadeStage<T>>,
    testing: bool,
}

impl<T> Fade<T> {
    pub fn new(store: &Store, fade_in: bool) -> Self {
        Self {
            stage: fade_in.then(|| FadeStage::FadingIn {
                fade: FadeBackdrop::new(FadeDir::In),
            }),
            testing: store.testing(),
        }
    }

    pub fn out(&mut self, event: T) {
        if let None | Some(FadeStage::FadingIn { .. }) = &self.stage {
            self.stage = Some(FadeStage::FadingOut {
                fade: FadeBackdrop::new(FadeDir::Out),
                event: Some(event),
                ready: false,
            });
        }
    }

    pub fn poll(&mut self) -> Option<T> {
        if let Some(FadeStage::FadingOut {
            event, ready: true, ..
        }) = &mut self.stage
        {
            event.take()
        } else {
            None
        }
    }

    pub fn render<U>(&mut self, ui: &mut Ui<U>) {
        match &mut self.stage {
            None => {
                //
            }

            Some(FadeStage::FadingIn { fade }) => {
                if ui.add(&*fade).is_ready() {
                    self.stage = None;
                }
            }

            Some(FadeStage::FadingOut { fade, ready, .. }) => {
                if ui.add(&*fade).is_ready() || self.testing {
                    *ready = true;
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
struct FadeBackdrop {
    dir: FadeDir,
    started_at: Instant,
}

impl FadeBackdrop {
    const DURATION: Duration = Duration::from_millis(250);

    fn new(dir: FadeDir) -> Self {
        Self {
            dir,
            started_at: Instant::now(),
        }
    }
}

impl<T> UiWidget<T> for &FadeBackdrop {
    type Response = Poll<()>;

    fn render(self, ui: &mut Ui<T>) -> Self::Response {
        let alpha = {
            let t = self
                .started_at
                .elapsed()
                .div_duration_f32(FadeBackdrop::DURATION)
                .clamp(0.0, 1.0);

            match self.dir {
                FadeDir::In => t,
                FadeDir::Out => 1.0 - t,
            }
            .powi(2)
        };

        for y in 0..ui.area.height {
            for x in 0..ui.area.width {
                let cell = &mut ui.buf[(x, y)];

                for color in [&mut cell.fg, &mut cell.bg] {
                    if let Color::Rgb(r, g, b) = color {
                        *r = ((*r as f32) * alpha) as u8;
                        *g = ((*g as f32) * alpha) as u8;
                        *b = ((*b as f32) * alpha) as u8;
                    } else {
                        // Should be unreachable, since we rely on RGB colors
                        // everywhere, but let's avoid panicking just in case
                    }
                }
            }
        }

        if self.started_at.elapsed() >= FadeBackdrop::DURATION {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FadeDir {
    In,
    Out,
}

#[derive(Clone, Debug)]
enum FadeStage<T> {
    FadingIn {
        fade: FadeBackdrop,
    },
    FadingOut {
        fade: FadeBackdrop,
        event: Option<T>,
        ready: bool,
    },
}
