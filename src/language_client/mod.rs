// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::ffi::OsStr;
use std::fs::read_dir;
use std::io::{BufRead, BufReader, Read, Result as IOResult, Write};
use std::path::Path;
use std::process::{ChildStdin, ChildStdout, Command};
use std::rc::Rc;
use std::str;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

use crossbeam_channel::{unbounded, Receiver, Sender};
use fnv::FnvHashMap;
use ropey::Rope;

use crate::config::Config;
use crate::language::Language;

mod api;
mod jsonrpc;
mod language_server_protocol;
mod types;
mod uri;

pub(crate) use api::LanguageServerResponse;
pub(crate) use types::{
    Diagnostic, DiagnosticCode, DiagnosticRelatedInformation, DiagnosticSeverity, DiagnosticTag,
    Position, PublishDiagnosticParams, Range,
};

use jsonrpc::{Id, Message, MessageContent};
use language_server_protocol as lsp;
use types::*;

pub(crate) struct LanguageClientManager {
    clients: FnvHashMap<(String, Language), LanguageClient>,
    api_tx: Sender<LanguageServerResponse>,
}

impl LanguageClientManager {
    pub(crate) fn new(api_tx: Sender<LanguageServerResponse>) -> Self {
        LanguageClientManager {
            clients: FnvHashMap::default(),
            api_tx,
        }
    }

    pub(crate) fn get_client(
        &mut self,
        language: Language,
        file_path: &str,
        config: &Config,
    ) -> Option<IOResult<LanguageClient>> {
        config
            .language
            .get(&language)
            .and_then(|lang_config| lang_config.language_server.as_ref())
            .and_then(|ls_config| {
                let abspath = crate::common::abspath(file_path);
                let path = Path::new(&abspath);
                path.parent().and_then(|dirpath| {
                    let mut root_path = dirpath;
                    'outer: for path in dirpath.ancestors() {
                        if let Ok(readdir) = read_dir(path) {
                            for entry in readdir.filter_map(|e| e.ok()) {
                                let child = entry.file_name();
                                for marker in config
                                    .completion_langserver_root_markers
                                    .iter()
                                    .chain(ls_config.root_markers.iter())
                                {
                                    if child == Path::new(&marker) {
                                        root_path = path;
                                        break 'outer;
                                    }
                                }
                            }
                        }
                    }
                    root_path.to_str().map(|path| {
                        let path = path.to_owned();
                        if let Some(lc) = self.clients.get(&(path.clone(), language.clone())) {
                            Ok(lc.clone())
                        } else {
                            LanguageClient::new(
                                &ls_config.executable,
                                &ls_config.arguments.clone(),
                                self.api_tx.clone(),
                                &path,
                            )
                            .map(|lc| {
                                let lc = lc;
                                self.clients.insert((path, language), lc.clone());
                                lc
                            })
                        }
                    })
                })
            })
    }
}

enum WriterMessage {
    Exit,
    Message(Message),
}

struct LanguageClientSyncState {
    id_method_map: FnvHashMap<Id, String>,
    server_capabilities: Option<ServerCapabilities>,
}

#[derive(Clone)]
pub(crate) struct LanguageClient {
    inner: Rc<RefCell<LanguageClientInner>>,
}

impl LanguageClient {
    fn new<S>(
        command: &str,
        args: &[S],
        api_tx: Sender<LanguageServerResponse>,
        root_path: &str,
    ) -> IOResult<LanguageClient>
    where
        S: AsRef<OsStr>,
    {
        LanguageClientInner::new(command, args, api_tx, root_path).map(|i| LanguageClient {
            inner: Rc::new(RefCell::new(i)),
        })
    }

    pub(crate) fn text_document_open(
        &mut self,
        path: &str,
        language: Language,
        version: usize,
        text: &Rope,
    ) {
        let inner = &mut *self.inner.borrow_mut();
        {
            let sync_state = inner.sync_state.lock().unwrap();
            if let Some(cap) = &sync_state.server_capabilities {
                if !cap.send_open_close() {
                    return;
                }
            }
        }
        let uri = uri::Uri::from_path(path).expect("failed to parse path URI");
        inner
            .wmsg_tx
            .send(WriterMessage::Message(Message::new(
                MessageContent::Notification {
                    method: "textDocument/didOpen".to_owned(),
                    params: Some(
                        serde_json::to_value(DidOpenTextDocumentParams {
                            textDocument: TextDocumentItem {
                                uri,
                                languageId: language.to_string(),
                                version,
                                text: text.to_string(),
                            },
                        })
                        .unwrap(),
                    ),
                },
            )))
            .unwrap();
    }

