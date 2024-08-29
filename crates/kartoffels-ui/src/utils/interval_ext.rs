use tokio::time::Interval;

pub trait IntervalExt
where
    Self: Sized,
{
    fn skipping_first(self) -> Self;
}

impl IntervalExt for Interval {
    fn skipping_first(mut self) -> Self {
        self.reset();
        self
    }
}
