use crate::global::*;
use chrono::{ DateTime, TimeDelta, Utc };

pub async fn self_management(simple_authenticator_status: AsyncModifiable<ModuleStatus>) {
    let mut time_last: DateTime<Utc> = Utc::now();
    loop {
        if let Ok(guard_simple_authenticator_status) = simple_authenticator_status.try_lock() {
            if
                guard_simple_authenticator_status.panicked ||
                (guard_simple_authenticator_status.initialized &&
                    SIMPLE_AUTHENTICATOR_SOCKET.get().is_none())
            {
                drop(guard_simple_authenticator_status); // Avoid poisoning the mutex lock.
                panic!();
            }
            drop(guard_simple_authenticator_status);
            let time_now = Utc::now();
            if time_last - time_now >= TimeDelta::minutes(1) {
                // Show monitoring message per minute.
                println!(
                    "{}",
                    ansi_term::Color::Blue.paint(
                        format!(
                            "[SIMPLE_AUTHENTICATOR] [INFO] [THREAD {}] [FILE `{}` LINE {}] Status reporting: Working very well.",
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!()
                        )
                    )
                );
                time_last = time_now;
            }
            fake_yield_now().await;
        } else {
            fake_yield_now().await;
        }
    }
}