    pub(crate) fn text_document_close(&mut self, path: &str) {
        let inner = &mut *self.inner.borrow_mut();
        {
            let sync_state = inner.sync_state.lock().unwrap();
            if let Some(cap) = &sync_state.server_capabilities {
                if !cap.send_open_close() {
                    return;
                }
            }
        }
        let uri = uri::Uri::from_path(path).expect("failed to parse path URI");
        inner
            .wmsg_tx
            .send(WriterMessage::Message(Message::new(
                MessageContent::Notification {
                    method: "textDocument/didClose".to_owned(),
                    params: Some(
                        serde_json::to_value(DidCloseTextDocumentParams {
                            textDocument: TextDocumentIdentifier { uri },
                        })
                        .unwrap(),
                    ),
                },
            )))
            .unwrap();
    }

    pub(crate) fn send_full_document_on_change(&self) -> bool {
        let inner = &mut *self.inner.borrow_mut();
        let sync_state = inner.sync_state.lock().unwrap();
        match sync_state
            .server_capabilities
            .as_ref()
            .unwrap()
            .send_change()
        {
            TextDocumentSyncKind::None => panic!("no text to be sent on save?"),
            TextDocumentSyncKind::Incremental => false,
            TextDocumentSyncKind::Full => true,
        }
    }

    pub(crate) fn text_document_change_full(&mut self, path: &str, version: usize, text: String) {
        let inner = &mut *self.inner.borrow_mut();
        let uri = uri::Uri::from_path(path).expect("failed to parse path URI");
        let version = Some(version);
        inner
            .wmsg_tx
            .send(WriterMessage::Message(Message::new(
                MessageContent::Notification {
                    method: "textDocument/didChange".to_owned(),
                    params: Some(
                        serde_json::to_value(DidChangeTextDocumentParams {
                            textDocument: VersionedTextDocumentIdentifier { uri, version },
                            contentChanges: vec![TextDocumentContentChangeEvent::Full { text }],
                        })
                        .unwrap(),
                    ),
                },
            )))
            .unwrap();
    }

    pub(crate) fn text_document_change(
        &mut self,
        path: &str,
        version: usize,
        range: Range,
        text: String,
    ) {
        let inner = &mut *self.inner.borrow_mut();
        let uri = uri::Uri::from_path(path).expect("failed to parse path URI");
        let version = Some(version);
        inner
            .wmsg_tx
            .send(WriterMessage::Message(Message::new(
                MessageContent::Notification {
                    method: "textDocument/didChange".to_owned(),
                    params: Some(
                        serde_json::to_value(DidChangeTextDocumentParams {
                            textDocument: VersionedTextDocumentIdentifier { uri, version },
                            contentChanges: vec![TextDocumentContentChangeEvent::Ranged {
                                range,
                                text,
                            }],
                        })
                        .unwrap(),
                    ),
                },
            )))
            .unwrap();
    }

    pub(crate) fn text_document_save(&mut self, path: &str, text: &Rope) {
        let inner = &mut *self.inner.borrow_mut();
        let text = {
            let sync_state = inner.sync_state.lock().unwrap();
            if let Some(cap) = &sync_state.server_capabilities {
                if !cap.send_save() {
                    return;
                }
                if cap.save_send_text() {
                    Some(text.to_string())
                } else {
                    None
                }
            } else {
                None
            }
        };
        let uri = uri::Uri::from_path(path).expect("failed to parse path URI");
        inner
            .wmsg_tx
            .send(WriterMessage::Message(Message::new(
                MessageContent::Notification {
                    method: "textDocument/didSave".to_owned(),
                    params: Some(
                        serde_json::to_value(DidSaveTextDocumentParams {
                            textDocument: TextDocumentIdentifier { uri },
                            text,
                        })
                        .unwrap(),
                    ),
                },
            )))
            .unwrap();
    }
}

