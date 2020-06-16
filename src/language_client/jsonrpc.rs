// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::fmt;

use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub(super) struct Message {
    content_type: Option<String>,
    content: MessageContent,
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let content = serde_json::to_string(&self.content).expect("failed to serialize JSON");
        let content_length = content.len();
        write!(f, "Content-Length: {}\r\n", content_length)?;
        if let Some(content) = &self.content_type {
            write!(f, "Content-Type: {}\r\n", content)?;
        }
        write!(f, "\r\n{}", &content)
    }
}

impl Message {
    pub(super) fn new(content: MessageContent) -> Message {
        Message {
            content_type: None,
            content,
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
pub(super) enum MessageContent {
    Call {
        id: Id,
        method: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        params: Option<Value>,
    },
    Notification {
        method: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        params: Option<Value>,
    },
    Result {
        id: Id,
        result: Value,
    },
    Error {
        id: Id,
        error: Error,
    },
}

impl Serialize for MessageContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            MessageContent::Call { id, method, params } => {
                let mut state =
                    serializer.serialize_struct("content", if params.is_none() { 4 } else { 3 })?;
                state.serialize_field("jsonrpc", "2.0")?;
                state.serialize_field("id", id)?;
                state.serialize_field("method", method)?;
                if let Some(params) = params {
                    state.serialize_field("params", params)?;
                }
                state.end()
            }
            MessageContent::Notification { method, params } => {
                let mut state =
                    serializer.serialize_struct("content", if params.is_none() { 3 } else { 2 })?;
                state.serialize_field("jsonrpc", "2.0")?;
                state.serialize_field("method", method)?;
                if let Some(params) = params {
                    state.serialize_field("params", params)?;
                }
                state.end()
            }
            MessageContent::Result { id, result } => {
                let mut state = serializer.serialize_struct("content", 3)?;
                state.serialize_field("jsonrpc", "2.0")?;
                state.serialize_field("id", id)?;
                state.serialize_field("result", result)?;
                state.end()
            }
            MessageContent::Error { id, error } => {
                let mut state = serializer.serialize_struct("content", 3)?;
                state.serialize_field("jsonrpc", "2.0")?;
                state.serialize_field("id", id)?;
                state.serialize_field("error", error)?;
                state.end()
            }
        }
    }
}

#[derive(Deserialize, Serialize)]
pub(super) struct Error {
    code: i64,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(untagged)]
pub(super) enum Id {
    Str(String),
    Num(i64),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_call() {
        let message = Message {
            content_type: None,
            content: MessageContent::Call {
                id: Id::Num(0),
                method: "dummy".to_owned(),
                params: None,
            },
        };
        assert_eq!(
            &message.to_string(),
            "Content-Length: 41\r\n\r\n{\"jsonrpc\":\"2.0\",\"id\":0,\"method\":\"dummy\"}"
        );
    }

    #[test]
    fn test_serialize_notification() {
        let message = Message {
            content_type: None,
            content: MessageContent::Notification {
                method: "dummy".to_owned(),
                params: None,
            },
        };
        assert_eq!(
            &message.to_string(),
            "Content-Length: 34\r\n\r\n{\"jsonrpc\":\"2.0\",\"method\":\"dummy\"}"
        );
    }

    #[test]
    fn test_serialize_result() {
        let message = Message {
            content_type: None,
            content: MessageContent::Result {
                id: Id::Num(0),
                result: Value::Null,
            },
        };
        assert_eq!(
            &message.to_string(),
            "Content-Length: 38\r\n\r\n{\"jsonrpc\":\"2.0\",\"id\":0,\"result\":null}"
        );
    }

    #[test]
    fn test_serialize_error() {
        let message = Message {
            content_type: None,
            content: MessageContent::Error {
                id: Id::Num(0),
                error: Error {
                    code: -32700,
                    message: "ParseError".to_owned(),
                    data: None,
                },
            },
        };
        assert_eq!(&message.to_string(), "Content-Length: 71\r\n\r\n{\"jsonrpc\":\"2.0\",\"id\":0,\"error\":{\"code\":-32700,\"message\":\"ParseError\"}}");
    }
}
