use crate::{Ui, UiWidget};
use ratatui::style::Color;
use std::task::Poll;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct Fade {
    dir: FadeDir,
    started_at: Instant,
}

impl Fade {
    const DURATION: Duration = Duration::from_millis(250);

    pub fn new(dir: FadeDir) -> Self {
        Self {
            dir,
            started_at: Instant::now(),
        }
    }

    pub fn dir(&self) -> FadeDir {
        self.dir
    }

    pub fn is_ready(&self) -> bool {
        self.started_at.elapsed() >= Fade::DURATION
    }
}

impl<T> UiWidget<T> for &Fade {
    type Response = Poll<()>;

    fn render(self, ui: &mut Ui<T>) -> Self::Response {
        let alpha = {
            let t = self
                .started_at
                .elapsed()
                .div_duration_f32(Fade::DURATION)
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

        if self.is_ready() {
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
pub struct FadeCtrl<T> {
    stage: Option<FadeCtrlStage<T>>,
    animate: bool,
}

impl<T> FadeCtrl<T>
where
    T: FadeCtrlEvent,
{
    pub fn animate(mut self, animate: bool) -> Self {
        self.animate = animate;
        self
    }

    pub fn fade_in(mut self, fade_in: bool) -> Self {
        if fade_in {
            self.stage = Some(FadeCtrlStage::FadeIn {
                fade: Fade::new(FadeDir::In),
            });
        }

        self
    }

    pub fn render(&mut self, ui: &mut Ui<'_, T>, f: impl FnOnce(&mut Ui<T>)) {
        let event = match &mut self.stage {
            Some(FadeCtrlStage::FadeIn { fade }) => {
                let event = ui.catch(f);

                if ui.add(&*fade).is_ready() {
                    self.stage = None;
                }

                event
            }

            Some(FadeCtrlStage::FadeOut { fade, event }) => {
                _ = ui.catch(f);

                if ui.add(&*fade).is_ready()
                    && let Some(event) = event.take()
                {
                    ui.throw(event);
                }

                None
            }

            None => ui.catch(f),
        };

        if let Some(event) = event {
            if self.animate && event.needs_fade_out() {
                self.stage = Some(FadeCtrlStage::FadeOut {
                    fade: Fade::new(FadeDir::Out),
                    event: Some(event),
                });
            } else {
                ui.throw(event);
            }
        }
    }
}

impl<T> Default for FadeCtrl<T> {
    fn default() -> Self {
        Self {
            stage: Default::default(),
            animate: Default::default(),
        }
    }
}

pub trait FadeCtrlEvent {
    fn needs_fade_out(&self) -> bool;
}

#[derive(Clone, Debug)]
enum FadeCtrlStage<T> {
    FadeIn { fade: Fade },
    FadeOut { fade: Fade, event: Option<T> },
}
