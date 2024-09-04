use crate::DrivenGame;
use anyhow::Result;
use kartoffels_ui::{theme, Button, Ui};
use ratatui::style::{Style, Styled, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Paragraph, WidgetRef};
use std::cmp;
use std::sync::LazyLock;
use termwiz::input::KeyCode;

#[rustfmt::skip]
pub static DIALOG_01: LazyLock<Dialog> = LazyLock::new(|| Dialog {
    text: vec![
        CLine::new("hey there and welcome to kartoffels 🫡"),
        CLine::new(""),
        CLine::from_iter([
            Span::raw(
                "in a couple minutes you're going to be, like, the bots' boss, \
                 so let's get down to business!"
            ),
            Span::raw("*").fg(theme::RED),
        ]),
        CLine::new(""),
        CLine::new("lesson #1:").bold(),
        CLine::new("you can navigate the interface using keyboard and/or mouse"),
        CLine::new("(that includes when you're connected through the terminal)"),
        CLine::new(""),
        CLine::new("lesson #2:").bold(),
        CLine::new("pressing Ctrl-c will always bring you to the main menu"),
        CLine::new(""),
        CLine::from_iter([
            Span::raw("* ").fg(theme::RED),
            Span::raw(
                "kartoffels ltd is not responsible for loss of hearing, loss \
                 of sight, sudden feeling of the flight and fight syndrome, \
                 wanting to do origami but being unable to etc."
            ),
        ]).fg(theme::DARK_GRAY).right_aligned(),
    ],

    actions: vec![(
        Button::new(KeyCode::Enter, "got it").right_aligned(),
        "ok",
    )],
});

#[rustfmt::skip]
pub static DIALOG_02: LazyLock<Dialog> = LazyLock::new(|| Dialog {
    text: vec![
        CLine::new("wow, you're learning so fast"),
        CLine::new(""),
        CLine::from_iter([
            Span::raw("NEXT LESSON!").bold(),
            Span::raw(" -- run this:"),
        ]),
        CLine::new(""),
        CLine::new("    git clone -b tutorial github.com/patryk27/kartoffel").fg(theme::WASHED_PINK),
        CLine::new(""),
        CLine::new("... and press enter once you're ready"),
    ],

    actions: vec![(
        Button::new(KeyCode::Enter, "i'm ready").right_aligned(),
        "ok",
    )],
});

#[rustfmt::skip]
pub static DIALOG_03: LazyLock<Dialog> = LazyLock::new(|| Dialog {
    text: vec![
        CLine::new(
            "perhaps i should mention that we'll be using rust, are you \
             comfortable with that?",
        ),
    ],

    actions: vec![(
        Button::new(KeyCode::Enter, "sure why not").right_aligned(),
        "ok",
    )],
});

#[rustfmt::skip]
pub static DIALOG_04: LazyLock<Dialog> = LazyLock::new(|| Dialog {
    text: vec![
        CLine::new("faboulous!").fg(theme::PINK).bold(),
        CLine::new(""),
        CLine::new(
            "launch vscode, vim, emacs or whatever makes your life colorful \
             and open `main.rs` from the cloned repository"
        ),
        CLine::new(""),
        CLine::new(
            "for, you see, writing a bot is similar to writing a regular rust \
             program - but it's also different, mucho different",
        ),
        CLine::new(""),
        CLine::new(
            "say, you don't have access to the standard library - there's no \
             `println!()`, no `std::fs` etc; everything your robot has access \
             to is a bit of memory, motor, radar and serial port, like the \
             people in ancient rome did",
        ),
    ],

    actions: vec![(
        Button::new(KeyCode::Enter, "got it").right_aligned(),
        "ok",
    )],
});

#[rustfmt::skip]
pub static DIALOG_05: LazyLock<Dialog> = LazyLock::new(|| Dialog {
    text: vec![
        CLine::new(
            "anyway, as you can see in the code, our robot currently doesn't \
             do much - it just calls `motor_step()` over and over",
        ),
        CLine::new(""),
        CLine::new(
            "this function is responsible for moving the robot one tile \
             forward in the direction it is currently facing",
        ),
        CLine::new(""),
        CLine::from_iter([
            Span::raw("boooring").bold(),
            Span::raw(" - let's see the robot in action !!"),
        ]),
        CLine::new(""),
        CLine::new("if you're on windows, run this:"),
        CLine::web("    ./build.bat").fg(theme::WASHED_PINK),
        CLine::ssh("    ./build.bat --copy").fg(theme::WASHED_PINK),
        CLine::new(""),
        CLine::new("otherwise, run this:"),
        CLine::web("    ./build").fg(theme::WASHED_PINK),
        CLine::ssh("    ./build --copy").fg(theme::WASHED_PINK),
        CLine::new(""),
        CLine::new(
            "... and having done so, press enter to close this window and then \
             press `u` to upload the bot",
        ),
        CLine::web(""),
        CLine::web(
            "when the file picker opens, choose a file called `kartoffel` - it \
             should be located next to `README.md` etc.",
        ),
    ],

    actions: vec![(
        Button::new(KeyCode::Enter, "i have done so").right_aligned(),
        "ok",
    )],
});

