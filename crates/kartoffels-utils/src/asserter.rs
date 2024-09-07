use pretty_assertions as pa;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct Asserter {
    dir: PathBuf,
}

impl Asserter {
    pub fn new(dir: impl AsRef<Path>) -> Self {
        Self {
            dir: dir.as_ref().to_owned(),
        }
    }

    pub fn assert(
        &self,
        fixture: impl AsRef<str>,
        actual: impl AsRef<str>,
    ) -> &Self {
        let fixture = fixture.as_ref();
        let actual = actual.as_ref();

        let expected_path = self.dir.join(fixture);
        let expected_new_path = self.dir.join(format!("{fixture}.new"));
        let expected = fs::read_to_string(&expected_path).unwrap_or_default();

        if expected == actual {
            _ = fs::remove_file(&expected_new_path);
        } else {
            _ = fs::write(&expected_new_path, actual);
        }

        pa::assert_eq!(
            expected,
            actual,
            "Found differences between `{}` and `{}`",
            expected_path.display(),
            expected_new_path.display(),
        );

        self
    }
}
