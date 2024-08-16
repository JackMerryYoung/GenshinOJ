use dlopen2::wrapper::WrapperApi;
use std::io::Read;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct SingleModuleConfigJson {
    name: String,
    id: String,
    enabled: bool,
    path: String,
    dependencies: Vec<String>,
    unload_timeout: usize,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct ModuleConfigJson {
    working_load: std::collections::HashMap<String, SingleModuleConfigJson>,
}

#[derive(dlopen2::wrapper::WrapperApi)]
struct ModuleInstance {
    on_init: fn(),
    on_unload: fn(),
}

fn main() {
    let mut module_instances: Vec<dlopen2::wrapper::Container<ModuleInstance>> = vec![];
    let mut tmp: std::path::PathBuf = std::env::current_dir().unwrap();
    tmp.pop();
    let module_config_json_file_path: String =
        String::from(tmp.to_str().unwrap()) + "/module_config_rs.json";
    let mut module_config_json_file: std::fs::File =
        match std::fs::File::open(&module_config_json_file_path) {
            Ok(file) => file,
            Err(e) => {
                eprintln!(
                "[Error] Failed to open module_config.json(`{}`). Maybe the file doesn't exist?",
                module_config_json_file_path
            );
                eprintln!("[Error] {}", e);
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
            println!("[Info] Loading module {}...", name);
            let module: Result<dlopen2::wrapper::Container<ModuleInstance>, dlopen2::Error> =
                unsafe { dlopen2::wrapper::Container::load(&config.path) };
            match module {
                Ok(module) => {
                    println!("[Info] Successfully loaded module {}.", name);
                    module_instances.push(module);
                }
                Err(e) => {
                    eprintln!(
                        "[Error] Failed to load module `{}`. Maybe the module file doesn't exist or is not a valid shared object?",
                        &config.path
                    );
                }
            }
        }
    }
}