struct LanguageClientInner {
    writer_thread: Option<thread::JoinHandle<()>>,
    reader_thread: Option<thread::JoinHandle<()>>,
    sync_state: Arc<Mutex<LanguageClientSyncState>>,
    wmsg_tx: Sender<WriterMessage>,
    next_id: i64,
}

impl Drop for LanguageClientInner {
    fn drop(&mut self) {
        self.wmsg_tx.send(WriterMessage::Exit).unwrap();
        if let Some(thread) = self.writer_thread.take() {
            let _ = thread.join();
        }
        if let Some(thread) = self.reader_thread.take() {
            let _ = thread.join();
        }
    }
}

impl LanguageClientInner {
    fn new<S>(
        command: &str,
        args: &[S],
        api_tx: Sender<LanguageServerResponse>,
        root_path: &str,
    ) -> IOResult<LanguageClientInner>
    where
        S: AsRef<OsStr>,
    {
        let child = Command::new(command)
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()?;
        let process_id = child.id();
        let reader = Box::new(BufReader::new(child.stdout.unwrap()));
        let writer = Box::new(child.stdin.unwrap());
        let (wmsg_tx, wmsg_rx) = unbounded();

        let mut sync_state = LanguageClientSyncState {
            id_method_map: FnvHashMap::default(),
            server_capabilities: None,
        };
        sync_state
            .id_method_map
            .insert(Id::Num(0), "initialize".to_owned());
        let sync_state = Arc::new(Mutex::new(sync_state));
        let sync_state_1 = sync_state.clone();

        let cond = Arc::new((Mutex::new(false), Condvar::new()));
        let cond2 = cond.clone();

        let reader_thread = Some(thread::spawn(move || {
            language_client_reader(reader, sync_state_1, api_tx, cond2)
        }));
        let writer_thread = Some(thread::spawn(move || {
            language_client_writer(writer, wmsg_rx)
        }));

        let ret = LanguageClientInner {
            reader_thread,
            writer_thread,
            wmsg_tx,
            next_id: 1,
            sync_state,
        };
        let root_uri = uri::Uri::from_path(&root_path).expect("failed to parse root URI");
        ret.wmsg_tx
            .send(WriterMessage::Message(Message::new(MessageContent::Call {
                id: Id::Num(0),
                method: "initialize".to_owned(),
                params: Some(
                    serde_json::to_value(InitializeParams {
                        processId: Some(process_id),
                        clientInfo: Some(ClientInfo {
                            name: crate_name!().to_owned(),
                            version: Some(crate_version!().to_owned()),
                        }),
                        rootUri: Some(root_uri),
                        capabilities: ClientCapabilities {
                            textDocument: Some(TextDocumentClientCapabilities {
                                synchronization: Some(TextDocumentSyncClientCapabilities {
                                    dynamicRegistration: Some(false),
                                    willSave: Some(false),
                                    willSaveWaitUntil: Some(false),
                                    didSave: Some(true),
                                }),
                                publishDiagnostics: Some(PublishDiagnosticsClientCapabilities {
                                    relatedInformation: Some(false),
                                    tagSupport: Some(PublishDiagnosticsClientTagSupport {
                                        valueSet: vec![
                                            DiagnosticTag::Unnecessary,
                                            DiagnosticTag::Deprecated,
                                        ],
                                    }),
                                    versionSupport: Some(true),
                                }),
                            }),
                        },
                    })
                    .unwrap(),
                ),
            })))
            .unwrap();

        let (lock, cvar) = &*cond;
        let mut initialized = lock.lock().unwrap();
        while !*initialized {
            initialized = cvar.wait(initialized).unwrap();
        }

        ret.wmsg_tx
            .send(WriterMessage::Message(Message::new(
                MessageContent::Notification {
                    method: "initialized".to_owned(),
                    params: Some(serde_json::to_value(InitializedParams {}).unwrap()),
                },
            )))
            .unwrap();

        Ok(ret)
    }
}

