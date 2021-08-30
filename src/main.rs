//! Example chat application.
//!
//! Run with
//!
//! ```not_rust
//! cargo run -p example-chat
//! ```

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Extension, RequestParts, TypedHeader};
use axum::handler::get;
use axum::http::header::HeaderMap;
use axum::response::{Html, IntoResponse};
use axum::{AddExtensionLayer, Error};
use axum::Router;
use futures::{sink::SinkExt, stream::StreamExt, Sink};
use std::collections::{HashSet, HashMap};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{broadcast, oneshot, Mutex};
use headers::HeaderMapExt;
use futures::stream::SplitSink;
use tokio::sync::mpsc;
use axum::http::{StatusCode, Response, HeaderValue};
use prost;
use onebot::*;
use std::convert::Infallible;
use prost::DecodeError;
use rs_pbbot_demo::onebot::frame::{Data, FrameType};
use rs_pbbot_demo::onebot::*;
use rs_pbbot_demo::onebot;
use rs_pbbot_demo::bot::Bot;
use tokio::sync::mpsc::Sender;
use uuid::Uuid;
use std::ops::Deref;

// Our shared state
struct AppState {
    bots: Mutex<HashMap<i64, Mutex<Bot>>>,
}

#[tokio::main]
async fn main() {
    let bots = Mutex::new(HashMap::new());

    let app_state = Arc::new(AppState { bots });

    let app = Router::new()
        .route("/ws/cq/", get(websocket_handler))
        .layer(AddExtensionLayer::new(app_state));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8081));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}


async fn websocket_handler(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    Extension(state): Extension<Arc<AppState>>,
) -> impl IntoResponse {
    let bot_id = headers.get("x-self-id").map(|id| id.to_str().unwrap().parse().unwrap_or_default()).unwrap_or_default();
    ws.on_upgrade(move |socket| websocket(socket, state, bot_id))
}

async fn websocket(stream: WebSocket, state: Arc<AppState>, bot_id: i64) {
    if bot_id == 0 {
        stream.close();
        return;
    }
    println!("bot connected: {}", bot_id);
    let (mut ws_out, mut ws_in) = stream.split();
    let (api_sender, mut api_receiver) = mpsc::channel(10); // api channel
    let resp_promises = Arc::new(Mutex::new(HashMap::new()));
    let mut bot = Bot { bot_id, api_sender: mpsc::Sender::clone(&api_sender), resp_promises: resp_promises.clone() };

    // 发送 api req
    let mut send_task = tokio::spawn(async move {
        while let Some(frame) = api_receiver.recv().await {
            let mut buf = Vec::new();
            prost::Message::encode(&frame, &mut buf);
            if ws_out.send(Message::Binary(buf)).await.is_err() {
                break;
            }
        }
    });

    // 接受 event 和 api resp
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_in.next().await {
            match msg {
                Message::Binary(buf) => {
                    let frame: onebot::Frame = match prost::Message::decode(buf.as_ref()) {
                        Ok(frame) => { frame }
                        Err(_) => break
                    };
                    if let Some(data) = frame.clone().data {
                        let mut bot = bot.clone();
                        tokio::spawn(async move {
                            match data {
                                Data::PrivateMessageEvent(event) => {
                                    let resp = bot.send_private_message(event.user_id, event.raw_message).await;
                                    if let Some(resp) = resp {
                                        println!("message_id: {}", resp.message_id);
                                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                                        bot.delete_msg(resp.message_id).await;
                                    }
                                    if let Some(get_group_list_resp) = bot.get_group_list().await {
                                        for group in get_group_list_resp.group {
                                            println!("{} {}", group.group_id, group.group_name)
                                        }
                                    }
                                }
                                Data::GroupMessageEvent(event) => {}
                                Data::GroupUploadNoticeEvent(event) => {}
                                Data::GroupAdminNoticeEvent(event) => {}
                                Data::GroupDecreaseNoticeEvent(event) => {}
                                Data::GroupIncreaseNoticeEvent(event) => {}
                                Data::GroupBanNoticeEvent(event) => {}
                                Data::FriendAddNoticeEvent(event) => {}
                                Data::GroupRecallNoticeEvent(event) => {}
                                Data::FriendRecallNoticeEvent(event) => {}
                                Data::FriendRequestEvent(event) => {}
                                Data::GroupRequestEvent(event) => {}
                                _ => {
                                    // 不是 event，一定是 api resp
                                    if let Some(api_resp_sender) = bot.resp_promises.lock().await.remove(frame.echo.clone().as_str()) {
                                        api_resp_sender.send(frame);
                                    }
                                }
                            }
                        });
                    }
                }
                Message::Close(_) => { break; }
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }
    ;
    println!("bot disconnected: {}", bot_id);
}



