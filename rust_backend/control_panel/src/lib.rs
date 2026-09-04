#![feature(thread_id_value)]

mod global;
mod analytics;
mod db_admin;
mod http_server;
mod self_management;
mod system_monitor;
mod user_management;
mod problem_management;
mod panel_html;

use crate::global::*;

#[unsafe(no_mangle)]
pub extern "Rust" fn on_init(
    // Accepted to match the dylib ABI main_backend loads every module through. The control panel
    // has no inter-module messaging, so it doesn't register itself in this map.
    _global_module_statuses_by_protocol: AsyncModifiable<
        std::collections::HashMap<String, AsyncModifiable<ModuleStatus>>
    >
) -> (tokio::runtime::Runtime, AsyncModifiable<ModuleStatus>) {
    let control_panel_runtime: tokio::runtime::Runtime = tokio::runtime::Builder
        ::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    MODULE_RUNTIME_HANDLE.set(control_panel_runtime.handle().clone()).unwrap();
    let control_panel_status: ModuleStatus = ModuleStatus {
        initialized: false,
        panicked: false,
        socket_port: new_async_modifiable(0),
        init_notify: std::sync::Arc::new(tokio::sync::Notify::new()),
    };
    let control_panel_status: AsyncModifiable<ModuleStatus> =
        new_async_modifiable(control_panel_status);

    {
        let control_panel_status: AsyncModifiable<ModuleStatus> = control_panel_status.clone();
        control_panel_runtime.spawn(async move {
            crate::http_server::serve(control_panel_status).await;
        });
    }

    {
        let control_panel_status: AsyncModifiable<ModuleStatus> = control_panel_status.clone();
        control_panel_runtime.spawn(crate::self_management::self_management(control_panel_status));
    }

    (control_panel_runtime, control_panel_status)
}

#[unsafe(no_mangle)]
pub extern "Rust" fn on_unload(unload_timeout_ms: usize) {
    println!(
        "{}",
        ansi_term::Color::Purple.paint(
            format!(
                "[{}] [DOWN] [THREAD {}] [FILE `{}` LINE {}] Unloading the control panel...",
                MODULE_IDENTITY,
                std::thread::current().id().as_u64(),
                file!(),
                line!()
            )
        )
    );
    let cleanup_completed = run_shutdown_task(async {
        disconnect_database_pool().await;
    }, std::time::Duration::from_millis(unload_timeout_ms as u64));
    println!(
        "{}",
        ansi_term::Color::Purple.paint(
            format!(
                "[{}] [DOWN] [THREAD {}] [FILE `{}` LINE {}] Control panel shutdown cleanup {}.",
                MODULE_IDENTITY,
                std::thread::current().id().as_u64(),
                file!(),
                line!(),
                if cleanup_completed { "completed" } else { "timed out" }
            )
        )
    );
}
