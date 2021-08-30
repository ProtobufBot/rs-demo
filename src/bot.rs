use crate::onebot;
use std::sync::Arc;
use tokio::sync::{Mutex, oneshot, mpsc};
use std::collections::HashMap;
use crate::onebot::frame::{Data, FrameType};
use crate::onebot::*;

#[derive(Clone)]
pub struct Bot {
    pub bot_id: i64,
    pub api_sender: mpsc::Sender<onebot::Frame>,
    pub resp_promises: Arc<Mutex<HashMap<String, oneshot::Sender<onebot::Frame>>>>,
}

impl Bot {
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
        self.resp_promises.lock().await.insert(echo.clone(), resp_sender);
        let api_resp_frame = resp_receiver.await.unwrap();
        return api_resp_frame.data;
    }

    ///
    /// 发送私聊消息
    ///
    /// @param user_id          对方 QQ 号
    /// @param content          消息内容
    /// @return 结果
    ///
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

    ///
    /// 发送群消息
    ///
    /// @param group_id         群号
    /// @param content          消息
    /// @return 结果
    ///
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

    ///
    /// 撤回消息
    ///
    /// @param message_id 消息 ID
    /// @return 结果
    ///
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

    ///
    /// 获取消息
    ///
    /// @param message_id 消息 ID
    /// @return 结果
    ///
    pub async fn get_msg(&mut self, message_id: i32) -> Option<GetMsgResp> {
        let resp = self.send_and_wait(Data::GetMsgReq(GetMsgReq {
            message_id
        })).await;
        if let Some(Data::GetMsgResp(resp)) = resp {
            Some(resp)
        } else {
            None
        }
    }

    ///
    /// 群组踢人
    ///
    /// @param group_id           群号
    /// @param user_id            要踢的 QQ 号
    /// @param reject_add_request 拒绝此人的加群请求
    /// @return 结果
    ///
    pub async fn set_group_kick(&mut self, group_id: i64, user_id: i64, reject_add_request: bool) -> Option<SetGroupKickResp> {
        let resp = self.send_and_wait(Data::SetGroupKickReq(SetGroupKickReq {
            group_id,
            user_id,
            reject_add_request,
        })).await;
        if let Some(Data::SetGroupKickResp(resp)) = resp {
            Some(resp)
        } else {
            None
        }
    }

    ///
    /// 群组单人禁言
    ///
    /// @param group_id 群号
    /// @param user_id  要禁言的 QQ 号
    /// @param duration 禁言时长，单位秒，0 表示取消禁言
    /// @return 结果
    ///
    pub async fn set_group_ban(&mut self, group_id: i64, user_id: i64, duration: i32) -> Option<SetGroupBanResp> {
        let resp = self.send_and_wait(Data::SetGroupBanReq(SetGroupBanReq {
            group_id,
            user_id,
            duration,
        })).await;
        if let Some(Data::SetGroupBanResp(resp)) = resp {
            Some(resp)
        } else {
            None
        }
    }

    ///
    /// 群组全员禁言
    ///
    /// @param group_id 群号
    /// @param enable   是否禁言
    /// @return 结果
    ///
    pub async fn set_group_whole_ban(&mut self, group_id: i64, enable: bool) -> Option<SetGroupWholeBanResp> {
        let resp = self.send_and_wait(Data::SetGroupWholeBanReq(SetGroupWholeBanReq {
            group_id,
            enable,
        })).await;
        if let Some(Data::SetGroupWholeBanResp(resp)) = resp {
            Some(resp)
        } else {
            None
        }
    }

    ///
    /// 设置群名片（群备注）
    ///
    /// @param group_id 群号
    /// @param user_id  要设置的 QQ 号
    /// @param card     群名片内容，空字符串表示删除群名片
    /// @return 结果
    ///
    pub async fn set_group_card(&mut self, group_id: i64, user_id: i64, card: String) -> Option<SetGroupCardResp> {
        let resp = self.send_and_wait(Data::SetGroupCardReq(SetGroupCardReq {
            group_id,
            user_id,
            card,
        })).await;
        if let Some(Data::SetGroupCardResp(resp)) = resp {
            Some(resp)
        } else {
            None
        }
    }

    ///
    /// 解散群组
    ///
    /// @param group_id   群号
    /// @param is_dismiss 是否解散，如果登录号是群主，则仅在此项为 true 时能够解散
    /// @return 结果
    ///
    pub async fn set_group_leave(&mut self, group_id: i64, is_dismiss: bool) -> Option<SetGroupLeaveResp> {
        let resp = self.send_and_wait(Data::SetGroupLeaveReq(SetGroupLeaveReq {
            group_id,
            is_dismiss,
        })).await;
        if let Some(Data::SetGroupLeaveResp(resp)) = resp {
            Some(resp)
        } else {
            None
        }
    }

    ///
    /// 设置群组专属头衔
    ///
    /// @param group_id      群号
    /// @param user_id       要设置的 QQ 号
    /// @param special_title 专属头衔，不填或空字符串表示删除专属头衔
    /// @param duration      专属头衔有效期，单位秒，-1 表示永久，不过此项似乎没有效果，可能是只有某些特殊的时间长度有效，有待测试
    /// @return 结果
    ///
    pub async fn set_group_special_title(&mut self, group_id: i64, user_id: i64, special_title: String, duration: i64) -> Option<SetGroupSpecialTitleResp> {
        let resp = self.send_and_wait(Data::SetGroupSpecialTitleReq(SetGroupSpecialTitleReq {
            group_id,
            user_id,
            special_title,
            duration,
        })).await;
        if let Some(Data::SetGroupSpecialTitleResp(resp)) = resp {
            Some(resp)
        } else {
            None
        }
    }

    ///
    /// 处理加好友请求
    ///
    /// @param flag    加好友请求的 flag（需从上报的数据中获得）
    /// @param approve 是否同意请求
    /// @param remark  添加后的好友备注（仅在同意时有效）
    /// @return 结果
    ///
    pub async fn set_friend_add_request(&mut self, flag: String, approve: bool, remark: String) -> Option<SetFriendAddRequestResp> {
        let resp = self.send_and_wait(Data::SetFriendAddRequestReq(SetFriendAddRequestReq {
            flag,
            approve,
            remark,
        })).await;
        if let Some(Data::SetFriendAddRequestResp(resp)) = resp {
            Some(resp)
        } else {
            None
        }
    }

    ///
    /// 处理加群请求／邀请
    ///
    /// @param flag     加群请求的 flag（需从上报的数据中获得）
    /// @param sub_type add 或 invite，请求类型（需要和上报消息中的 sub_type 字段相符）
    /// @param approve  是否同意请求／邀请
    /// @param reason   拒绝理由（仅在拒绝时有效）
    /// @return 结果
    ///
    // TODO r#type 不知道怎么写
    pub async fn set_group_add_request(&mut self, flag: String, sub_type: String, approve: bool, reason: String) -> Option<SetGroupAddRequestResp> {
        let resp = self.send_and_wait(Data::SetGroupAddRequestReq(SetGroupAddRequestReq {
            flag,
            sub_type,
            r#type: "".to_string(),
            approve,
            reason,
        })).await;
        if let Some(Data::SetGroupAddRequestResp(resp)) = resp {
            Some(resp)
        } else {
            None
        }
    }

    ///
    /// 获取登录号信息
    ///
    /// @return 结果
    ///
    pub async fn get_login_info(&mut self) -> Option<GetLoginInfoResp> {
        let resp = self.send_and_wait(Data::GetLoginInfoReq(GetLoginInfoReq {})).await;
        if let Some(Data::GetLoginInfoResp(resp)) = resp {
            Some(resp)
        } else {
            None
        }
    }

    ///
    /// 获取陌生人信息
    ///
    /// @param user_id   QQ号
    /// @return 结果
    ///
    pub async fn get_stranger_info(&mut self, user_id: i64) -> Option<GetStrangerInfoResp> {
        let resp = self.send_and_wait(Data::GetStrangerInfoReq(GetStrangerInfoReq {
            user_id,
            no_cache: false,
        })).await;
        if let Some(Data::GetStrangerInfoResp(resp)) = resp {
            Some(resp)
        } else {
            None
        }
    }

    ///
    /// 获取好友列表
    ///
    /// @return 结果
    ///
    pub async fn get_friend_list(&mut self) -> Option<GetFriendListResp> {
        let resp = self.send_and_wait(Data::GetFriendListReq(GetFriendListReq {})).await;
        if let Some(Data::GetFriendListResp(resp)) = resp {
            Some(resp)
        } else {
            None
        }
    }

    ///
    /// 获取群列表
    ///
    /// @return 结果
    ///
    pub async fn get_group_list(&mut self) -> Option<GetGroupListResp> {
        let resp = self.send_and_wait(Data::GetGroupListReq(GetGroupListReq {})).await;
        if let Some(Data::GetGroupListResp(resp)) = resp {
            Some(resp)
        } else {
            None
        }
    }

    ///
    /// 获取群信息
    ///
    /// @param group_id 群号
    /// @param no_cache 是否不使用缓存（使用缓存可能更新不及时，但响应更快）
    /// @return 结果
    ///
    pub async fn get_group_info(&mut self, group_id: i64, no_cache: bool) -> Option<GetGroupInfoResp> {
        let resp = self.send_and_wait(Data::GetGroupInfoReq(GetGroupInfoReq {
            group_id,
            no_cache,
        })).await;
        if let Some(Data::GetGroupInfoResp(resp)) = resp {
            Some(resp)
        } else {
            None
        }
    }

    ///
    /// 获取群成员信息
    ///
    /// @param group_id 群号
    /// @param user_id  QQ 号
    /// @param no_cache 是否不使用缓存（使用缓存可能更新不及时，但响应更快）
    /// @return 结果
    ///
    pub async fn get_group_member_info(&mut self, group_id: i64, user_id: i64, no_cache: bool) -> Option<GetGroupMemberInfoResp> {
        let resp = self.send_and_wait(Data::GetGroupMemberInfoReq(GetGroupMemberInfoReq {
            group_id,
            user_id,
            no_cache,
        })).await;
        if let Some(Data::GetGroupMemberInfoResp(resp)) = resp {
            Some(resp)
        } else {
            None
        }
    }

    ///
    /// 获取群成员列表
    ///
    ///
    /// 响应内容为 JSON 数组，每个元素的内容和上面的 /get_group_member_info 接口相同，但对于同一个群组的同一个成员，获取列表时和获取单独的成员信息时，某些字段可能有所不同，例如 area、title 等字段在获取列表时无法获得，具体应以单独的成员信息为准。
    ///
    /// @param group_id 群号
    /// @return 结果
    ///
    pub async fn get_group_member_list(&mut self, group_id: i64) -> Option<GetGroupListResp> {
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