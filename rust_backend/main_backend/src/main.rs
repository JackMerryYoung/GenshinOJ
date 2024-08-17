use dlopen2::wrapper::WrapperApi;
use std::io::Read;

struct Holder {
    func: Box<dyn Fn() -> futures::future::BoxFuture<'static, ()>>
}

impl Holder {
    fn new<F>(f: fn() -> F) -> Holder where F: futures::future::Future<Output = ()> + Send + 'static {
        Holder {
            func: Box::new(move || Box::pin(f())),
        }
    }

    async fn run(&self) {
        (self.func)().await;
    }
}

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
    on_init: extern "Rust" fn() -> core::future::Future<Output = ()>,
    on_unload: extern "Rust" fn() -> core::future::Future<Output = ()>,
}

#[tokio::main]
async fn main() {
    let mut module_instances: Vec<dlopen2::wrapper::Container<ModuleInstance>> = vec![];
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
                "[MAIN_BACKEND] [Error] Failed to open module_config_rs.json from `{}`. Maybe the file doesn't exist?",
                module_config_json_file_path
            );
            eprintln!("[MAIN_BACKEND] [Error] {}", e);
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
            println!("[MAIN_BACKEND] [Info] Loading module {}...", name);
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
                        "[MAIN_BACKEND] [Info] Successfully loaded module `{}` from `{}`.",
                        name, &path
                    );
                    Holder::new(module.on_init).run().await;
                    module_instances.push(module);
                }
                Err(_e) => {
                    eprintln!(
                        "[MAIN_BACKEND] [Error] Failed to load module `{}` from `{}`. Maybe the module file doesn't exist or is not a valid shared object?",
                        name, &path
                    );
                    if module_config_json.restricted_mode {
                        panic!("[MAIN_BACKEND] [Error] Due to the restricted mode, the server backend now is shutting down.");
                    }
                }
            }
        }
    }
}
