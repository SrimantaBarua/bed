// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use serde_json::Value;

use super::api::LanguageServerResponse;
use super::jsonrpc::Error;

type ApiTx = crossbeam_channel::Sender<LanguageServerResponse>;

pub(crate) fn handle_initialize_result(api_tx: &ApiTx, raw_result: Value) {
    //eprintln!("Server name: {}, version: {:?}", );
}

pub(crate) fn handle_publish_diagnostics_notification(
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