#[rustfmt::skip]
pub static DIALOG_06: LazyLock<Dialog> = LazyLock::new(|| Dialog {
    text: vec![
        CLine::new("nice!"),
        CLine::new(""),
        CLine::new(
            "you, [subject name here] must be the pride of [subject hometown \
             here].",
        )
    ],

    actions: vec![(
        Button::new(KeyCode::Enter, "i am").right_aligned(),
        "ok",
    )],
});

#[rustfmt::skip]
pub static DIALOG_07: LazyLock<Dialog> = LazyLock::new(|| Dialog {
    text: vec![
        CLine::new(
            "the game has been automatically paused to show you the humoristic \
             element a moment ago",
        ),
        CLine::new(""),
        CLine::new(
            "now, close this dialogue, press space to unpause the game and \
             let's see the bot in action",
        ),
        CLine::new(""),
        CLine::new(
            "if everything goes correctly, we should see the robot driving \
             forward and falling out the map",
        ),
    ],

    actions: vec![(
        Button::new(KeyCode::Enter, "let's see the robot driving").right_aligned(),
        "ok",
    )],
});

#[rustfmt::skip]
pub static DIALOG_08: LazyLock<Dialog> = LazyLock::new(|| Dialog {
    text: vec![
        CLine::new("nice!").fg(theme::GREEN),
        CLine::new(""),
        CLine::new(
            "i mean, not nice, because we're dead - but it's nice relatively \
             speaking",
        ),
        CLine::new(""),
        CLine::new(
            "after a bot dies, the game automatically restarts it - close this \
             dialog, unpause the game and let's see that",
        ),
    ],

    actions: vec![(
        Button::new(KeyCode::Enter, "let's see the robot dying again").right_aligned(),
        "ok",
    )],
});

#[derive(Clone, Debug)]
pub struct Dialog {
    text: Vec<CLine>,
    actions: Vec<(Button<'static>, &'static str)>,
}

impl Dialog {
    pub async fn show(
        &'static self,
        game: &DrivenGame,
    ) -> Result<&'static str> {
        game.dialog(move |ui, resp| {
            let text = {
                let text: Text = self
                    .text
                    .iter()
                    .filter(|line| line.matches(ui))
                    .map(|line| line.inner.clone())
                    .collect();

                Paragraph::new(text).wrap(Default::default())
            };

            let width = cmp::min(60, ui.area().width - 4);
            let height = text.line_count(width) as u16 + 2;

            ui.info_dialog(width, height, Some(" tutorial "), |ui| {
                text.render_ref(ui.area(), ui.buf());
                ui.space(height - 1);

                for (button, button_resp) in &self.actions {
                    if button.render(ui).pressed {
                        if let Some(resp) = resp.take() {
                            _ = resp.send(*button_resp);
                        }
                    }
                }
            });
        })
        .await
    }
}

#[derive(Clone, Debug)]
struct CLine {
    inner: Line<'static>,
    cond: Option<CLineCondition>,
}

impl CLine {
    fn new(content: &'static str) -> Self {
        Self {
            inner: Line::raw(content),
            cond: None,
        }
    }

    fn ssh(content: &'static str) -> Self {
        Self {
            inner: Line::raw(content),
            cond: Some(CLineCondition::ShowOnlyOnSsh),
        }
    }

    fn web(content: &'static str) -> Self {
        Self {
            inner: Line::raw(content),
            cond: Some(CLineCondition::ShowOnlyOnWeb),
        }
    }

    fn right_aligned(mut self) -> Self {
        self.inner = self.inner.right_aligned();
        self
    }

    fn matches(&self, ui: &Ui) -> bool {
        match self.cond {
            Some(CLineCondition::ShowOnlyOnSsh) => ui.ty().is_ssh(),
            Some(CLineCondition::ShowOnlyOnWeb) => ui.ty().is_web(),
            None => true,
        }
    }
}

impl Styled for CLine {
    type Item = Self;

    fn style(&self) -> Style {
        Styled::style(&self.inner)
    }

    fn set_style<S>(self, style: S) -> Self::Item
    where
        S: Into<Style>,
    {
        Self {
            inner: self.inner.set_style(style),
            cond: self.cond,
        }
    }
}

impl FromIterator<Span<'static>> for CLine {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Span<'static>>,
    {
        Self {
            inner: iter.into_iter().collect(),
            cond: None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum CLineCondition {
    ShowOnlyOnSsh,
    ShowOnlyOnWeb,
}
