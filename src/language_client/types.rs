// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::convert::TryFrom;

use serde::{Deserialize, Serialize};

use super::uri::Uri;

#[derive(Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct Position {
    pub(crate) line: usize,
    pub(crate) character: usize,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Range {
    pub(crate) start: Position,
    pub(crate) end: Position,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Location {
    pub(crate) uri: Uri,
    pub(crate) range: Range,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct Diagnostic {
    pub(crate) range: Range,
    pub(crate) severity: Option<DiagnosticSeverity>,
    pub(crate) code: Option<DiagnosticCode>,
    pub(crate) source: Option<String>,
    pub(crate) message: String,
    pub(crate) tags: Option<Vec<DiagnosticTag>>,
    pub(crate) relatedInformation: Option<Vec<DiagnosticRelatedInformation>>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PublishDiagnosticParams {
    pub(crate) uri: Uri,
    pub(crate) version: Option<i64>,
    pub(crate) diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum DiagnosticCode {
    Str(String),
    Num(i64),
}

#[derive(Debug, Deserialize)]
#[serde(try_from = "u8")]
pub(crate) enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

impl TryFrom<u8> for DiagnosticSeverity {
    type Error = u8;

    fn try_from(u: u8) -> Result<Self, u8> {
        match u {
            1 => Ok(DiagnosticSeverity::Error),
            2 => Ok(DiagnosticSeverity::Warning),
            3 => Ok(DiagnosticSeverity::Information),
            4 => Ok(DiagnosticSeverity::Hint),
            _ => Err(u),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(try_from = "u8")]
pub(crate) enum DiagnosticTag {
    Unnecesaary,
    Deprecated,
}

impl TryFrom<u8> for DiagnosticTag {
    type Error = u8;

    fn try_from(u: u8) -> Result<Self, u8> {
        match u {
            1 => Ok(DiagnosticTag::Unnecesaary),
            2 => Ok(DiagnosticTag::Deprecated),
            _ => Err(u),
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct DiagnosticRelatedInformation {
    pub(crate) location: Location,
    pub(crate) message: String,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub(super) struct InitializeParams {
    pub(super) processId: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) clientInfo: Option<ClientInfo>,
    pub(super) rootUri: Option<Uri>,
    pub(super) capabilities: ClientCapabilities,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) trace: Option<Trace>,
}

#[derive(Debug, Serialize)]
pub(super) struct ClientInfo {
    pub(super) name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) version: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct ClientCapabilities {}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub(super) struct InitializeResult {}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum Trace {
    Off,
    Messages,
    Verbose,
}

#[derive(Debug, Serialize)]
pub(super) struct InitializedParams {}
