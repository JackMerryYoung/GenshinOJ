use crate::global::*;
use chrono::{ DateTime, TimeDelta, Utc };

pub async fn self_management(ws_server_status: AsyncModifiable<ModuleStatus>) {
    let mut time_last: DateTime<Utc> = Utc::now();
    loop {
        if let Ok(guard_ws_server_status) = ws_server_status.try_lock() {
            if
                guard_ws_server_status.panicked ||
                (guard_ws_server_status.initialized && WS_SERVER_SOCKET.get().is_none())
            {
                drop(guard_ws_server_status); // Avoid poisoning the mutex lock.
                panic!();
            }
            drop(guard_ws_server_status);
            let time_now = Utc::now();
            if time_now - time_last >= TimeDelta::minutes(1) {
                // Show monitoring message per minute.
                let guard_ws_server_connections_cnt: tokio::sync::MutexGuard<
                    '_,
                    usize
                > = WS_SERVER_CONNECTIONS_CNT.lock().await;
                println!(
                    "{}",
                    ansi_term::Color::Blue.paint(
                        format!(
                            "[{}] [INFO] [THREAD {}] [FILE `{}` LINE {}] Status reporting: Working very well with {} connection(s) in total.",
                            MODULE_IDENTITY,
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!(),
                            *guard_ws_server_connections_cnt
                        )
                    )
                );
                time_last = time_now;
            }
            fake_yield_now(200).await;
        } else {
            fake_yield_now(200).await;
        }
    }
}
