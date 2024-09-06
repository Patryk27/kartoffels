use crate::theme;
use ratatui::style::Stylize;
use ratatui::text::Line;
use std::mem;

pub trait FromMarkdown {
    fn md(s: &str) -> Self;
}

impl FromMarkdown for Line<'static> {
    fn md(s: &str) -> Self {
        if s.starts_with("# ") {
            return Line::raw(s.to_owned()).bold();
        }

        if s.starts_with("    ") {
            return Line::raw(s.to_owned()).fg(theme::WASHED_PINK);
        }

        let mut chars = s.chars();
        let mut spans = Vec::new();
        let mut pending_span = String::new();

        while let Some(ch) = chars.next() {
            if ['*', '_', '`'].contains(&ch) {
                if !pending_span.is_empty() {
                    spans.push(mem::take(&mut pending_span).into());
                }

                let tag = ch;
                let mut span = String::new();

                loop {
                    let ch = chars.next().expect("unterminated tag: *");

                    if ch == tag {
                        break;
                    } else {
                        span.push(ch);
                    }
                }

                spans.push(match tag {
                    '*' => span.bold(),
                    '_' => span.italic(),
                    '`' => span.fg(theme::WASHED_PINK),
                    _ => unreachable!(),
                });
            } else {
                pending_span.push(ch);
            }
        }

        if !pending_span.is_empty() {
            spans.push(pending_span.into());
        }

        spans.into_iter().collect()
    }
}
