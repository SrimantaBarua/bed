// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use serde_json::Value;

use super::api::LanguageServerResponse;

type ApiTx = crossbeam_channel::Sender<LanguageServerResponse>;

pub(super) fn handle_publish_diagnostics_notification(
    api_tx: &ApiTx,
    opt_raw_params: Option<Value>,
) {
    opt_raw_params
        .and_then(|raw_params| serde_json::from_value(raw_params).ok())
        .map(|params| {
            api_tx
                .send(LanguageServerResponse::Diagnostic(params))
                .unwrap()
        });
}
