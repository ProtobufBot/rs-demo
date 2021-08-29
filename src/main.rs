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
use futures::{sink::SinkExt, stream::StreamExt};
use std::collections::{HashSet, HashMap};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
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

    let (mut ws_out, mut ws_in) = stream.split();
    while let Some(msg) = ws_in.next().await {
        match msg {
            Ok(msg) => {
                // println!("{:?}", msg);
                match msg {
                    Message::Text(_) => {}
                    Message::Binary(buf) => {
                        let frame: onebot::Frame = prost::Message::decode(&*buf).unwrap();
                        match frame.data {
                            None => { continue; }
                            Some(data) => {
                                match data {
                                    Data::PrivateMessageEvent(event) => {
                                        let out_frame = Frame {
                                            bot_id,
                                            frame_type: FrameType::TSendPrivateMsgReq.into(),
                                            echo: "123".to_string(),
                                            ok: true,
                                            extra: Default::default(),
                                            data: Some(Data::SendPrivateMsgReq(SendPrivateMsgReq {
                                                user_id: event.user_id,
                                                message: event.message,
                                                auto_escape: false,
                                            })),
                                        };
                                        let mut buf = Vec::new();
                                        prost::Message::encode(&out_frame, &mut buf);
                                        println!("{:?}", &buf);
                                        if ws_out.send(Message::Binary(buf)).await.is_err() {
                                            break;
                                        }
                                        println!("private message from {}: {}", event.user_id, event.raw_message);
                                    }
                                    Data::GroupMessageEvent(event) => {
                                        println!("group message from {}: {}", event.group_id, event.raw_message)
                                    }
                                    Data::GroupUploadNoticeEvent(event) => {
                                        println!("group file from {}: {}", event.group_id, event.file.unwrap().name)
                                    }
                                    Data::GroupAdminNoticeEvent(_) => {}
                                    Data::GroupDecreaseNoticeEvent(_) => {}
                                    Data::GroupIncreaseNoticeEvent(_) => {}
                                    Data::GroupBanNoticeEvent(_) => {}
                                    Data::FriendAddNoticeEvent(_) => {}
                                    Data::GroupRecallNoticeEvent(_) => {}
                                    Data::FriendRecallNoticeEvent(_) => {}
                                    Data::FriendRequestEvent(_) => {}
                                    Data::GroupRequestEvent(_) => {}
                                    _ => {}
                                }
                            }
                        }
                    }
                    Message::Ping(_) => {}
                    Message::Close(_) => { break; }
                    _ => {}
                }
            }
            Err(err) => { break; }
        }
    }
    state.bots.lock().unwrap().remove(&bot_id); // 掉线就会break while


    // let (sender, mut receiver) = mpsc::channel(100);
    //
    // let bot = Bot { bot_id, sender: Mutex::new(sender) };


    // let mut send_task = tokio::spawn(async move {
    //     while let Some(frame) = receiver.recv().await {
    //         //     if ws_out.send(msg).await.is_err() {
    //         //         break;
    //         //     }
    //     }
    // });
    //
    // // Clone things we want to pass to the receiving task.
    //
    // // This task will receive messages from client and send them to broadcast subscribers.
    // let mut recv_task = tokio::spawn(async move {
    //     while let Some(Ok(Message::Binary(buf))) = ws_in.next().await {
    //         let frame: onebot::Frame = match prost::Message::decode(&*buf) {
    //             Ok(frame) => { frame }
    //             Err(_) => { continue; }
    //         };
    //         let a = frame;
    //     }
    // });

    // // If any one of the tasks exit, abort the other.
    // tokio::select! {
    //     _ = (&mut send_task) => recv_task.abort(),
    //     _ = (&mut recv_task) => send_task.abort(),
    // };


    // Send user left message.

    // Remove username from map so new clients can take it.
}


struct Bot {
    bot_id: i64,
    sender: Mutex<mpsc::Sender<onebot::Frame>>,

}

impl Bot {
    pub fn on_frame(&self) {}
    pub fn send_and_wait() {}
}

