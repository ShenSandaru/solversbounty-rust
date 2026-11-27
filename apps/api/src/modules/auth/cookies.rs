use subtle::ConstantTimeEq;

use super::constants::{CSRF_COOKIE, SESSION_COOKIE};

pub fn read_named_cookie(header: Option<&str>, name: &str) -> Option<String> {
    let header = header?;
    for part in header.split(';') {
        let trimmed = part.trim();
        let Some((cookie_name, value)) = trimmed.split_once('=') else {
            continue;
        };
        if cookie_name == name {
            return Some(urlencoding_like_decode(value));
        }
    }
    None
}

fn urlencoding_like_decode(value: &str) -> String {
    let mut out = String::new();
    let bytes = value.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(&value[i + 1..i + 3], 16) {
                out.push(byte as char);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

pub fn read_session_cookie(header: Option<&str>) -> Option<String> {
    read_named_cookie(header, SESSION_COOKIE)
}

pub fn session_cookie(token: &str, max_age_seconds: i64, secure: bool) -> String {
    cookie_line(SESSION_COOKIE, token, max_age_seconds, secure, true)
}

pub fn csrf_cookie(token: &str, max_age_seconds: i64, secure: bool) -> String {
    cookie_line(CSRF_COOKIE, token, max_age_seconds, secure, true)
}

pub fn clear_session_cookie(secure: bool) -> String {
    session_cookie("", 0, secure)
}

pub fn clear_csrf_cookie(secure: bool) -> String {
    csrf_cookie("", 0, secure)
}

pub fn csrf_tokens_match(cookie_header: Option<&str>, provided: Option<&str>) -> bool {
    let stored = read_named_cookie(cookie_header, CSRF_COOKIE);
    let (Some(stored), Some(provided)) = (stored, provided) else {
        return false;
    };
    let left = stored.as_bytes();
    let right = provided.as_bytes();
    if left.len() != right.len() {
        return false;
    }
    left.ct_eq(right).into()
}

fn cookie_line(name: &str, token: &str, max_age_seconds: i64, secure: bool, http_only: bool) -> String {
    let encoded = percent_encode_cookie_value(token);
    let mut parts = vec![
        format!("{name}={encoded}"),
        "SameSite=Lax".into(),
        "Path=/".into(),
        format!("Max-Age={max_age_seconds}"),
    ];
    if http_only {
        parts.insert(1, "HttpOnly".into());
    }
    if secure {
        parts.push("Secure".into());
    }
    parts.join("; ")
}

fn percent_encode_cookie_value(token: &str) -> String {
    let mut out = String::new();
    for b in token.bytes() {
        if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.' || b == b'~' {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}