// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use super::jsonrpc::Id;
use super::types::{Hover, PublishDiagnosticParams};

pub(crate) enum LanguageServerResponse {
    Diagnostic(PublishDiagnosticParams),
    Hover(Id, String, Hover),
}
