#![feature(thread_id_value)]

mod global;
mod socket_actions;
mod self_management;
mod judge_socket;
mod judging;
mod solutions;
mod discussions;
mod notifications;

use crate::global::*;

use mysql_async::prelude::*;

#[unsafe(no_mangle)]
pub extern "Rust" fn on_init(
    global_module_statuses_by_protocol: AsyncModifiable<
        std::collections::HashMap<String, AsyncModifiable<ModuleStatus>>
    >
) -> (tokio::runtime::Runtime, AsyncModifiable<ModuleStatus>) {
    let judge_runtime: tokio::runtime::Runtime = tokio::runtime::Builder
        ::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    MODULE_RUNTIME_HANDLE.set(judge_runtime.handle().clone()).unwrap();
    GLOBAL_MODULE_STATUSES_BY_PROTOCOL.set(global_module_statuses_by_protocol.clone()).unwrap();
    let judge_status: ModuleStatus = ModuleStatus {
        initialized: false,
        panicked: false,
        socket_port: new_async_modifiable(0),
        init_notify: std::sync::Arc::new(tokio::sync::Notify::new()),
    };
    let judge_status: AsyncModifiable<ModuleStatus> = new_async_modifiable(judge_status);

    {
        let judge_status: AsyncModifiable<ModuleStatus> = judge_status.clone();
        judge_runtime.spawn(async move {
            let mut conn: mysql_async::Conn = MYSQL_DATABASE_POOL.get_conn().await.unwrap();
            "CREATE DATABASE IF NOT EXISTS RsOJ".ignore(&mut conn).await.unwrap();
            "USE RsOJ".ignore(&mut conn).await.unwrap();
            "CREATE TABLE IF NOT EXISTS submissions (
                submission_id INT AUTO_INCREMENT PRIMARY KEY NOT NULL,
                username VARCHAR(256) NOT NULL,
                problem_number INT NOT NULL,
                result VARCHAR(16) NOT NULL DEFAULT 'PD',
                general_score INT NOT NULL DEFAULT 0,
                statuses TEXT NOT NULL,
                scores TEXT NOT NULL,
                code TEXT NOT NULL,
                language VARCHAR(16) NOT NULL DEFAULT '',
                is_test_submission_mode BOOLEAN NOT NULL DEFAULT FALSE,
                created_at BIGINT NOT NULL
            )"
                .ignore(&mut conn).await
                .unwrap();
            // User-authored (and, later, official/starred) write-ups for a problem. `content` is a
            // JSON array of markdown lines, mirroring how `submissions.code` and problem statements
            // are stored. `likes`/`dislikes` are denormalized running totals kept in sync with the
            // per-user rows in `solution_votes`.
            "CREATE TABLE IF NOT EXISTS solutions (
                solution_id INT AUTO_INCREMENT PRIMARY KEY NOT NULL,
                problem_number INT NOT NULL,
                username VARCHAR(256) NOT NULL,
                title VARCHAR(512) NOT NULL,
                content TEXT NOT NULL,
                is_official BOOLEAN NOT NULL DEFAULT FALSE,
                likes INT NOT NULL DEFAULT 0,
                dislikes INT NOT NULL DEFAULT 0,
                created_at BIGINT NOT NULL
            )"
                .ignore(&mut conn).await
                .unwrap();
            // One row per (user, solution); `vote` is 1 for a like and -1 for a dislike. The
            // composite primary key enforces a single vote per user per solution.
            "CREATE TABLE IF NOT EXISTS solution_votes (
                username VARCHAR(256) NOT NULL,
                solution_id INT NOT NULL,
                vote TINYINT NOT NULL,
                PRIMARY KEY (username, solution_id)
            )"
                .ignore(&mut conn).await
                .unwrap();
            "CREATE TABLE IF NOT EXISTS solution_comments (
                comment_id INT AUTO_INCREMENT PRIMARY KEY NOT NULL,
                solution_id INT NOT NULL,
                username VARCHAR(256) NOT NULL,
                content TEXT NOT NULL,
                likes INT NOT NULL DEFAULT 0,
                dislikes INT NOT NULL DEFAULT 0,
                created_at BIGINT NOT NULL
            )"
                .ignore(&mut conn).await
                .unwrap();
            "CREATE TABLE IF NOT EXISTS solution_comment_votes (
                username VARCHAR(256) NOT NULL,
                comment_id INT NOT NULL,
                vote TINYINT NOT NULL,
                PRIMARY KEY (username, comment_id)
            )"
                .ignore(&mut conn).await
                .unwrap();
            // Global discussion forum (not tied to any problem): user-authored threads with a reply
            // area. Mirrors the solutions tables — `content` is a JSON array of markdown lines, and
            // `likes`/`dislikes` are running totals kept in sync with the per-user vote rows.
            "CREATE TABLE IF NOT EXISTS discussions (
                discussion_id INT AUTO_INCREMENT PRIMARY KEY NOT NULL,
                username VARCHAR(256) NOT NULL,
                title VARCHAR(512) NOT NULL,
                content TEXT NOT NULL,
                likes INT NOT NULL DEFAULT 0,
                dislikes INT NOT NULL DEFAULT 0,
                created_at BIGINT NOT NULL
            )"
                .ignore(&mut conn).await
                .unwrap();
            "CREATE TABLE IF NOT EXISTS discussion_votes (
                username VARCHAR(256) NOT NULL,
                discussion_id INT NOT NULL,
                vote TINYINT NOT NULL,
                PRIMARY KEY (username, discussion_id)
            )"
                .ignore(&mut conn).await
                .unwrap();
            "CREATE TABLE IF NOT EXISTS discussion_replies (
                reply_id INT AUTO_INCREMENT PRIMARY KEY NOT NULL,
                discussion_id INT NOT NULL,
                username VARCHAR(256) NOT NULL,
                content TEXT NOT NULL,
                likes INT NOT NULL DEFAULT 0,
                dislikes INT NOT NULL DEFAULT 0,
                created_at BIGINT NOT NULL
            )"
                .ignore(&mut conn).await
                .unwrap();
            "CREATE TABLE IF NOT EXISTS discussion_reply_votes (
                username VARCHAR(256) NOT NULL,
                reply_id INT NOT NULL,
                vote TINYINT NOT NULL,
                PRIMARY KEY (username, reply_id)
            )"
                .ignore(&mut conn).await
                .unwrap();
            // Per-user "info center" inbox. One row per delivered notification. `kind` is 'mention'
            // (the `actor` wrote @recipient) or 'reply' (the `actor` commented on / replied to the
            // recipient's solution or discussion). `target_url` is the frontend path to open when the
            // notification is clicked; `excerpt` is a short preview of the triggering text.
            "CREATE TABLE IF NOT EXISTS notifications (
                notification_id INT AUTO_INCREMENT PRIMARY KEY NOT NULL,
                recipient VARCHAR(256) NOT NULL,
                actor VARCHAR(256) NOT NULL,
                kind VARCHAR(16) NOT NULL,
                source_type VARCHAR(32) NOT NULL,
                target_url VARCHAR(512) NOT NULL,
                excerpt VARCHAR(512) NOT NULL,
                is_read BOOLEAN NOT NULL DEFAULT FALSE,
                created_at BIGINT NOT NULL,
                INDEX idx_recipient_created (recipient, created_at)
            )"
                .ignore(&mut conn).await
                .unwrap();
            // Cover the list/count paths and common problem/user filters. These are best-effort
            // migrations so an existing index does not prevent startup.
            for statement in [
                "ALTER TABLE submissions ADD INDEX idx_submissions_problem (problem_number, submission_id)",
                "ALTER TABLE submissions ADD INDEX idx_submissions_user (username, submission_id)",
                "ALTER TABLE solutions ADD INDEX idx_solutions_problem_created (problem_number, is_official, created_at, solution_id)",
                "ALTER TABLE solution_comments ADD INDEX idx_solution_comments_solution_created (solution_id, created_at, comment_id)",
                "ALTER TABLE discussions ADD INDEX idx_discussions_created (created_at, discussion_id)",
                "ALTER TABLE discussion_replies ADD INDEX idx_discussion_replies_discussion_created (discussion_id, created_at, reply_id)",
                "ALTER TABLE notifications ADD INDEX idx_notifications_recipient_id (recipient, notification_id)",
                "ALTER TABLE notifications ADD INDEX idx_notifications_recipient_read (recipient, is_read, notification_id)",
            ] {
                let _ = statement.ignore(&mut conn).await;
            }
            drop(conn);

            let mut guard_judge_status: tokio::sync::MutexGuard<
                '_,
                ModuleStatus
            > = judge_status.lock().await; // Get the status of the server.
            // Try to establish a socket for messaging.
            let mut judge_socket_port: u16 = 9003;
            let mut guard_judge_status_socket_port: tokio::sync::MutexGuard<
                '_,
                u16
            > = guard_judge_status.socket_port.lock().await;
            let judge_socket: tokio::net::TcpListener;
            (judge_socket, *guard_judge_status_socket_port) = loop {
                let judge_socket_result: Result<
                    tokio::net::TcpListener,
                    std::io::Error
                > = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", judge_socket_port)).await;
                if let Ok(x) = judge_socket_result {
                    println!(
                        "{}",
                        ansi_term::Color::Green.paint(
                            format!(
                                "[{}] [INFO] [THREAD {}] [FILE `{}` LINE {}] Initialized the socket on port {}.",
                                MODULE_IDENTITY,
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!(),
                                judge_socket_port
                            )
                        )
                    );
                    break (x, judge_socket_port);
                } else {
                    println!(
                        "{}",
                        ansi_term::Color::Yellow.paint(
                            format!(
                                "[{}] [WARNING] [THREAD {}] [FILE `{}` LINE {}] Failed to open the socket on port {}. Retrying...",
                                MODULE_IDENTITY,
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!(),
                                judge_socket_port
                            )
                        )
                    );
                }
                if judge_socket_port == u16::MAX {
                    eprintln!(
                        "{}",
                        ansi_term::Color::Red.paint(
                            format!(
                                "[{}] [ERROR] [THREAD {}] [FILE `{}` LINE {}] Exceeded maximum retry times. Now quitting... ",
                                MODULE_IDENTITY,
                                std::thread::current().id().as_u64(),
                                file!(),
                                line!()
                            )
                        )
                    );
                    panic!();
                }
                judge_socket_port += 1;
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            };
            JUDGE_SOCKET.set(new_async_modifiable(judge_socket)).unwrap();
            drop(guard_judge_status_socket_port);
            guard_judge_status.initialized = true;
            // notify_waiters(), not notify_one(): there are two independent waiters on this same
            // init_notify — main_backend's module-loading wait, and this module's own internal
            // wait below before it starts processing its socket. notify_one() only wakes one of
            // them, permanently starving the other.
            guard_judge_status.init_notify.notify_waiters();
            drop(guard_judge_status);
        });
    }

    {
        let judge_status: AsyncModifiable<ModuleStatus> = judge_status.clone();
        judge_runtime.spawn(async move {
            // Wait for initialization, notified instead of polled. The setter uses
            // notify_waiters() (not notify_one()) because main_backend's own module-loading wait
            // is a second, independent waiter on this same init_notify; notify_waiters() only
            // reaches waiters already registered at the moment it's called, so `enable()` must
            // run here before the flag check to register us immediately.
            let init_notify: std::sync::Arc<tokio::sync::Notify> = judge_status
                .lock().await.init_notify
                .clone();
            let notified = init_notify.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            let already_initialized: bool = judge_status.lock().await.initialized;
            if !already_initialized {
                notified.await;
            }

            // Now processing socket message
            judge_socket::socket_message_processing().await;
        });
    }

    {
        judge_runtime.spawn(async move {
            // ws_server doesn't necessarily exist in the map yet (it may not have been loaded by
            // main_backend at all), so there's no event to wait on for that — poll until it
            // registers itself.
            let ws_server_status: AsyncModifiable<ModuleStatus> = loop {
                let guard_global_module_statuses_by_protocol =
                    global_module_statuses_by_protocol.lock().await;
                if
                    let Some(ws_server_status) =
                        guard_global_module_statuses_by_protocol.get("std_ws_server")
                {
                    let ws_server_status: AsyncModifiable<ModuleStatus> = ws_server_status.clone();
                    drop(guard_global_module_statuses_by_protocol);
                    break ws_server_status;
                }
                drop(guard_global_module_statuses_by_protocol);
                println!(
                    "{}",
                    ansi_term::Color::Blue.paint(
                        format!(
                            "[{}] [INFO] [THREAD {}] [FILE `{}` LINE {}] Waiting for the Websocket server to be initialized...",
                            MODULE_IDENTITY,
                            std::thread::current().id().as_u64(),
                            file!(),
                            line!()
                        )
                    )
                );
                tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
            };
            // Now that ws_server is registered, wait for it to finish initializing — notified
            // instead of polled, same as init_notify everywhere else. In practice ws_server only
            // appears in the map after main_backend's own wait for it already completed, so
            // `already_initialized` will already be true here; `enable()` is kept for
            // consistency with the other waiters in case that ordering ever changes.
            let init_notify: std::sync::Arc<tokio::sync::Notify> = ws_server_status
                .lock().await.init_notify
                .clone();
            let notified = init_notify.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            let already_initialized: bool = ws_server_status.lock().await.initialized;
            if !already_initialized {
                notified.await;
            }
            println!(
                "{}",
                ansi_term::Color::Green.paint(
                    format!(
                        "[{}] [INFO] [THREAD {}] [FILE `{}` LINE {}] The Websocket server has been initialized.",
                        MODULE_IDENTITY,
                        std::thread::current().id().as_u64(),
                        file!(),
                        line!()
                    )
                )
            );

            judge_socket::connect_to_ws_server().await;
        });
    }

    {
        let judge_status: AsyncModifiable<ModuleStatus> = judge_status.clone();
        judge_runtime.spawn(self_management::self_management(judge_status));
    }
    (judge_runtime, judge_status)
}

