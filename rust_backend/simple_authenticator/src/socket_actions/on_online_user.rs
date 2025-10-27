use crate::global::*;

#[derive(serde::Deserialize, serde::Serialize)]
struct SocketJsonMessageContentOnOnlineUserResult {
    check_result: bool,
    request_key: String,
}

pub async fn on_online_user(msg: SocketJsonMessageWithWsId) {
    let msg: SocketJsonMessage = SocketJsonMessage {
        r#type: String::from("on_online_user"),
        content: serde_json
            ::to_value(SocketJsonMessageContentOnOnlineUserResult {
                check_result: false,
                request_key: msg.request_key,
            })
            .unwrap(),
        request_key: uuid::Uuid::new_v4().to_string(),
        from_protocol: msg.from_protocol,
    };
}
