// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use serde_json::Value;

use super::api::LanguageServerResponse;
use super::jsonrpc::Error;

type ApiTx = crossbeam_channel::Sender<LanguageServerResponse>;

pub(crate) fn handle_initialize_result(api_tx: &ApiTx, raw_result: Value) {
    //eprintln!("Server name: {}, version: {:?}", );
}