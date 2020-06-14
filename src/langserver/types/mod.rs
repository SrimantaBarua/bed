// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::convert::TryFrom;

use serde::{Deserialize, Serialize};

mod uri;
pub(super) use uri::Uri;

#[derive(Serialize, Deserialize)]
pub(super) struct Position {
    pub(super) line: usize,
    pub(super) character: usize,
}

#[derive(Serialize, Deserialize)]
pub(super) struct Range {
    pub(super) start: Position,
    pub(super) end: Position,
}

#[derive(Serialize, Deserialize)]
pub(super) struct Location {
    pub(super) uri: Uri,
    pub(super) range: Range,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(super) struct LocationLink {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) originSelectionRange: Option<Range>,
    pub(super) targetUri: Uri,
    pub(super) targetRange: Range,
    pub(super) targetSelectionRange: Range,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(super) struct Diagnostic {
    pub(super) range: Range,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) severity: Option<DiagnosticSeverity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) code: Option<DiagnosticCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) source: Option<String>,
    pub(super) message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) tags: Option<Vec<DiagnosticTag>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) relatedInformation: Option<Vec<DiagnosticRelatedInformation>>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub(super) enum DiagnosticCode {
    Str(String),
    Num(i64),
}

#[derive(Serialize, Deserialize)]
#[serde(try_from = "usize")]
pub(super) enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

impl TryFrom<usize> for DiagnosticSeverity {
    type Error = usize;

    fn try_from(u: usize) -> Result<Self, usize> {
        match u {
            1 => Ok(DiagnosticSeverity::Error),
            2 => Ok(DiagnosticSeverity::Warning),
            3 => Ok(DiagnosticSeverity::Information),
            4 => Ok(DiagnosticSeverity::Hint),
            _ => Err(u),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(try_from = "usize")]
pub(super) enum DiagnosticTag {
    Unnecessary,
    Deprecated,
}

impl TryFrom<usize> for DiagnosticTag {
    type Error = usize;

    fn try_from(u: usize) -> Result<Self, usize> {
        match u {
            1 => Ok(DiagnosticTag::Unnecessary),
            2 => Ok(DiagnosticTag::Deprecated),
            _ => Err(u),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(super) struct DiagnosticRelatedInformation {
    pub(super) location: Location,
    pub(super) message: String,
}

#[derive(Serialize)]
#[allow(non_snake_case)]
pub(super) struct InitializeParams {
    pub(super) processID: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) clientInfo: Option<ClientInfo>,
    pub(super) rootUri: Option<Uri>,
    pub(super) capabilities: ClientCapabilities,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) trace: Option<Trace>,
}

#[derive(Serialize)]
pub(super) struct ClientInfo {
    pub(super) name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) version: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub(super) struct ClientCapabilities {}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub(super) struct InitializeResult {
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum Trace {
    Off,
    Messages,
    Verbose,
}
