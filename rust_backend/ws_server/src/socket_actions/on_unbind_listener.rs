use crate::global::*;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SocketJsonMessageContentOnUnbindListener {
    pub protocol: String,
    pub version: String,
    pub commands_to_unbind: Vec<String>,
}

pub async fn on_unbind_listener(msg: SocketJsonMessage) {
    if
        let Ok(content) = serde_json::from_value::<SocketJsonMessageContentOnUnbindListener>(
            msg.content
        )
    {
        let ws_server_external_listeners_by_command =
            WS_SERVER_EXTERNAL_LISTENERS_BY_COMMAND.get().unwrap();
        let mut guard_ws_server_external_listeners_by_command =
            ws_server_external_listeners_by_command.write().unwrap();

        // Copy-on-write, same reasoning as on_bind_listener.rs: Arc::get_mut would
        // unpredictably fail whenever a reader (ws_handler.rs) holds its own clone
        // of the Arc outside the lock, even though we hold the write lock here.
        let mut new_map: std::collections::HashMap<String, ExternalListener> =
            (**guard_ws_server_external_listeners_by_command).clone();

        for command_to_bind in content.commands_to_unbind {
            if let Some(external_listener) = new_map.get_mut(&command_to_bind) {
                match external_listener.protocols.remove(&content.protocol) {
                    Some(_) => {
                        println!(
                            "{}",
                            ansi_term::Color::Blue.paint(
                                format!(
                                    "[{}] [INFO] [THREAD {}] [FILE `{}` LINE {}] Protocol `{}` (version `{}`) unbound `{}`.",
                                    MODULE_IDENTITY,
                                    std::thread::current().id().as_u64(),
                                    file!(),
                                    line!(),
                                    content.protocol,
                                    content.version,
                                    external_listener.command
                                )
                            )
                        );
                    }
                    None => {
                        println!(
                            "{}",
                            ansi_term::Color::Yellow.paint(
                                format!(
                                    "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Someone tried to unbind a listener whose protocol doesn't exist.",
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
                            "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Someone tried to unbind a listener whose command is not implemented.",
                            MODULE_IDENTITY,
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!()
                        )
                    )
                );
            }
        }

        *guard_ws_server_external_listeners_by_command = std::sync::Arc::new(new_map);
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