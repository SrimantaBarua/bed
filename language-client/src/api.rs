// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use super::types::PublishDiagnosticParams;

pub enum LanguageServerCommand {}

pub enum LanguageServerResponse {
    Diagnostic(PublishDiagnosticParams),
}
