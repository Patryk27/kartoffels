use tokio::sync::broadcast;

pub trait BroadcastReceiverRx<T> {
    fn recv_pending(&mut self) -> Vec<T>;
}

impl<T> BroadcastReceiverRx<T> for broadcast::Receiver<T>
where
    T: Clone,
{
    fn recv_pending(&mut self) -> Vec<T> {
        let mut items = Vec::new();

        while let Ok(item) = self.try_recv() {
            items.push(item);
        }

        items
    }
}
