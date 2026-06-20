use crate::global::*;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SocketJsonMessageContentOnBindListener {
    pub protocol: String,
    pub version: String,
    pub commands_to_bind: Vec<String>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SocketJsonMessageContentOnUnbindListener {
    pub protocol: String,
    pub version: String,
    pub commands_to_unbind: Vec<String>,
}

fn userish_commands() -> Vec<String> {
    vec![
        String::from("on_user_profile"),
        String::from("on_follow"),
        String::from("on_unfollow"),
        String::from("on_friends_list"),
        String::from("on_search_users")
    ]
}

pub async fn connect_to_ws_server() {
    let msg_to_send = SocketJsonMessage {
        r#type: String::from("on_bind_listener"),
        content: serde_json
            ::to_value(SocketJsonMessageContentOnBindListener {
                protocol: String::from("std_userish"),
                version: String::from(env!("CARGO_PKG_VERSION")),
                commands_to_bind: userish_commands(),
            })
            .unwrap(),
        request_key: uuid::Uuid::new_v4().to_string(),
        from_protocol: String::from("std_userish"),
    };
    send_socket_json_message(&serde_json::to_value(msg_to_send).unwrap(), "std_ws_server").await;
}

pub async fn disconnect_from_ws_server() {
    let msg_to_send = SocketJsonMessage {
        r#type: String::from("on_unbind_listener"),
        content: serde_json
            ::to_value(SocketJsonMessageContentOnUnbindListener {
                protocol: String::from("std_userish"),
                version: String::from(env!("CARGO_PKG_VERSION")),
                commands_to_unbind: userish_commands(),
            })
            .unwrap(),
        request_key: uuid::Uuid::new_v4().to_string(),
        from_protocol: String::from("std_userish"),
    };
    send_socket_json_message(&serde_json::to_value(msg_to_send).unwrap(), "std_ws_server").await;
}
