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

// TODO: Use Rc for better performance
#[derive(serde::Deserialize, serde::Serialize)]
struct SingleModuleConfigJson {
    name: String,
    id: String,
    protocol: String,
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

fn new_async_modifiable<T>(x: T) -> AsyncModifiable<T> {
    std::sync::Arc::new(tokio::sync::Mutex::new(x))
}

#[derive(dlopen2::wrapper::WrapperApi)]
struct ModuleInstance {
    on_init: extern "Rust" fn(
        rt: &tokio::runtime::Runtime,
        global_module_statuses_by_protocol: AsyncModifiable<
            std::collections::HashMap<String, AsyncModifiable<ModuleStatus>>,
        >,
    ) -> (tokio::runtime::Runtime, AsyncModifiable<ModuleStatus>),
    on_unload: extern "Rust" fn(),
}

pub struct ModuleStatus {
    initialized: bool,
    panicked: bool,
    socket_port: AsyncModifiable<u16>,
}

struct ModuleCombination {
    name: String,
    instance: dlopen2::wrapper::Container<ModuleInstance>,
    tokio_runtime: tokio::runtime::Runtime,
}

fn main() {
    let module_combinations: AsyncModifiable<Vec<ModuleCombination>> = new_async_modifiable(vec![]);
    let module_statuses_by_protocol: AsyncModifiable<
        std::collections::HashMap<String, AsyncModifiable<ModuleStatus>>,
    > = new_async_modifiable(std::collections::HashMap::new());
    let module_config_json: AsyncModifiable<ModuleConfigJson> =
        MAIN_TOKIO_RUNTIME.block_on(async {
            new_async_modifiable(parse_module_config_json().await) // Parse from module config json string.
        });
    {
        let module_combinations: AsyncModifiable<Vec<ModuleCombination>> =
            module_combinations.clone();
        let module_statuses_by_protocol: AsyncModifiable<
            std::collections::HashMap<String, AsyncModifiable<ModuleStatus>>,
        > = module_statuses_by_protocol.clone();
        let module_config_json: AsyncModifiable<ModuleConfigJson> = module_config_json.clone();
        MAIN_TOKIO_RUNTIME.block_on(async move {
            let guard_module_config_json: tokio::sync::MutexGuard<'_, ModuleConfigJson> =
                module_config_json.lock().await;
            load_modules(
                module_combinations,
                module_statuses_by_protocol,
                &guard_module_config_json,
            )
            .await; // Load modules.
        });
    }

    {
        let module_combinations: AsyncModifiable<Vec<ModuleCombination>> =
            module_combinations.clone();
        let module_config_json: AsyncModifiable<ModuleConfigJson> = module_config_json.clone();
        let main_backend_panic_flag: AsyncModifiable<bool> = MAIN_BACKEND_PANIC_FLAG.clone();
        MAIN_TOKIO_RUNTIME.spawn(async move {
            loop {
                let guard_module_combinations: tokio::sync::MutexGuard<
                    '_,
                    Vec<ModuleCombination>
                > = module_combinations.lock().await;
                let guard_module_statuses: tokio::sync::MutexGuard<'_, std::collections::HashMap<String, AsyncModifiable<ModuleStatus>>> = module_statuses_by_protocol.lock().await;
                let guard_module_config_json: tokio::sync::MutexGuard<
                    '_,
                    ModuleConfigJson
                > = module_config_json.lock().await;
                for (index, status) in guard_module_statuses.iter().enumerate() {
                    let status: &AsyncModifiable<ModuleStatus> = status.1;
                    let guard_status: tokio::sync::MutexGuard<
                        '_,
                        ModuleStatus
                    > = status.lock().await;
                    if guard_status.panicked {
                        println!(
                            "{}", ansi_term::Color::Yellow.paint(
                                format!(
                                    "[MAIN_BACKEND] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Module {} has panicked.",
                                    std::thread::current().id().as_u64(),
                                    file!(),
                                    line!(),
                                    guard_module_combinations[index].name
                                )
                            )
                        );
                        if guard_module_config_json.restricted_mode {
                            eprintln!(
                                "{}", ansi_term::Color::Red.paint(
                                    format!(
                                        "[MAIN_BACKEND] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Due to the restricted mode, the server backend now is shutting down.",
                                        std::thread::current().id().as_u64(),
                                        file!(),
                                        line!()
                                    )
                                )
                            );
                            drop(guard_status);
                            drop(guard_module_config_json);
                            println!(
                                "{}", ansi_term::Color::Blue.paint(
                                    format!(
                                        "[MAIN_BACKEND] [INFO] [THREAD {}] [FILE `{}` LINE {}] Now quitting...",
                                        std::thread::current().id().as_u64(),
                                        file!(),
                                        line!()
                                    )
                                )
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
                "{}", ansi_term::Color::Blue.paint(
                    format!(
                        "[MAIN_BACKEND] [INFO] [THREAD {}] [FILE `{}` LINE {}] Successfully unloaded all the modules.",
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!()
                    )
                )
            );
            println!(
                "{}", ansi_term::Color::Blue.paint(
                    format!(
                        "[MAIN_BACKEND] [INFO] [THREAD {}] [FILE `{}` LINE {}] Now quitting...",
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!()
                    )
                )
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
                "{}", ansi_term::Color::Blue.paint(
                    format!(
                        "[MAIN_BACKEND] [INFO] [THREAD {}] [FILE `{}` LINE {}] Successfully unloaded all the modules.",
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!()
                    )
                )
            );
            println!(
                "{}", ansi_term::Color::Blue.paint(
                    format!(
                        "[MAIN_BACKEND] [INFO] [THREAD {}] [FILE `{}` LINE {}] Now quitting...",
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!()
                    )
                )
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

struct ForwardStarRepresentation {
    node_cnt: usize,
    edge_cnt: usize,
    edges: Vec<ForwardStarRepresentationEdge>,
    head: Vec<Option<usize>>,
}

struct ForwardStarRepresentationEdge {
    nxt: usize,
    to: usize,
}

impl ForwardStarRepresentation {
    fn new(node_cnt: usize) -> ForwardStarRepresentation {
        let mut head: Vec<Option<usize>> = Vec::with_capacity(node_cnt + 1);
        for _ in 0..=node_cnt {
            head.push(None);
        }
        let mut edges: Vec<ForwardStarRepresentationEdge> = Vec::with_capacity(node_cnt);
        edges.push(ForwardStarRepresentationEdge { nxt: 0, to: 0 });
        ForwardStarRepresentation {
            node_cnt,
            edge_cnt: 0,
            edges,
            head,
        }
    }
    fn add_edge(&mut self, from: usize, to: usize) {
        self.edge_cnt += 1;
        self.edges.push(ForwardStarRepresentationEdge {
            nxt: self.head[from].unwrap_or(0),
            to,
        });
        self.head[from] = Some(self.edge_cnt);
    }
}

fn check_protocol_version_requirements_satisfied(
    required_version: &String,
    given_version: &String,
) -> bool {
    required_version == given_version
}

async fn load_modules(
    module_combinations: AsyncModifiable<Vec<ModuleCombination>>,
    module_statuses_by_protocol: AsyncModifiable<
        std::collections::HashMap<String, AsyncModifiable<ModuleStatus>>,
    >,
    module_config_json: &ModuleConfigJson,
) {
    // Giving ID to each module.
    let mut ids_by_module_protocol: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    let mut modules_protocol_version: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    let mut modules_name_by_id: std::collections::HashMap<usize, String> =
        std::collections::HashMap::new();
    let mut modules_protocol_by_id: std::collections::HashMap<usize, String> =
        std::collections::HashMap::new();
    let mut id_cnt: usize = 0;
    for config in module_config_json.working_load.values() {
        if config.enabled {
            id_cnt += 1;
            let pos: usize = config.protocol.find('@').unwrap();
            let module_protocol_without_version: String = String::from(&config.protocol[..pos]); // TODO: Use Rc for better performance
            if ids_by_module_protocol
                .insert(module_protocol_without_version.clone(), id_cnt)
                .is_some()
            {
                eprintln!(
                    "{}", ansi_term::Color::Red.paint(
                        format!(
                            "[MAIN_BACKEND] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Encountered different modules implemented the same protocol {}.",
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!(),
                            &module_protocol_without_version
                        )
                    )
                );
                panic!();
            }

            let module_protocol_with_only_version: String =
                String::from(&config.protocol[pos + 1..]); // TODO: Use Rc for better performance
            if modules_protocol_version
                .insert(
                    module_protocol_without_version.clone(),
                    module_protocol_with_only_version,
                )
                .is_some()
            {
                eprintln!(
                    "{}", ansi_term::Color::Red.paint(
                        format!(
                            "[MAIN_BACKEND] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Encountered different modules implemented the same protocol {}.",
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!(),
                            &module_protocol_without_version
                        )
                    )
                );
                panic!();
            }

            if modules_name_by_id
                .insert(id_cnt, config.id.clone())
                .is_some()
            {
                eprintln!(
                    "{}", ansi_term::Color::Red.paint(
                        format!(
                            "[MAIN_BACKEND] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Encountered different modules implemented the same protocol {}.",
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!(),
                            &module_protocol_without_version
                        )
                    )
                );
                panic!();
            };

            if modules_protocol_by_id
                .insert(id_cnt, module_protocol_without_version.clone())
                .is_some()
            {
                eprintln!(
                    "{}", ansi_term::Color::Red.paint(
                        format!(
                            "[MAIN_BACKEND] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Encountered different modules implemented the same protocol {}.",
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!(),
                            &module_protocol_without_version
                        )
                    )
                );
                panic!();
            };
        }
    }
    // Build dependencies DAG.
    let mut forward_star_representation: ForwardStarRepresentation =
        ForwardStarRepresentation::new(id_cnt);
    let mut indegs: Vec<usize> = vec![0; id_cnt + 1];
    for config in module_config_json.working_load.values() {
        if config.enabled {
            for dependencies_protocol in &config.dependencies {
                let pos: usize = dependencies_protocol.find('@').unwrap();
                let dependencies_protocol_without_version: String =
                    String::from(&dependencies_protocol[..pos]);
                if check_protocol_version_requirements_satisfied(
                    &String::from(&dependencies_protocol[pos + 1..]),
                    modules_protocol_version
                        .get(&dependencies_protocol_without_version)
                        .unwrap_or_else(||{
                            eprintln!(
                                "{}", ansi_term::Color::Red.paint(
                                    format!(
                                        "[MAIN_BACKEND] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Protocol requirements are not satisfied.",
                                        std::thread::current().id().as_u64(),
                                        file!(),
                                        line!()
                                    )
                                )
                            );
                            eprintln!(
                                "{}", ansi_term::Color::Red.paint(
                                    format!(
                                        "[MAIN_BACKEND] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Requires protocol `{}`.",
                                        std::thread::current().id().as_u64(),
                                        file!(),
                                        line!(),
                                        &dependencies_protocol_without_version
                                    )
                                )
                            );
                            panic!();
                        }),
                ) {
                    let result: Option<&usize> =
                    ids_by_module_protocol.get(&dependencies_protocol_without_version);
                    if let Some(id_by_module_protocol) = result {
                        let pos: usize = config.protocol.find('@').unwrap();
                        let protocol_without_version: String =
                            String::from(&config.protocol[..pos]);
                        let to: usize = *(ids_by_module_protocol
                                .get(&protocol_without_version)
                                .unwrap());
                        forward_star_representation.add_edge(
                            *id_by_module_protocol,
                            to,
                        );
                        indegs[to] += 1;
                    } else {
                        panic!();
                    }
                }
            }
        }
    }
    // Load modules.
    let mut queue: std::collections::VecDeque<usize> = std::collections::VecDeque::new();
    for (index, value) in indegs.iter().enumerate().take(id_cnt + 1).skip(1) {
        if *value == 0 {
            queue.push_back(index);
        }
    }
    while !queue.is_empty() {
        let head_ele: usize = queue.pop_front().unwrap();
        let module_name: &String = modules_name_by_id.get(&head_ele).unwrap();
        println!(
            "{}",
            ansi_term::Color::Blue.paint(format!(
                "[MAIN_BACKEND] [INFO] [THREAD {}] [FILE `{}` LINE {}] Loading module {}...",
                std::thread::current().id().as_u64(),
                file!(),
                line!(),
                module_name
            ))
        );

        let module_config = module_config_json.working_load.get(module_name).unwrap();
        let module_library_file_path: String = get_parent_path()
            + "/rust_backend/modules/"
            + &module_config.id
            + "/lib"
            + &module_config.id
            + ".so"; // Get the path of the module.

        let module: Result<dlopen2::wrapper::Container<ModuleInstance>, dlopen2::Error> =
            unsafe { dlopen2::wrapper::Container::load(&module_library_file_path) }; // Load the module.

        match module {
            Ok(module) => {
                println!(
                    "{}", ansi_term::Color::Blue.paint(
                        format!(
                            "[MAIN_BACKEND] [INFO] [THREAD {}] [FILE `{}` LINE {}] Successfully loaded module `{}` from `{}`.",
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!(),
                            module_name,
                            &module_library_file_path
                        )
                    )
                );

                let result: (tokio::runtime::Runtime, AsyncModifiable<ModuleStatus>) =
                    module.on_init(&MAIN_TOKIO_RUNTIME, module_statuses_by_protocol.clone());
                {
                    let status: AsyncModifiable<ModuleStatus> = result.1.clone();
                    loop {
                        // Waiting for the initialization to be completed.
                        let guard_status: tokio::sync::MutexGuard<'_, ModuleStatus> =
                            status.lock().await;
                        if guard_status.initialized {
                            drop(guard_status);
                            break;
                        }
                        drop(guard_status);
                        fake_yield_now().await;
                    }
                    let mut guard_module_combinations: tokio::sync::MutexGuard<
                        '_,
                        Vec<ModuleCombination>,
                    > = module_combinations.lock().await;
                    guard_module_combinations.push(ModuleCombination {
                        name: String::from(module_name),
                        instance: module,
                        tokio_runtime: result.0,
                    }); // Save the Tokio runtime.
                    drop(guard_module_combinations);
                    let mut guard_module_statuses_by_protocol: tokio::sync::MutexGuard<
                        '_,
                        std::collections::HashMap<String, AsyncModifiable<ModuleStatus>>,
                    > = module_statuses_by_protocol.lock().await;
                    guard_module_statuses_by_protocol.insert(
                        String::from(modules_protocol_by_id.get(&head_ele).unwrap()),
                        status,
                    );
                }

                if let Some(mut tmp) = forward_star_representation.head[head_ele] {
                    loop {
                        let v: usize = forward_star_representation.edges[tmp].to;
                        indegs[v] -= 1;
                        if indegs[v] == 0 {
                            queue.push_back(v);
                        }
                        tmp = forward_star_representation.edges[tmp].nxt;
                        if tmp == 0 {
                            break;
                        }
                    }
                }
            }
            Err(_) => {
                eprintln!(
                    "{}", ansi_term::Color::Red.paint(
                        format!(
                            "[MAIN_BACKEND] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Failed to load module `{}` from `{}`. Maybe the module file doesn't exist or is not a valid shared object?",
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!(),
                            module_name,
                            &module_library_file_path
                        )
                    )
                );

                if module_config_json.restricted_mode {
                    eprintln!(
                        "{}", ansi_term::Color::Red.paint(
                            format!(
                                "[MAIN_BACKEND] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Due to the restricted mode, the server backend now is shutting down.",
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!()
                            )
                        )
                    );
                    // TODO: Calling all the other modules to shutdown.
                    panic!();
                }
            }
        }
    }
}

const FAKE_YIELD_NOW_MILLISECONDS: u64 = 100;

async fn fake_yield_now() {
    tokio::time::sleep(tokio::time::Duration::from_millis(
        FAKE_YIELD_NOW_MILLISECONDS,
    ))
    .await;
}
