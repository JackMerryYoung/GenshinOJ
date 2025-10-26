use crate::global::*;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SocketJsonMessageContentOnBindListener {
    pub protocol: String,
    pub commands_to_bind: Vec<String>,
}

pub async fn on_unbind_listener(msg: SocketJsonMessage) {
    if
        let Ok(content) = serde_json::from_value::<SocketJsonMessageContentOnBindListener>(
            msg.content
        )
    {
        let ws_server_external_listeners_by_command =
            WS_SERVER_EXTERNAL_LISTENERS_BY_COMMAND.get().unwrap();
        let mut guard_ws_server_external_listeners_by_command =
            ws_server_external_listeners_by_command.lock().await;
        for command_to_bind in content.commands_to_bind {
            if
                let Some(external_listener) =
                    (*guard_ws_server_external_listeners_by_command).get_mut(&command_to_bind)
            {
                let result = external_listener.protocols.remove(&content.protocol);
                if !result {
                    println!(
                        "{}",
                        ansi_term::Color::Yellow.paint(
                            format!(
                                "[WS_SERVER] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Someone tried to unbind a listener whose protocol doesn't exist.",
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!()
                            )
                        )
                    );
                }
            } else {
                println!(
                    "{}",
                    ansi_term::Color::Yellow.paint(
                        format!(
                            "[WS_SERVER] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Someone tried to unbind a listener whose command is not implemented.",
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!()
                        )
                    )
                );
            }
        }
    } else {
        println!(
            "{}",
            ansi_term::Color::Yellow.paint(
                format!(
                    "[WS_SERVER] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The JSON message received is in wrong format.",
                    std::thread::current().id().as_u64(),
                    file!(),
                    line!()
                )
            )
        );
    }
}
