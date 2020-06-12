// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::fmt;

use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};
use serde_json::Value;

struct Message {
    content_type: String,
    content: MessageContent,
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let content = serde_json::to_string(&self.content).expect("failed to serialize JSON");
        let content_length = content.len();
        write!(
            f,
            "Content-Length: {}\r\nContent-Type: {}\r\n\r\n{}",
            content_length, self.content_type, content
        )
    }
}

#[derive(Serialize)]
#[serde(untagged)]
enum MessageContent {
    Request(Request),
    Response(Response),
}

enum Request {
    Call(Id, CallTyp),
    Notif(NotifTyp),
}

impl Serialize for Request {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Request::Call(i, c) => c.serialize(serializer, i),
            Request::Notif(n) => n.serialize(serializer),
        }
    }
}

enum CallTyp {
    Dummy,
}

impl CallTyp {
    fn serialize<S>(&self, serializer: S, id: &Id) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state =
            serializer.serialize_struct("Response", if self.has_params() { 4 } else { 3 })?;
        state.serialize_field("jsonrpc", "2.0")?;
        state.serialize_field("id", id)?;
        match self {
            CallTyp::Dummy => {
                state.serialize_field("method", "dummy")?;
            }
        }
        state.end()
    }

    fn has_params(&self) -> bool {
        false
    }
}

enum NotifTyp {
    Dummy,
}

impl NotifTyp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state =
            serializer.serialize_struct("Response", if self.has_params() { 3 } else { 2 })?;
        state.serialize_field("jsonrpc", "2.0")?;
        match self {
            NotifTyp::Dummy => {
                state.serialize_field("method", "dummy")?;
            }
        }
        state.end()
    }

    fn has_params(&self) -> bool {
        false
    }
}

enum Response {
    Result(Id, Value),
    Error(Id, Error),
}

impl Serialize for Response {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Response", 3)?;
        state.serialize_field("jsonrpc", "2.0")?;
        match self {
            Response::Result(id, r) => {
                state.serialize_field("id", id)?;
                state.serialize_field("result", r)?;
            }
            Response::Error(id, e) => {
                state.serialize_field("id", id)?;
                state.serialize_field("error", e)?;
            }
        }
        state.end()
    }
}

#[derive(Serialize)]
struct Error {
    code: i64,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum Id {
    Str(String),
    Num(i64),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_call() {
        let message = Message {
            content_type: "application/vscode-jsonrpc; charset=utf-8".to_owned(),
            content: MessageContent::Request(Request::Call(Id::Num(0), CallTyp::Dummy)),
        };
        assert_eq!(&message.to_string(), "Content-Length: 41\r\nContent-Type: application/vscode-jsonrpc; charset=utf-8\r\n\r\n{\"jsonrpc\":\"2.0\",\"id\":0,\"method\":\"dummy\"}");
    }

    #[test]
    fn test_serialize_notification() {
        let message = Message {
            content_type: "application/vscode-jsonrpc; charset=utf-8".to_owned(),
            content: MessageContent::Request(Request::Notif(NotifTyp::Dummy)),
        };
        assert_eq!(&message.to_string(), "Content-Length: 34\r\nContent-Type: application/vscode-jsonrpc; charset=utf-8\r\n\r\n{\"jsonrpc\":\"2.0\",\"method\":\"dummy\"}");
    }

    #[test]
    fn test_serialize_result() {
        let message = Message {
            content_type: "application/vscode-jsonrpc; charset=utf-8".to_owned(),
            content: MessageContent::Response(Response::Result(Id::Num(0), Value::Null)),
        };
        assert_eq!(&message.to_string(), "Content-Length: 38\r\nContent-Type: application/vscode-jsonrpc; charset=utf-8\r\n\r\n{\"jsonrpc\":\"2.0\",\"id\":0,\"result\":null}");
    }

    #[test]
    fn test_serialize_error() {
        let message = Message {
            content_type: "application/vscode-jsonrpc; charset=utf-8".to_owned(),
            content: MessageContent::Response(Response::Error(
                Id::Num(0),
                Error {
                    code: -32700,
                    message: "ParseError".to_owned(),
                    data: None,
                },
            )),
        };
        assert_eq!(&message.to_string(), "Content-Length: 71\r\nContent-Type: application/vscode-jsonrpc; charset=utf-8\r\n\r\n{\"jsonrpc\":\"2.0\",\"id\":0,\"error\":{\"code\":-32700,\"message\":\"ParseError\"}}");
    }
}
