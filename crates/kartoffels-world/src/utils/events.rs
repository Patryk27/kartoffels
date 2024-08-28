use super::Container;
use std::collections::VecDeque;

#[derive(Default)]
pub struct Events {
    events: Container,
}

impl Events {
    pub fn send<T>(&mut self, event: T)
    where
        T: Send + 'static,
    {
        self.events.get_mut::<VecDeque<_>>().push_back(event);
    }

    pub fn recv<T>(&mut self) -> Option<T>
    where
        T: Send + 'static,
    {
        self.events.get_mut::<VecDeque<_>>().pop_front()
    }
}
