use crate::global::*;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SocketJsonMessageContentOnBindListener {
    pub protocol: String,
    pub commands_to_bind: Vec<String>,
}

pub async fn on_bind_listener(msg: SocketJsonMessage) {
    if
        let Ok(content) = serde_json::from_value::<SocketJsonMessageContentOnBindListener>(
            msg.content
        )
    {
        let mut guard_ws_server_external_listeners_by_command =
            WS_SERVER_EXTERNAL_LISTENERS_BY_COMMAND.lock().await;
        for command_to_bind in content.commands_to_bind {
            if
                let Some(external_listener) =
                    (*guard_ws_server_external_listeners_by_command).get_mut(&command_to_bind)
            {
                let result = external_listener.protocols.insert(content.protocol.clone());
                if !result {
                    println!(
                        "{}",
                        ansi_term::Color::Yellow.paint(
                            format!(
                                "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Someone tried to bind a listener whose protocol exists.",
                                MODULE_IDENTITY,
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
                            "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Someone tried to bind a listener whose command is not implemented.",
                            MODULE_IDENTITY,
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
                    "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] The JSON message received is in wrong format.",
                    MODULE_IDENTITY,
                    std::thread::current().id().as_u64(),
                    file!(),
                    line!()
                )
            )
        );
    }
}
