// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::convert::TryFrom;

use serde::{Deserialize, Serialize};

use super::uri::Uri;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub(crate) struct Position {
    pub(crate) line: usize,
    pub(crate) character: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Range {
    pub(crate) start: Position,
    pub(crate) end: Position,
}

#[derive(Clone, Debug, Deserialize)]
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
    pub(crate) version: Option<usize>,
    pub(crate) diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum DiagnosticCode {
    Str(String),
    Num(i64),
}

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "u8")]
#[serde(into = "u8")]
pub(crate) enum DiagnosticTag {
    Unnecessary,
    Deprecated,
}

impl TryFrom<u8> for DiagnosticTag {
    type Error = u8;

    fn try_from(u: u8) -> Result<Self, u8> {
        match u {
            1 => Ok(DiagnosticTag::Unnecessary),
            2 => Ok(DiagnosticTag::Deprecated),
            _ => Err(u),
        }
    }
}

impl Into<u8> for DiagnosticTag {
    fn into(self) -> u8 {
        match self {
            DiagnosticTag::Unnecessary => 1,
            DiagnosticTag::Deprecated => 2,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
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
}

#[derive(Debug, Serialize)]
pub(super) struct ClientInfo {
    pub(super) name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) version: Option<String>,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub(super) struct PublishDiagnosticsClientCapabilities {
    pub(super) relatedInformation: Option<bool>,
    pub(super) tagSupport: Option<PublishDiagnosticsClientTagSupport>,
    pub(super) versionSupport: Option<bool>,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub(super) struct PublishDiagnosticsClientTagSupport {
    pub(super) valueSet: Vec<DiagnosticTag>,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub(super) struct TextDocumentClientCapabilities {
    pub(super) synchronization: Option<TextDocumentSyncClientCapabilities>,
    pub(super) publishDiagnostics: Option<PublishDiagnosticsClientCapabilities>,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub(super) struct TextDocumentSyncClientCapabilities {
    pub(super) dynamicRegistration: Option<bool>,
    pub(super) willSave: Option<bool>,
    pub(super) willSaveWaitUntil: Option<bool>,
    pub(super) didSave: Option<bool>,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub(super) struct ClientCapabilities {
    pub(super) textDocument: Option<TextDocumentClientCapabilities>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub(super) struct InitializeResult {
    pub(super) capabilities: ServerCapabilities,
    pub(super) serverInfo: Option<ServerInfo>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub(super) struct ServerCapabilities {
    #[serde(default)]
    pub(super) textDocumentSync: ServerTextDocumentSync,
}

impl ServerCapabilities {
    pub(super) fn send_open_close(&self) -> bool {
        match &self.textDocumentSync {
            ServerTextDocumentSync::Kind(_) => true,
            ServerTextDocumentSync::Options(o) => o.openClose,
        }
    }

    pub(super) fn send_save(&self) -> bool {
        match &self.textDocumentSync {
            ServerTextDocumentSync::Kind(_) => true,
            ServerTextDocumentSync::Options(o) => o.save.is_some(),
        }
    }

    pub(super) fn send_change(&self) -> TextDocumentSyncKind {
        match &self.textDocumentSync {
            ServerTextDocumentSync::Kind(k) => *k,
            ServerTextDocumentSync::Options(o) => o.change,
        }
    }

    pub(super) fn save_send_text(&self) -> bool {
        match &self.textDocumentSync {
            ServerTextDocumentSync::Kind(_) => false,
            ServerTextDocumentSync::Options(o) => match &o.save {
                None => false,
                Some(TextDocumentSaveOptions::Options(o)) => o.includeText,
                Some(TextDocumentSaveOptions::Bool(b)) => *b,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(try_from = "u8")]
pub(super) enum TextDocumentSyncKind {
    None,
    Full,
    Incremental,
}

impl Default for TextDocumentSyncKind {
    fn default() -> TextDocumentSyncKind {
        TextDocumentSyncKind::None
    }
}

impl TryFrom<u8> for TextDocumentSyncKind {
    type Error = u8;

    fn try_from(u: u8) -> Result<Self, u8> {
        match u {
            0 => Ok(TextDocumentSyncKind::None),
            1 => Ok(TextDocumentSyncKind::Full),
            2 => Ok(TextDocumentSyncKind::Incremental),
            _ => Err(u),
        }
    }
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub(super) struct TextDocumentSyncOptions {
    #[serde(default)]
    pub(super) openClose: bool,
    #[serde(default)]
    pub(super) change: TextDocumentSyncKind,
    #[serde(default)]
    pub(super) willSave: bool,
    #[serde(default)]
    pub(super) willSaveWaitUntil: bool,
    pub(super) save: Option<TextDocumentSaveOptions>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub(super) struct SaveOptions {
    #[serde(default)]
    pub(super) includeText: bool,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum TextDocumentSaveOptions {
    Bool(bool),
    Options(SaveOptions),
}

impl Default for TextDocumentSaveOptions {
    fn default() -> TextDocumentSaveOptions {
        TextDocumentSaveOptions::Bool(false)
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum ServerTextDocumentSync {
    Kind(TextDocumentSyncKind),
    Options(TextDocumentSyncOptions),
}

impl Default for ServerTextDocumentSync {
    fn default() -> ServerTextDocumentSync {
        ServerTextDocumentSync::Kind(TextDocumentSyncKind::default())
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct ServerInfo {
    pub(super) name: String,
    pub(super) version: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct InitializedParams {}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub(super) struct DidOpenTextDocumentParams {
    pub(super) textDocument: TextDocumentItem,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub(super) struct DidSaveTextDocumentParams {
    pub(super) textDocument: TextDocumentIdentifier,
    pub(super) text: Option<String>,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub(super) struct DidCloseTextDocumentParams {
    pub(super) textDocument: TextDocumentIdentifier,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub(super) struct DidChangeTextDocumentParams {
    pub(super) textDocument: VersionedTextDocumentIdentifier,
    pub(super) contentChanges: Vec<TextDocumentContentChangeEvent>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub(super) enum TextDocumentContentChangeEvent {
    Ranged { range: Range, text: String },
    Full { text: String },
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub(super) struct TextDocumentItem {
    pub(super) uri: Uri,
    pub(super) languageId: String,
    pub(super) version: usize,
    pub(super) text: String,
}

#[derive(Debug, Serialize)]
pub(super) struct TextDocumentIdentifier {
    pub(super) uri: Uri,
}

#[derive(Debug, Serialize)]
pub(super) struct VersionedTextDocumentIdentifier {
    pub(super) uri: Uri,
    pub(super) version: Option<usize>,
}
