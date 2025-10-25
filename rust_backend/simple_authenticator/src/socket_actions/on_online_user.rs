use crate::socket_message_processing::SocketJsonMessage;
use tokio::net::TcpStream;

pub async fn on_online_user(msg: SocketJsonMessage) {
    let msg: SocketJsonMessage = SocketJsonMessage {
        r#type: String::from("on_online_user"),
        content: serde_json::to_value(SocketJsonMessageContentOnOnlineUserResult {
            check_result: false,
        })
        .unwrap(),
        request_key: uuid::Uuid::new_v4().to_string(),
        from_protocol: msg.from_protocol,
    };
    let mut socket: tokio::net::TcpStream =
        crate::simple_authenticator_socket::get_socket_by_protocol(&msg.from_protocol).await;
    socket.writable().await.unwrap();
    socket
        .write_all(serde_json::to_string(&msg).unwrap().as_bytes())
        .await
        .unwrap();
    socket.flush().await.unwrap();
}
    