// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use serde::{Deserialize, Serialize};

use super::uri::Uri;

#[derive(Serialize)]
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
pub(super) struct InitializeResult {}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum Trace {
    Off,
    Messages,
    Verbose,
}
