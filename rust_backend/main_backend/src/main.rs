#![feature(thread_id_value)]
#![allow(clippy::type_complexity)]

use dlopen2::wrapper::WrapperApi;
use std::io::Read;

static MAIN_TOKIO_RUNTIME: once_cell::sync::Lazy<tokio::runtime::Runtime> =
    once_cell::sync::Lazy::new(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
    });

static MAIN_BACKEND_PANIC_FLAG: std::sync::LazyLock<AsyncModifiable<bool>> =
    std::sync::LazyLock::new(|| std::sync::Arc::new(tokio::sync::Mutex::new(false)));

#[derive(serde::Deserialize, serde::Serialize)]
struct SingleModuleConfigJson {
    name: String,
    id: String,
    enabled: bool,
    dependencies: Vec<String>,
    unload_timeout: usize,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct ModuleConfigJson {
    working_load: std::collections::HashMap<String, SingleModuleConfigJson>,
    restricted_mode: bool,
}

type AsyncModifiable<T> = std::sync::Arc<tokio::sync::Mutex<T>>;

#[derive(dlopen2::wrapper::WrapperApi)]
struct ModuleInstance {
    on_init: extern "Rust" fn(
        rt: &tokio::runtime::Runtime,
    ) -> (tokio::runtime::Runtime, AsyncModifiable<ModuleStatus>),
    on_unload: extern "Rust" fn(),
}

pub struct ModuleStatus {
    initialized: bool,
    panicked: bool,
}

struct ModuleCombination {
    name: String,
    instance: dlopen2::wrapper::Container<ModuleInstance>,
    tokio_runtime: tokio::runtime::Runtime,
    status: AsyncModifiable<ModuleStatus>,
}

fn main() {
    let module_combinations: AsyncModifiable<Vec<ModuleCombination>> =
        std::sync::Arc::new(tokio::sync::Mutex::new(vec![]));
    let module_config_json: AsyncModifiable<ModuleConfigJson> =
        MAIN_TOKIO_RUNTIME.block_on(async {
            std::sync::Arc::new(tokio::sync::Mutex::new(parse_module_config_json().await)) // Parse from module config json string.
        });
    {
        let module_combinations: AsyncModifiable<Vec<ModuleCombination>> =
            std::sync::Arc::clone(&module_combinations);
        let module_config_json: AsyncModifiable<ModuleConfigJson> =
            std::sync::Arc::clone(&module_config_json);
        MAIN_TOKIO_RUNTIME.block_on(async move {
            let mut guard_module_combinations: tokio::sync::MutexGuard<'_, Vec<ModuleCombination>> =
                module_combinations.lock().await;
            let guard_module_config_json: tokio::sync::MutexGuard<'_, ModuleConfigJson> =
                module_config_json.lock().await;
            load_modules(&mut guard_module_combinations, &guard_module_config_json).await; // Load modules.
        });
    }

    {
        let module_combinations: AsyncModifiable<Vec<ModuleCombination>> =
            module_combinations.clone();
        let module_config_json: AsyncModifiable<ModuleConfigJson> = module_config_json.clone();
        let main_backend_panic_flag: std::sync::Arc<tokio::sync::Mutex<bool>> =
            MAIN_BACKEND_PANIC_FLAG.clone();
        MAIN_TOKIO_RUNTIME.spawn(async move {
            loop {
                let guard_module_combinations: tokio::sync::MutexGuard<
                    '_,
                    Vec<ModuleCombination>
                > = module_combinations.lock().await;
                let guard_module_config_json: tokio::sync::MutexGuard<
                    '_,
                    ModuleConfigJson
                > = module_config_json.lock().await;
                for x in guard_module_combinations.iter() {
                    let guard_status: tokio::sync::MutexGuard<
                        '_,
                        ModuleStatus
                    > = x.status.lock().await;
                    if guard_status.panicked {
                        println!(
                            "[MAIN_BACKEND] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Module {} has panicked.",
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!(),
                            x.name
                        );
                        if guard_module_config_json.restricted_mode {
                            eprintln!(
                                "[MAIN_BACKEND] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Due to the restricted mode, the server backend now is shutting down.",
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!()
                            );
                            drop(guard_status);
                            drop(guard_module_config_json);
                            println!(
                                "[MAIN_BACKEND] [INFO] [THREAD {}] [FILE `{}` LINE {}] Now quitting...",
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!()
                            );
                            let mut guard_main_backend_panic_flag: tokio::sync::MutexGuard<
                                '_,
                                bool
                            > = main_backend_panic_flag.lock().await;
                            *guard_main_backend_panic_flag = true;
                            drop(guard_main_backend_panic_flag);
                            panic!();
                        }
                    }
                    drop(guard_status);
                    fake_yield_now().await;
                }
                drop(guard_module_combinations);
                drop(guard_module_config_json);
                fake_yield_now().await;
            }
        });
    }

    {
        let module_combinations: AsyncModifiable<Vec<ModuleCombination>> =
            module_combinations.clone();
        MAIN_TOKIO_RUNTIME.block_on(async move {
            // Wait fot Ctrl+C.
            tokio::signal::ctrl_c().await.unwrap();
            let guard_module_combinations: tokio::sync::MutexGuard<
                '_,
                Vec<ModuleCombination>
            > = module_combinations.lock().await;
            for module in guard_module_combinations.iter() {
                module.instance.on_unload(); // Unload each module.
            }
            drop(guard_module_combinations);
            println!(
                "[MAIN_BACKEND] [INFO] [THREAD {}] [FILE `{}` LINE {}] Successfully unloaded all the modules.",
                std::thread::current().id().as_u64(),
                file!(),
                line!()
            );
            println!(
                "[MAIN_BACKEND] [INFO] [THREAD {}] [FILE `{}` LINE {}] Now quitting...",
                std::thread::current().id().as_u64(),
                file!(),
                line!()
            );
            std::process::exit(0);
        });
    }

    {
        let module_combinations: AsyncModifiable<Vec<ModuleCombination>> =
            module_combinations.clone();
        let main_backend_panic_flag: std::sync::Arc<tokio::sync::Mutex<bool>> =
            MAIN_BACKEND_PANIC_FLAG.clone();
        MAIN_TOKIO_RUNTIME.block_on(async move {
            loop {
                let guard_main_backend_panic_flag: tokio::sync::MutexGuard<
                    '_,
                    bool
                > = main_backend_panic_flag.lock().await;
                if *guard_main_backend_panic_flag {
                    break;
                }
            }
            let guard_module_combinations: tokio::sync::MutexGuard<
                '_,
                Vec<ModuleCombination>
            > = module_combinations.lock().await;
            for module in guard_module_combinations.iter() {
                module.instance.on_unload(); // Unload each module.
            }
            drop(guard_module_combinations);
            println!(
                "[MAIN_BACKEND] [INFO] [THREAD {}] [FILE `{}` LINE {}] Successfully unloaded all the modules.",
                std::thread::current().id().as_u64(),
                file!(),
                line!()
            );
            println!(
                "[MAIN_BACKEND] [INFO] [THREAD {}] [FILE `{}` LINE {}] Now quitting...",
                std::thread::current().id().as_u64(),
                file!(),
                line!()
            );
            std::process::exit(0);
        });
    }
}

fn get_parent_path() -> String {
    // Get the parent path
    let mut pwd: std::path::PathBuf = std::env::current_dir().unwrap();
    pwd.pop();
    String::from(pwd.to_str().unwrap())
}

async fn parse_module_config_json() -> ModuleConfigJson {
    let mut module_config_json_string: String = String::new();
    let module_config_json_file_path: String = get_parent_path() + "/module_config_rs.json";
    let mut module_config_json_file: std::fs::File = match std::fs::File::open(
        &module_config_json_file_path,
    ) {
        Ok(file) => file,
        Err(e) => {
            eprintln!(
                "[MAIN_BACKEND] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Failed to open module_config_rs.json from `{}`. Maybe the file doesn't exist?",
                std::thread::current().id().as_u64(),
                file!(),
                line!(),
                module_config_json_file_path
            );
            eprintln!(
                "[MAIN_BACKEND] [ERROR] [THREAD {}] [FILE `{}` LINE {}] {}",
                e,
                std::thread::current().id().as_u64(),
                file!(),
                line!()
            );
            let main_backend_panic_flag: std::sync::Arc<tokio::sync::Mutex<bool>> =
                MAIN_BACKEND_PANIC_FLAG.clone();
            let mut guard_main_backend_panic_flag: tokio::sync::MutexGuard<'_, bool> =
                main_backend_panic_flag.lock().await;
            *guard_main_backend_panic_flag = true;
            drop(guard_main_backend_panic_flag);
            panic!();
        }
    };
    module_config_json_file
        .read_to_string(&mut module_config_json_string)
        .unwrap(); // Read module config json file to string.

    serde_json::from_str(module_config_json_string.as_str()).unwrap()
}

async fn load_modules(
    module_combinations: &mut Vec<ModuleCombination>,
    module_config_json: &ModuleConfigJson,
) {
    // Load modules
    for (name, config) in &module_config_json.working_load {
        if config.enabled {
            println!(
                "[MAIN_BACKEND] [INFO] [THREAD {}] [FILE `{}` LINE {}] Loading module {}...",
                std::thread::current().id().as_u64(),
                file!(),
                line!(),
                name
            );

            let module_library_file_path: String = get_parent_path()
                + "/rust_backend/modules/"
                + &config.id
                + "/lib"
                + &config.id
                + ".so"; // Get the path of the module.

            let module: Result<dlopen2::wrapper::Container<ModuleInstance>, dlopen2::Error> =
                unsafe { dlopen2::wrapper::Container::load(&module_library_file_path) }; // Load the module.

            match module {
                Ok(module) => {
                    println!(
                        "[MAIN_BACKEND] [INFO] [THREAD {}] [FILE `{}` LINE {}] Successfully loaded module `{}` from `{}`.",
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!(),
                        name,
                        &module_library_file_path
                    );

                    let result: (tokio::runtime::Runtime, AsyncModifiable<ModuleStatus>) =
                        module.on_init(&MAIN_TOKIO_RUNTIME);
                    {
                        let status: std::sync::Arc<tokio::sync::Mutex<ModuleStatus>> =
                            result.1.clone();
                        module_combinations.push(ModuleCombination {
                            name: String::from(name),
                            instance: module,
                            tokio_runtime: result.0,
                            status,
                        }); // Save the Tokio runtime.
                    }
                }
                Err(_) => {
                    eprintln!(
                        "[MAIN_BACKEND] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Failed to load module `{}` from `{}`. Maybe the module file doesn't exist or is not a valid shared object?",
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!(),
                        name,
                        &module_library_file_path
                    );

                    if module_config_json.restricted_mode {
                        eprintln!(
                            "[MAIN_BACKEND] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Due to the restricted mode, the server backend now is shutting down.",
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!()
                        );
                        panic!();
                    }
                }
            }
        }
    }
}

const FAKE_YIELD_NOW_MILLISECONDS: u64 = 1;

async fn fake_yield_now() {
    tokio::time::sleep(tokio::time::Duration::from_millis(
        FAKE_YIELD_NOW_MILLISECONDS,
    ))
    .await;
}
