//! Envelope structs -> JSON (`docs/specs/01-envelope.md`).
//!
//! Every command prints exactly one envelope on stdout:
//! `{"status":"ok","message":"…","data":{…}}` (exit 0) or
//! `{"status":"error","message":"…","code":"…","details":{…}}` (exit 1).

use serde::Serialize;

use crate::core::error::CarpenterError;
use crate::models::Data;

#[derive(Serialize)]
struct OkEnvelope {
    status: &'static str,
    message: String,
    data: Data,
}

#[derive(Serialize)]
struct ErrEnvelope {
    status: &'static str,
    message: String,
    code: &'static str,
    details: serde_json::Value,
}

/// Render a command result into `(stdout_json, is_error)`.
///
/// Pure: serialization is infallible for our types (the `unwrap_or_else` fallback
/// only fires on a structurally-impossible encode failure, yielding a minimal
/// error envelope rather than a panic).
pub fn render(result: Result<Data, CarpenterError>) -> (String, bool) {
    match result {
        Ok(data) => {
            let message = data.message();
            let env = OkEnvelope {
                status: "ok",
                message,
                data,
            };
            (encode(&env), false)
        }
        Err(e) => {
            let env = ErrEnvelope {
                status: "error",
                message: e.to_string(),
                code: e.code(),
                details: e.details(),
            };
            (encode(&env), true)
        }
    }
}

fn encode<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value)
        .unwrap_or_else(|_| String::from(r#"{"status":"error","message":"envelope encode failed","code":"StoreError","details":{}}"#))
}

#[cfg(test)]
#[test]
fn render_ok_envelope() {
    let data = Data::Howto {
        howto: String::from("body"),
    };
    let (json, is_error) = render(Ok(data));
    assert!(!is_error);
    assert!(json.contains(r#""status":"ok""#), "{json}");
    assert!(json.contains(r#""data":{"howto":"body"}"#), "{json}");
}

#[cfg(test)]
#[test]
fn render_error_envelope() {
    let (json, is_error) = render(Err(CarpenterError::NotFound("p1".into())));
    assert!(is_error);
    assert!(json.contains(r#""status":"error""#), "{json}");
    assert!(json.contains(r#""code":"NotFound""#), "{json}");
    assert!(json.contains(r#""details":{}"#), "{json}");
}
