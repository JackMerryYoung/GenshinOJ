#![feature(thread_id_value)]

use dlopen2::wrapper::WrapperApi;
use std::io::Read;

static MAIN_TOKIO_RUNTIME: once_cell::sync::Lazy<tokio::runtime::Runtime> =
    once_cell::sync::Lazy::new(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
    });

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct SingleModuleConfigJson {
    name: String,
    id: String,
    enabled: bool,
    dependencies: Vec<String>,
    unload_timeout: usize,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct ModuleConfigJson {
    working_load: std::collections::HashMap<String, SingleModuleConfigJson>,
    restricted_mode: bool,
}

#[derive(dlopen2::wrapper::WrapperApi)]
struct ModuleInstance {
    on_init: extern "Rust" fn(rt: &tokio::runtime::Runtime) -> tokio::runtime::Runtime,
    on_unload: extern "Rust" fn(),
}

fn main() {
    let mut module_instances: Vec<dlopen2::wrapper::Container<ModuleInstance>> = vec![];
    let mut module_tokio_runtimes: Vec<tokio::runtime::Runtime> = vec![];
    let mut tmp: std::path::PathBuf = std::env::current_dir().unwrap();
    tmp.pop();
    let module_config_json_file_path: String =
        String::from(tmp.to_str().unwrap()) + "/module_config_rs.json";
    let mut module_config_json_file: std::fs::File = match std::fs::File::open(
        &module_config_json_file_path,
    ) {
        Ok(file) => file,
        Err(e) => {
            eprintln!(
                "[MAIN_BACKEND] [ERROR] [THREAD {}] Failed to open module_config_rs.json from `{}`. Maybe the file doesn't exist?",
                std::thread::current().id().as_u64(), module_config_json_file_path
            );
            eprintln!(
                "[MAIN_BACKEND] [THREAD {}] [ERROR] {}",
                std::thread::current().id().as_u64(),
                e
            );
            return;
        }
    };
    let mut module_config_json_string: String = String::new();
    module_config_json_file
        .read_to_string(&mut module_config_json_string)
        .unwrap();
    let module_config_json: ModuleConfigJson =
        serde_json::from_str(module_config_json_string.as_str()).unwrap();
    for (name, config) in module_config_json.working_load {
        if config.enabled {
            println!(
                "[MAIN_BACKEND] [INFO] [THREAD {}] Loading module {}...",
                std::thread::current().id().as_u64(),
                name
            );
            let path: String = String::from(tmp.to_str().unwrap())
                + "/rust_backend/modules/"
                + &config.id
                + "/lib"
                + &config.id
                + ".so";
            let module: Result<dlopen2::wrapper::Container<ModuleInstance>, dlopen2::Error> =
                unsafe { dlopen2::wrapper::Container::load(&path) };
            match module {
                Ok(module) => {
                    println!(
                        "[MAIN_BACKEND] [INFO] [THREAD {}] Successfully loaded module `{}` from `{}`.",
                        std::thread::current().id().as_u64(), name, &path
                    );

                    module_tokio_runtimes.push(module.on_init(&MAIN_TOKIO_RUNTIME));
                    module_instances.push(module);
                }
                Err(_e) => {
                    eprintln!(
                        "[MAIN_BACKEND] [ERROR] [THREAD {}] Failed to load module `{}` from `{}`. Maybe the module file doesn't exist or is not a valid shared object?",
                        std::thread::current().id().as_u64(), name, &path
                    );
                    if module_config_json.restricted_mode {
                        eprintln!("[MAIN_BACKEND] [ERROR] [THREAD {}] Due to the restricted mode, the server backend now is shutting down.", std::thread::current().id().as_u64());
                        panic!();
                    }
                }
            }
        }
    }

    MAIN_TOKIO_RUNTIME.block_on(async {
        tokio::signal::ctrl_c().await.unwrap();
        for module in &module_instances {
            module.on_unload();
        }
        println!(
            "[MAIN_BACKEND] [INFO] [THREAD {}] Successfully unloaded all the modules.",
            std::thread::current().id().as_u64()
        );
        println!(
            "[MAIN_BACKEND] [INFO] [THREAD {}] Now quitting...",
            std::thread::current().id().as_u64()
        );
        std::process::exit(0);
    });
}
