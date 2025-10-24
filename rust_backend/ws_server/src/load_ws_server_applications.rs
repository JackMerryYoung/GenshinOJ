use crate::global::*;
use std::io::Read;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct WebsocketServerApplicationsConfigJson {
    pub ws_server_applications: std::collections::HashMap<String, SingleWebsocketApplicationConfigJson>,
    pub restricted_mode: bool,
}

pub fn load_ws_server_applications(
    rt: &tokio::runtime::Runtime,
    ws_server_applications_libraries: &mut Vec<(dlopen2::symbor::Library, tokio::runtime::Runtime)>,
    global_module_statuses_by_protocol: AsyncModifiable<
        std::collections::HashMap<String, AsyncModifiable<ModuleStatus>>,
    >,
    ws_server_applications_config_json: &WebsocketServerApplicationsConfigJson,
) -> Result<(), dlopen2::Error> {
    for (name, config) in &ws_server_applications_config_json.ws_server_applications {
        if config.enabled {
            println!(
                "{}", ansi_term::Color::Blue.paint(
                    format!(
                        "[WS_SERVER] [INFO] [THREAD {}] [FILE `{}` LINE {}] Loading websocket server application {}...",
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!(),
                        name
                    )
                )
            );
            let ws_server_application_library_file_path: String = get_parent_path()
                + "/rust_backend/modules/ws_server/assets/lib/lib"
                + &config.id
                + ".so"; // Get the path of the module.
            let ws_server_application_library: Result<dlopen2::symbor::Library, dlopen2::Error> =
                dlopen2::symbor::Library::open(&ws_server_application_library_file_path);

            match ws_server_application_library {
                Ok(ws_server_application_library) => {
                    println!(
                        "{}", ansi_term::Color::Blue.paint(
                            format!(
                                "[WS_SERVER] [INFO] [THREAD {}] [FILE `{}` LINE {}] Successfully loaded websocket server application `{}` from `{}`.",
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!(),
                                name,
                                &ws_server_application_library_file_path
                            )
                        )
                    );

                    if let Ok(callback) = unsafe {
                        ws_server_application_library
                            .symbol::<unsafe extern "Rust" fn(
                            &tokio::runtime::Runtime,
                            AsyncModifiable<
                                std::collections::HashMap<String, AsyncModifiable<ModuleStatus>>,
                            >,
                        ) -> tokio::runtime::Runtime>("on_init")
                    } {
                        let ws_server_application_library_runtime: tokio::runtime::Runtime =
                            unsafe { callback(rt, global_module_statuses_by_protocol.clone()) };
                        ws_server_applications_libraries.push((
                            ws_server_application_library,
                            ws_server_application_library_runtime,
                        ));
                    }
                }
                Err(e) => {
                    eprintln!(
                        "{}", ansi_term::Color::Red.paint(
                            format!(
                                "[WS_SERVER] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Failed to load websocket server application `{}` from `{}`. Maybe the websocket server application file doesn't exist or is not a valid shared object?",
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!(),
                                name,
                                &ws_server_application_library_file_path
                            )
                        )
                    );

                    if ws_server_applications_config_json.restricted_mode {
                        eprintln!(
                            "{}", ansi_term::Color::Red.paint(
                                format!(
                                    "[WS_SERVER] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Due to the restricted mode, the websocket server now is shutting down.",
                                    std::thread::current().id().as_u64(),
                                    file!(),
                                    line!()
                                )
                            )
                        );
                        return Err(e);
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn parse_ws_server_applications_config_json() -> WebsocketServerApplicationsConfigJson {
    let mut ws_server_applications_config_json_string: String = String::new();
    let ws_server_applications_config_json_file_path: String =
        get_parent_path() + "/rust_backend/ws_server/ws_server_config_rs.json";
    let mut ws_server_applications_config_json_file: std::fs::File = match std::fs::File::open(
        &ws_server_applications_config_json_file_path,
    ) {
        Ok(file) => file,
        Err(e) => {
            eprintln!(
                "{}", ansi_term::Color::Red.paint(
                    format!(
                        "[WS_SERVER] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Failed to open ws_server_config_rs.json from `{}`. Maybe the file doesn't exist?",
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!(),
                        ws_server_applications_config_json_file_path
                    )
                )
            );
            eprintln!(
                "{}", ansi_term::Color::Red.paint(
                    format!(
                        "[WS_SERVER] [ERROR] [THREAD {}] [FILE `{}` LINE {}] {}",
                        file!(),
                        line!(),
                        std::thread::current().id().as_u64(),
                        e
                    )
                )
            );
            panic!();
        }
    };
    ws_server_applications_config_json_file
        .read_to_string(&mut ws_server_applications_config_json_string)
        .unwrap(); // Read module config json file to string.

    serde_json::from_str(ws_server_applications_config_json_string.as_str()).unwrap()
}
