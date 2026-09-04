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

pub async fn connect_to_ws_server() {
    let msg_to_send = SocketJsonMessage {
        r#type: String::from("on_bind_listener"),
        content: serde_json
            ::to_value(SocketJsonMessageContentOnBindListener {
                protocol: String::from("std_authenticator"),
                version: String::from(env!("CARGO_PKG_VERSION")),
                commands_to_bind: vec![
                    String::from("on_close_connection"),
                    String::from("on_login_check"),
                    String::from("on_login"),
                    String::from("on_online_user"),
                    String::from("on_quit"),
                    String::from("on_register"),
                    String::from("on_session_restore")
                ],
            })
            .unwrap(),
        request_key: uuid::Uuid::new_v4().to_string(),
        from_protocol: String::from("std_authenticator"),
    };
    send_socket_json_message(&serde_json::to_value(msg_to_send).unwrap(), "std_ws_server").await;
}

pub async fn disconnect_from_ws_server() {
    let msg_to_send = SocketJsonMessage {
        r#type: String::from("on_unbind_listener"),
        content: serde_json
            ::to_value(SocketJsonMessageContentOnUnbindListener {
                protocol: String::from("std_authenticator"),
                version: String::from(env!("CARGO_PKG_VERSION")),
                commands_to_unbind: vec![
                    String::from("on_close_connection"),
                    String::from("on_login_check"),
                    String::from("on_login"),
                    String::from("on_online_user"),
                    String::from("on_quit"),
                    String::from("on_register"),
                    String::from("on_session_restore")
                ],
            })
            .unwrap(),
        request_key: uuid::Uuid::new_v4().to_string(),
        from_protocol: String::from("std_authenticator"),
    };
    send_socket_json_message(&serde_json::to_value(msg_to_send).unwrap(), "std_ws_server").await;
}