fn language_client_reader(
    mut reader: Box<BufReader<ChildStdout>>,
    sync_state: Arc<Mutex<LanguageClientSyncState>>,
    api_tx: Sender<LanguageServerResponse>,
    initialized_cond: Arc<(Mutex<bool>, Condvar)>,
) {
    let mut line = String::new();
    let mut content = Vec::new();
    let mut opt_content_length;
    'outer: loop {
        opt_content_length = None;
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(n) if n > 0 => {}
                _ => break 'outer,
            }
            let trimmed = line.trim();
            if trimmed.len() == 0 {
                break;
            }
            if trimmed.starts_with("Content-Length: ") {
                opt_content_length = trimmed["Content-Length: ".len()..].trim().parse().ok();
            }
        }
        if let Some(content_length) = opt_content_length {
            content.resize(content_length, 0);
            if reader.read_exact(&mut content).is_err() {
                break;
            }
            if let Ok(raw_message) = serde_json::from_slice::<MessageContent>(&content) {
                /*
                debug!(
                    "RECEIVED: {}",
                    serde_json::to_string_pretty(&raw_message).unwrap()
                );
                */
                match raw_message {
                    MessageContent::Call { id, method, params } => {
                        debug!(
                            "raw_message: {}",
                            serde_json::to_string_pretty(&MessageContent::Call {
                                id,
                                method,
                                params,
                            })
                            .unwrap()
                        );
                    }
                    MessageContent::Notification { method, params } => match method.as_ref() {
                        "textDocument/publishDiagnostics" => {
                            lsp::handle_publish_diagnostics_notification(&api_tx, params)
                        }
                        _ => {
                            debug!(
                                "raw_message: {}",
                                serde_json::to_string_pretty(&MessageContent::Notification {
                                    method,
                                    params,
                                })
                                .unwrap()
                            );
                        }
                    },
                    MessageContent::Result { id, result } => {
                        let mut locked_state = sync_state.lock().unwrap();
                        if let Some(method) = locked_state.id_method_map.remove(&id) {
                            match method.as_ref() {
                                "initialize" => {
                                    let formatted = serde_json::to_string(&result).unwrap();
                                    let params =
                                        match serde_json::from_value::<InitializeResult>(result) {
                                            Ok(params) => params,
                                            Err(e) => panic!(
                                                "failed to parse initialize result: {}: {}",
                                                e, formatted
                                            ),
                                        };
                                    locked_state.server_capabilities = Some(params.capabilities);
                                    let (lock, cvar) = &*initialized_cond;
                                    let mut initialized = lock.lock().unwrap();
                                    *initialized = true;
                                    cvar.notify_one();
                                }
                                _ => {
                                    debug!(
                                        "raw_message: {}",
                                        serde_json::to_string_pretty(&MessageContent::Result {
                                            id,
                                            result
                                        })
                                        .unwrap()
                                    );
                                }
                            }
                        } else {
                            error!(
                                "Result without ID: {}",
                                serde_json::to_string_pretty(&MessageContent::Result {
                                    id,
                                    result
                                })
                                .unwrap()
                            )
                        }
                    }
                    MessageContent::Error { id, error } => {
                        if let Some(method) =
                            { sync_state.lock().unwrap().id_method_map.remove(&id) }
                        {
                            match method.as_ref() {
                                "initialize" => panic!(
                                    "failed to initialize language server: {}",
                                    serde_json::to_string(&error).unwrap()
                                ),
                                _ => {
                                    debug!(
                                        "raw_message: {}",
                                        serde_json::to_string_pretty(&MessageContent::Error {
                                            id,
                                            error
                                        })
                                        .unwrap()
                                    );
                                }
                            }
                        } else {
                            error!(
                                "Error without ID: {}",
                                serde_json::to_string_pretty(&MessageContent::Error { id, error })
                                    .unwrap()
                            )
                        }
                    }
                }
            }
        }
    }
}

fn language_client_writer(mut writer: Box<ChildStdin>, wmsg_rx: Receiver<WriterMessage>) {
    while let Ok(message) = wmsg_rx.recv() {
        match message {
            WriterMessage::Exit => {
                let _ = write!(
                    &mut writer,
                    "{}",
                    Message::new(MessageContent::Notification {
                        method: "exit".to_owned(),
                        params: None,
                    })
                );
                break;
            }
            WriterMessage::Message(message) => {
                /*
                 * debug!("MESSAGE: {}", message);
                 */
                if write!(&mut writer, "{}", message).is_err() {
                    break;
                }
            }
        }
    }
}
