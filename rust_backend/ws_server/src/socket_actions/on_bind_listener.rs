use crate::global::*;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SocketJsonMessageContentOnBindListener {
    pub protocol: String,
    pub version: String,
    pub commands_to_bind: Vec<String>,
}

pub async fn on_bind_listener(msg: SocketJsonMessage) {
    if
        let Ok(content) = serde_json::from_value::<SocketJsonMessageContentOnBindListener>(
            msg.content
        )
    {
        let Ok(protocol_version) = semver::Version::parse(&content.version) else {
            println!(
                "{}",
                ansi_term::Color::Yellow.paint(
                    format!(
                        "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Protocol `{}` tried to bind with an invalid version string `{}`.",
                        MODULE_IDENTITY,
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!(),
                        content.protocol,
                        content.version
                    )
                )
            );
            return;
        };

        let ws_server_external_listeners_by_command =
            WS_SERVER_EXTERNAL_LISTENERS_BY_COMMAND.get().unwrap();
        let mut guard_ws_server_external_listeners_by_command =
            ws_server_external_listeners_by_command.write().unwrap();

        // Copy-on-write: clone the current snapshot's contents into a fresh map,
        // mutate the fresh map, then atomically publish it as a new Arc.
        // We can't mutate through the existing Arc in place, because readers
        // (ws_handler.rs) hold their own clones of it outside the lock — so
        // Arc::get_mut would unpredictably fail whenever a reader's clone is
        // still alive, even though we hold the write lock.
        let mut new_map: std::collections::HashMap<String, ExternalListener> =
            (**guard_ws_server_external_listeners_by_command).clone();

        for command_to_bind in content.commands_to_bind {
            if let Some(external_listener) = new_map.get_mut(&command_to_bind) {
                if !external_listener.required_protocol_version.matches(&protocol_version) {
                    println!(
                        "{}",
                        ansi_term::Color::Yellow.paint(
                            format!(
                                "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Protocol `{}` version `{}` does not satisfy the required version `{}` for command `{}`.",
                                MODULE_IDENTITY,
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!(),
                                content.protocol,
                                protocol_version,
                                external_listener.required_protocol_version,
                                external_listener.command
                            )
                        )
                    );
                    continue;
                }
                if
                    external_listener.protocols
                        .insert(content.protocol.clone(), protocol_version.clone())
                        .is_none()
                {
                    println!(
                        "{}",
                        ansi_term::Color::Green.paint(
                            format!(
                                "[{}] [INFO] [THREAD {}] [FILE `{}` LINE {}] Protocol `{}` (version `{}`) successfully binded `{}`.",
                                MODULE_IDENTITY,
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!(),
                                content.protocol,
                                protocol_version,
                                command_to_bind
                            )
                        )
                    );
                } else {
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