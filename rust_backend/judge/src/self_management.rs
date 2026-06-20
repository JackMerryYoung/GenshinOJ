use crate::global::*;
use chrono::{ DateTime, TimeDelta, Utc };

pub async fn self_management(judge_status: AsyncModifiable<ModuleStatus>) {
    let mut time_last: DateTime<Utc> = Utc::now();
    loop {
        let guard_judge_status = judge_status.lock().await;
        if
            guard_judge_status.panicked ||
            (guard_judge_status.initialized && JUDGE_SOCKET.get().is_none())
        {
            drop(guard_judge_status); // Avoid poisoning the mutex lock.
            panic!();
        }
        drop(guard_judge_status);
        let time_now = Utc::now();
        if time_now - time_last >= TimeDelta::minutes(1) {
            // Show monitoring message per minute.
            println!(
                "{}",
                ansi_term::Color::Blue.paint(
                    format!(
                        "[{}] [INFO] [THREAD {}] [FILE `{}` LINE {}] Status reporting: Working very well.",
                        MODULE_IDENTITY,
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!()
                    )
                )
            );
            time_last = time_now;
        }
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    }
}
