// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::convert::TryFrom;

use serde::{Deserialize, Serialize};

use super::uri::Uri;

#[derive(Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct Position {
    pub line: usize,
    pub character: usize,
}

#[derive(Debug, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Deserialize)]
pub struct Location {
    pub uri: Uri,
    pub range: Range,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: Option<DiagnosticSeverity>,
    pub code: Option<DiagnosticCode>,
    pub source: Option<String>,
    pub message: String,
    pub tags: Option<Vec<DiagnosticTag>>,
    pub relatedInformation: Option<Vec<DiagnosticRelatedInformation>>,
}

#[derive(Debug, Deserialize)]
pub struct PublishDiagnosticParams {
    pub uri: Uri,
    pub version: Option<i64>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum DiagnosticCode {
    Str(String),
    Num(i64),
}

#[derive(Debug, Deserialize)]
#[serde(try_from = "u8")]
pub enum DiagnosticSeverity {
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
pub enum DiagnosticTag {
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
pub struct DiagnosticRelatedInformation {
    pub location: Location,
    pub message: String,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub(crate) struct InitializeParams {
    pub(crate) processId: Option<u32>,
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
pub(crate) struct InitializedParams {}
