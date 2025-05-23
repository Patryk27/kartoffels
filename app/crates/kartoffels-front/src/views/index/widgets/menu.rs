use super::super::Event;
use crate::{theme, Ui};
use ratatui::layout::Size;
use ratatui::style::Style;
use ratatui::widgets::{Block, Padding};
use termwiz::input::KeyCode;

#[derive(Debug)]
pub struct Menu {
    has_public_worlds: bool,
    variant: Variant,
}

impl Menu {
    pub fn new<T>(ui: &Ui<T>, has_public_worlds: bool) -> (Self, Size) {
        let variant = if ui.area.height >= 40 {
            Variant::Tall
        } else if ui.area.height > 30 {
            Variant::Short
        } else {
            Variant::Tiny
        };

        let size = {
            let mut height = 8;

            // `[p] play`
            if has_public_worlds {
                height += 2;
            }

            // `[q] quit`
            if ui.ty.is_ssh() {
                height += if variant.is_tiny() { 1 } else { 2 };
            }

            // Extra spacers for buttons
            if variant.is_tall() {
                height += 3;
            }

            Size { width: 50, height }
        };

        let this = Self {
            has_public_worlds,
            variant,
        };

        (this, size)
    }

    pub fn render(self, ui: &mut Ui<Event>) {
        let block = Block::bordered()
            .border_style(Style::new().fg(theme::GREEN).bg(theme::BG))
            .padding(Padding::horizontal(1));

        ui.block(block, |ui| {
            if self.has_public_worlds {
                ui.btn("play", KeyCode::Char('p'), |btn| {
                    btn.help("fight bots uploaded by other players")
                        .throwing(Event::Play)
                });

                if self.variant.is_tall() {
                    ui.space(1);
                }
            }

            ui.btn("sandbox", KeyCode::Char('s'), |btn| {
                btn.help("experiment on a private world")
                    .throwing(Event::Sandbox)
            });

            if self.variant.is_tall() {
                ui.space(1);
            }

            ui.btn("tutorial", KeyCode::Char('t'), |btn| {
                btn.help("learn how to play the game, quick & cheap")
                    .throwing(Event::Tutorial)
            });

            if self.variant.is_tall() {
                ui.space(1);
            }

            ui.btn("challenges", KeyCode::Char('c'), |btn| {
                btn.help("solve single-player exercises")
                    .throwing(Event::Challenges)
            });

            if ui.ty.is_ssh() {
                if self.variant.is_short() || self.variant.is_tall() {
                    ui.space(1);
                }

                ui.btn("quit", KeyCode::Escape, |btn| {
                    btn.throwing(Event::Quit)
                });
            }
        });
    }
}

#[derive(Clone, Copy, Debug)]
enum Variant {
    Tiny,
    Short,
    Tall,
}

impl Variant {
    fn is_tiny(&self) -> bool {
        matches!(self, Self::Tiny)
    }

    fn is_short(&self) -> bool {
        matches!(self, Self::Short)
    }

    fn is_tall(&self) -> bool {
        matches!(self, Self::Tall)
    }
}
