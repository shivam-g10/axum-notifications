use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use core::net::SocketAddr;

use crate::types::{Channels, MessageType, Notification};

pub async fn handle_socket(mut socket: WebSocket, who: SocketAddr, state: Channels, user_id: String) {
    // send a ping (unsupported by some browsers) just to kick things off and get a response
    if socket.send(Message::Ping(vec![1, 2, 3])).await.is_ok() {
        tracing::debug!("Pinged {who}...");
    } else {
        tracing::error!("Could not send ping {who}!");
        // no Error here since the only thing we can do is to close the connection.
        // If we can not send messages, there is no way to salvage the statemachine anyway.
        return;
    }

    // Receive the first message sent in the lines above to make sure socket is active
    if let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg { // Pong([1,2,3]) 
            // first message is close websocket message
            if let Message::Close(_) = msg {
                return;
            }
        } else {
            tracing::error!("client {who} abruptly disconnected");
            return;
        }
    }

    tracing::info!("Connected {who}...");

    let (mut ws_sender, mut ws_receiver) = socket.split();

    let broadcast_tx = state.tx.clone();

    // clone user id to move to recv task
    let user_id_rx = user_id.clone();

    // This task will receive messages from client and send them to broadcast subscribers.
    let mut recv_task = tokio::spawn(async move {

        while let Some(Ok(Message::Text(text))) = ws_receiver.next().await {

            // parse message coming from client
            let received_notif: Notification = serde_json::from_str(&text).unwrap_or_default();

            // We will allow only Ping from client since this is a notification system
            if received_notif.message == MessageType::Ping {
                // create a Pong notification
                let notification = Notification {
                    user_id: user_id_rx.clone(),
                    message: MessageType::Pong
                };
                // send notification to broadcast channel
                let _ = broadcast_tx.send(notification);
            }
        }
    });

    // Subscribe before sending joined message.
    let mut broadcast_rx = state.tx.subscribe();

    // This task will receive broadcast messages and send text message to our client.
    let mut send_task = tokio::spawn(async move {

        // wait for message from broadcast channel
        while let Ok(msg) = broadcast_rx.recv().await {

            // Send message only if the received notification belongs to the connected user
            if &user_id == &msg.user_id {

                if ws_sender.send(Message::Text(serde_json::to_string(&msg).unwrap_or_default())).await.is_err() {
                    // In any websocket error, break loop.
                    break;
                }
            }
        }
    });

     // If any one of the tasks exit, abort the other.
     tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    // returning from the handler closes the websocket connection
    tracing::info!("Websocket context {who} destroyed");
}
