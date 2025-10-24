use crate::global::*;

pub async fn self_management(ws_server_status: AsyncModifiable<ModuleStatus>) {
    let mut monitor_time_cnt: usize = 0;
    loop {
        if let Ok(guard_ws_server_status) = ws_server_status.try_lock() {
            if guard_ws_server_status.panicked || (guard_ws_server_status.initialized && WS_SERVER_SOCKET.get().is_none()) {
                drop(guard_ws_server_status); // Avoid poisoning the mutex lock.
                panic!();
            }
            drop(guard_ws_server_status);
            monitor_time_cnt += 1;
            if monitor_time_cnt == 600 {
                // Show monitoring message per minute.
                let guard_ws_server_connections_cnt: tokio::sync::MutexGuard<'_, usize> =
                    WS_SERVER_CONNECTIONS_CNT.lock().await;
                println!(
                    "{}", ansi_term::Color::Blue.paint(
                        format!(
                            "[WS_SERVER] [INFO] [THREAD {}] [FILE `{}` LINE {}] Status reporting: Working very well with {} connections in total.",
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!(),
                            *guard_ws_server_connections_cnt
                        )
                    )
                );
                monitor_time_cnt = 0;
            }
            fake_yield_now().await;
        } else {
            fake_yield_now().await;
        }
    }
}