#[unsafe(no_mangle)]
pub extern "Rust" fn on_unload(unload_timeout_ms: usize) {
    println!(
        "{}",
        ansi_term::Color::Purple.paint(
            format!(
                "[{}] [DOWN] [THREAD {}] [FILE `{}` LINE {}] Unloading the judge...",
                MODULE_IDENTITY,
                std::thread::current().id().as_u64(),
                file!(),
                line!()
            )
        )
    );

    let cleanup_completed = run_shutdown_task(async move {
        // Resolve whether ws_server is up and release the lock before calling
        // disconnect_from_ws_server() below, since it locks GLOBAL_MODULE_STATUSES_BY_PROTOCOL
        // itself again internally (via get_socket_port_by_protocol) — holding it here too would
        // self-deadlock the task on tokio::sync::Mutex, which isn't reentrant.
        let ws_server_initialized: bool = {
            let guard_global_module_statuses_by_protocol = GLOBAL_MODULE_STATUSES_BY_PROTOCOL.get()
                .unwrap()
                .lock().await;
            match guard_global_module_statuses_by_protocol.get("std_ws_server") {
                Some(ws_server_status) => ws_server_status.lock().await.initialized,
                None => false,
            }
        };
        if ws_server_initialized {
            judge_socket::disconnect_from_ws_server().await;
        }
        disconnect_database_pool().await;
    }, std::time::Duration::from_millis(unload_timeout_ms as u64));

    println!(
        "{}",
        ansi_term::Color::Purple.paint(
            format!(
                "[{}] [DOWN] [THREAD {}] [FILE `{}` LINE {}] Judge shutdown cleanup {}.",
                MODULE_IDENTITY,
                std::thread::current().id().as_u64(),
                file!(),
                line!(),
                if cleanup_completed { "completed" } else { "timed out" }
            )
        )
    );
}
