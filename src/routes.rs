use axum::{extract::{ConnectInfo, Path, State, WebSocketUpgrade}, response::{sse::Event, IntoResponse, Sse}, Json};
use futures_util::Stream;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use core::net::SocketAddr;
use std::{convert::Infallible, time::Duration};

use crate::{types::{Channels, MessageType, Notification, NotificationMessage}, websocket::handle_socket};

/// Handle Websocket connection
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Channels>,
    Path(user_id): Path<String>
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, addr, state, user_id))
}

/// Forward message in request body to broadcast channel
pub async fn send_notification_handler(
    State(state): State<Channels>,
    Path(user_id): Path<String>,
    Json(payload): Json<NotificationMessage>
) -> impl IntoResponse {
    let notification = Notification {
        user_id,
        message: MessageType::Data(payload.message)
    };
    let _ = state.tx.send(notification);
    Json(serde_json::json!({"status": 200}))
}

/// Handle Server Sent Event connection
pub async fn sse_handler(
    State(state): State<Channels>,
    Path(user_id): Path<String>
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {

    // Get broadcast receiver
    let broadcast_rx = state.tx.subscribe();
    // clone user id to use in closures below
    let user_id_filter = user_id.clone();

    // convert broadcast receiver into stream
    let stream = BroadcastStream::new(broadcast_rx)
    // filter notifications for current user
    .filter(move |f| {
        let value = match f {
            Ok(notification) => notification,
            Err(e) => &Notification { user_id: user_id_filter.clone(), message: MessageType::Error(e.to_string()) },
        };
        return user_id_filter == value.user_id;
    })
    // Format notifications into Server Sent Events
    .map(move |f| { 
        let user_id = user_id.to_string();
        let value = match f {
            Ok(notification) => notification,
            Err(e) => Notification { user_id, message: MessageType::Error(e.to_string()) },
        };
        let data = serde_json::to_string(&value).unwrap_or_default();
        Ok(Event::default().data(data))
    });
    
    // Send response to client
    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}