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
use std::sync::{Arc, Mutex};
use tokio::sync::{broadcast, oneshot};
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
                                    if let Some(api_resp_sender) = bot.resp_promises.lock().unwrap().remove(frame.echo.clone().as_str()) {
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

#[derive(Clone)]
struct Bot {
    bot_id: i64,
    api_sender: Sender<onebot::Frame>,
    resp_promises: Arc<Mutex<HashMap<String, oneshot::Sender<onebot::Frame>>>>,
}

impl Bot {
    pub fn on_frame(&self) {}
    pub async fn send_and_wait(&mut self, data: Data) -> Option<Data> {
        // 构造API请求
        let echo: String = uuid::Uuid::new_v4().to_simple().to_string();
        let req_frame_type = get_frame_type(&data).into();
        let api_req_frame = Frame {
            bot_id: self.bot_id,
            frame_type: req_frame_type,
            echo: echo.clone(),
            ok: true,
            extra: Default::default(),
            data: Some(data),
        };

        // 发送API请求
        let mut buf = Vec::new();
        prost::Message::encode(&api_req_frame, &mut buf);
        let api_sender = mpsc::Sender::clone(&self.api_sender);
        api_sender.send(api_req_frame).await;

        // 等待API响应
        let (mut resp_sender, mut resp_receiver) = oneshot::channel();
        self.resp_promises.lock().unwrap().insert(echo.clone(), resp_sender);
        let api_resp_frame = resp_receiver.await.unwrap();
        return api_resp_frame.data;
    }
    pub async fn send_private_message(&mut self, user_id: i64, content: String) -> Option<SendPrivateMsgResp> {
        let resp = self.send_and_wait(Data::SendPrivateMsgReq(SendPrivateMsgReq {
            user_id,
            message: vec![
                onebot::Message {
                    r#type: "text".to_string(),
                    data: [
                        ("text".to_string(), content),
                    ].iter().cloned().collect(),
                }
            ],
            auto_escape: false,
        })).await;
        if let Some(Data::SendPrivateMsgResp(resp)) = resp {
            Some(resp)
        } else {
            None
        }
    }
    pub async fn send_group_message(&mut self, group_id: i64, content: String) -> Option<SendGroupMsgResp> {
        let resp = self.send_and_wait(Data::SendGroupMsgReq(SendGroupMsgReq {
            group_id,
            message: vec![
                onebot::Message {
                    r#type: "text".to_string(),
                    data: [
                        ("text".to_string(), content),
                    ].iter().cloned().collect(),
                }
            ],
            auto_escape: false,
        })).await;
        if let Some(Data::SendGroupMsgResp(resp)) = resp {
            Some(resp)
        } else {
            None
        }
    }
    pub async fn delete_msg(&mut self, message_id: i32) -> Option<DeleteMsgResp> {
        let resp = self.send_and_wait(Data::DeleteMsgReq(DeleteMsgReq {
            message_id
        })).await;
        if let Some(Data::DeleteMsgResp(resp)) = resp {
            Some(resp)
        } else {
            None
        }
    }
    pub async fn get_group_list(&mut self) -> Option<GetGroupListResp> {
        let resp = self.send_and_wait(Data::GetGroupListReq(GetGroupListReq {})).await;
        if let Some(Data::GetGroupListResp(resp)) = resp {
            Some(resp)
        } else {
            None
        }
    }
}


fn get_frame_type(data: &Data) -> onebot::frame::FrameType {
    match data {
        Data::SendPrivateMsgReq(_) => { FrameType::TSendPrivateMsgReq }
        Data::SendGroupMsgReq(_) => { FrameType::TSendGroupMsgReq }
        Data::SendMsgReq(_) => { FrameType::TSendGroupMsgReq }
        Data::DeleteMsgReq(_) => { FrameType::TDeleteMsgReq }
        Data::GetMsgReq(_) => { FrameType::TGetMsgReq }
        Data::GetForwardMsgReq(_) => { FrameType::TGetForwardMsgReq }
        Data::SendLikeReq(_) => { FrameType::TSendLikeReq }
        Data::SetGroupKickReq(_) => { FrameType::TSetGroupKickReq }
        Data::SetGroupBanReq(_) => { FrameType::TSetGroupBanReq }
        Data::SetGroupAnonymousBanReq(_) => { FrameType::TSetGroupAnonymousBanReq }
        Data::SetGroupWholeBanReq(_) => { FrameType::TSetGroupWholeBanReq }
        Data::SetGroupAdminReq(_) => { FrameType::TSetGroupAdminReq }
        Data::SetGroupAnonymousReq(_) => { FrameType::TSetGroupAnonymousReq }
        Data::SetGroupCardReq(_) => { FrameType::TSetGroupCardReq }
        Data::SetGroupNameReq(_) => { FrameType::TSetGroupNameReq }
        Data::SetGroupLeaveReq(_) => { FrameType::TSetGroupLeaveReq }
        Data::SetGroupSpecialTitleReq(_) => { FrameType::TSetGroupSpecialTitleReq }
        Data::SetFriendAddRequestReq(_) => { FrameType::TSetFriendAddRequestReq }
        Data::SetGroupAddRequestReq(_) => { FrameType::TSetGroupAddRequestReq }
        Data::GetLoginInfoReq(_) => { FrameType::TGetLoginInfoReq }
        Data::GetStrangerInfoReq(_) => { FrameType::TGetStrangerInfoReq }
        Data::GetFriendListReq(_) => { FrameType::TGetFriendListReq }
        Data::GetGroupInfoReq(_) => { FrameType::TGetGroupInfoReq }
        Data::GetGroupListReq(_) => { FrameType::TGetGroupListReq }
        Data::GetGroupMemberInfoReq(_) => { FrameType::TGetGroupMemberInfoReq }
        Data::GetGroupMemberListReq(_) => { FrameType::TGetGroupMemberListReq }
        Data::GetGroupHonorInfoReq(_) => { FrameType::TGetGroupHonorInfoReq }
        Data::GetCookiesReq(_) => { FrameType::TGetCookiesReq }
        Data::GetCsrfTokenReq(_) => { FrameType::TGetCsrfTokenReq }
        Data::GetCredentialsReq(_) => { FrameType::TGetCredentialsReq }
        Data::GetRecordReq(_) => { FrameType::TGetRecordReq }
        Data::GetImageReq(_) => { FrameType::TGetImageReq }
        Data::CanSendImageReq(_) => { FrameType::TCanSendImageReq }
        Data::CanSendRecordReq(_) => { FrameType::TCanSendRecordReq }
        Data::GetStatusReq(_) => { FrameType::TGetStatusReq }
        Data::GetVersionInfoReq(_) => { FrameType::TGetVersionInfoReq }
        Data::SetRestartReq(_) => { FrameType::TSetRestartReq }
        Data::CleanCacheReq(_) => { FrameType::TCleanCacheReq }
        _ => { FrameType::Tunknown }
    }
}


