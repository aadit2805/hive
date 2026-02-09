use tokio::sync::mpsc;
use super::types::HiveEvent;

/// Event queue buffer size
const QUEUE_SIZE: usize = 1000;

/// Creates a new event queue channel pair
pub fn create_event_queue() -> (EventSender, EventReceiver) {
    let (tx, rx) = mpsc::channel(QUEUE_SIZE);
    (EventSender(tx), EventReceiver(rx))
}

/// Sender side of the event queue
#[derive(Clone)]
pub struct EventSender(pub mpsc::Sender<HiveEvent>);

impl EventSender {
    pub async fn send(&self, event: HiveEvent) -> Result<(), mpsc::error::SendError<HiveEvent>> {
        self.0.send(event).await
    }

    pub fn blocking_send(&self, event: HiveEvent) -> Result<(), mpsc::error::SendError<HiveEvent>> {
        self.0.blocking_send(event)
    }

    pub fn inner(&self) -> mpsc::Sender<HiveEvent> {
        self.0.clone()
    }
}

/// Receiver side of the event queue
pub struct EventReceiver(pub mpsc::Receiver<HiveEvent>);

impl EventReceiver {
    pub async fn recv(&mut self) -> Option<HiveEvent> {
        self.0.recv().await
    }

    pub fn try_recv(&mut self) -> Result<HiveEvent, mpsc::error::TryRecvError> {
        self.0.try_recv()
    }
}
