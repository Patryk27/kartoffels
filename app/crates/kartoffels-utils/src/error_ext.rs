use anyhow::Error;
use itertools::Itertools;

pub trait ErrorExt {
    fn to_fmt_string(&self) -> String;
}

impl ErrorExt for Error {
    fn to_fmt_string(&self) -> String {
        self.chain()
            .map(|err| err.to_string())
            .join("\n\ncaused by:\n")
            .to_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    use indoc::indoc;

    #[test]
    fn to_fmt_string() {
        let actual = anyhow!("firmware is totally invalid")
            .context("Couldn't Parse Firmware")
            .context("couldn't upload bot")
            .to_fmt_string();

        let expected = indoc! {"
            couldn't upload bot

            caused by:
            couldn't parse firmware

            caused by:
            firmware is totally invalid
        "};

        assert_eq!(expected.trim_end(), actual);
    }
}
