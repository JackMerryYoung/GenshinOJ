use crate::global::*;

use axum::extract::{ Multipart, Path };
use axum::http::{ StatusCode, header };
use axum::response::{ IntoResponse, Response };

fn is_valid_username(username: &str) -> bool {
    !username.is_empty() &&
        username.len() <= 256 &&
        username.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn sniff_image_extension(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(&[0x89, 0x50, 0x4e, 0x47]) {
        return Some("png");
    }
    if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        return Some("jpg");
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Some("gif");
    }
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Some("webp");
    }
    None
}

fn content_type_for_extension(extension: &str) -> &'static str {
    match extension {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => "application/octet-stream",
    }
}

#[derive(serde::Serialize)]
struct ContentOnValidateSession {
    username: String,
    session_token: String,
    request_key: String,
}

// Asks simple_authenticator to validate the session, via the same `on_validate_session` RPC
// `judge` uses for submissions — replies are routed back here because `on_validate_session`
// replies to `msg.from_protocol`, which we set to our own protocol below.
async fn validate_session(username: &str, session_token: &str) -> bool {
    let request_key: String = uuid::Uuid::new_v4().to_string();
    let (tx, rx) = tokio::sync::oneshot::channel::<bool>();
    {
        let mut guard_pending_validate_session_requests = PENDING_VALIDATE_SESSION_REQUESTS.lock().await;
        guard_pending_validate_session_requests.insert(request_key.clone(), tx);
    }

    let msg_to_send: SocketJsonMessage = SocketJsonMessage {
        r#type: String::from("on_validate_session"),
        content: serde_json
            ::to_value(ContentOnValidateSession {
                username: username.to_string(),
                session_token: session_token.to_string(),
                request_key: request_key.clone(),
            })
            .unwrap(),
        request_key: uuid::Uuid::new_v4().to_string(),
        from_protocol: String::from("std_ws_server"),
    };
    send_socket_json_message(
        &serde_json::to_value(msg_to_send).unwrap(),
        &String::from("std_authenticator")
    ).await;

    match tokio::time::timeout(std::time::Duration::from_secs(5), rx).await {
        Ok(Ok(session_valid)) => session_valid,
        _ => {
            let mut guard_pending_validate_session_requests = PENDING_VALIDATE_SESSION_REQUESTS.lock().await;
            guard_pending_validate_session_requests.remove(&request_key);
            false
        }
    }
}

pub async fn upload_avatar(mut multipart: Multipart) -> Response {
    let mut username: Option<String> = None;
    let mut session_token: Option<String> = None;
    let mut file_bytes: Option<bytes::Bytes> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        match field.name().unwrap_or("") {
            "username" => {
                username = field.text().await.ok();
            }
            "session_token" => {
                session_token = field.text().await.ok();
            }
            "file" => {
                file_bytes = field.bytes().await.ok();
            }
            _ => {}
        }
    }

    let (Some(username), Some(session_token), Some(file_bytes)) = (
        username,
        session_token,
        file_bytes,
    ) else {
        return (StatusCode::BAD_REQUEST, "Missing required fields").into_response();
    };

    if !is_valid_username(&username) {
        return (StatusCode::BAD_REQUEST, "Invalid username").into_response();
    }

    if file_bytes.len() > MAX_AVATAR_SIZE_BYTES {
        return (StatusCode::PAYLOAD_TOO_LARGE, "File exceeds the 1MB limit").into_response();
    }

    let Some(extension) = sniff_image_extension(&file_bytes) else {
        return (
            StatusCode::BAD_REQUEST,
            "Unsupported or unrecognized image format (png/jpg/gif/webp only)",
        ).into_response();
    };

    if !validate_session(&username, &session_token).await {
        return (StatusCode::UNAUTHORIZED, "Invalid session").into_response();
    }

    let avatars_dir: String = get_avatars_dir_path();
    if tokio::fs::create_dir_all(&avatars_dir).await.is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to prepare storage").into_response();
    }

    // Remove any previous avatar for this user, which may have a different extension.
    for ext in ALLOWED_AVATAR_EXTENSIONS {
        let _ = tokio::fs::remove_file(format!("{}/{}.{}", avatars_dir, username, ext)).await;
    }

    let path: String = format!("{}/{}.{}", avatars_dir, username, extension);
    if tokio::fs::write(&path, &file_bytes).await.is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save avatar").into_response();
    }

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        serde_json
            ::json!({ "success": true, "avatar_url": format!("/avatar/{}", username) })
            .to_string(),
    ).into_response()
}

pub async fn serve_avatar(Path(username): Path<String>) -> Response {
    if !is_valid_username(&username) {
        return StatusCode::NOT_FOUND.into_response();
    }

    let avatars_dir: String = get_avatars_dir_path();
    for ext in ALLOWED_AVATAR_EXTENSIONS {
        let path: String = format!("{}/{}.{}", avatars_dir, username, ext);
        if let Ok(bytes) = tokio::fs::read(&path).await {
            return ([(header::CONTENT_TYPE, content_type_for_extension(ext))], bytes).into_response();
        }
    }

    StatusCode::NOT_FOUND.into_response()
}
