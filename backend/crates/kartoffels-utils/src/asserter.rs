use pretty_assertions::private::CreateComparison;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct Asserter {
    dir: PathBuf,
    failed: bool,
}

impl Asserter {
    pub fn new(dir: impl AsRef<Path>) -> Self {
        let dir = dir.as_ref();

        assert!(dir.exists(), "directory not found: {dir:?}");

        Self {
            dir: dir.to_owned(),
            failed: false,
        }
    }

    pub fn assert(
        &mut self,
        fixture: impl AsRef<str>,
        actual: impl AsRef<str>,
    ) -> &mut Self {
        let fixture = fixture.as_ref();
        let actual = actual.as_ref();

        let expected_path = self.dir.join(fixture);
        let expected_new_path = self.dir.join(format!("{fixture}.new"));
        let expected = fs::read_to_string(&expected_path).unwrap_or_default();

        if expected == actual {
            _ = fs::remove_file(&expected_new_path);
        } else {
            _ = fs::write(&expected_new_path, actual);

            eprintln!(
                "found differences between `{}` and `{}`:\n\n{}",
                expected_path.display(),
                expected_new_path.display(),
                (expected, actual).create_comparison(),
            );

            self.failed = true;
        }

        self
    }

    pub fn finish(&mut self) {
        if self.failed {
            panic!("some assertions failed");
        }
    }
}
