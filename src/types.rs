use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::Sender;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum MessageType<T> {
    #[default]
    Ping,
    Pong,
    Data(T),
    Error(T)
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct Notification {
    pub user_id: String,
    pub message: MessageType<String>
}

#[derive(Debug, Clone)]
pub(crate) struct Channels {
    pub tx: Sender<Notification>
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct NotificationMessage {
    pub message: String
}
