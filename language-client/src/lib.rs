// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::ffi::OsStr;
use std::fs::read_dir;
use std::hash::Hash;
use std::io::{BufRead, BufReader, Read, Result as IOResult, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command};
use std::rc::Rc;
use std::str;
use std::sync::{Arc, Mutex};
use std::thread;

use crossbeam_channel::{unbounded, Receiver, Sender};
use fnv::FnvHashMap;

mod api;
mod jsonrpc;
mod types;
mod uri;

pub use api::{LanguageServerCommand, LanguageServerResponse};

pub trait LanguageKey: Clone + Eq + Hash + PartialEq {}
pub trait BufferKey: 'static + Clone + Copy + Eq + Hash + PartialEq + Send + Sync {}

pub struct LanguageConfig<S, P>
where
    S: AsRef<OsStr>,
    P: AsRef<Path>,
{
    pub command: String,
    pub args: Vec<S>,
    pub root_markers: Vec<P>,
}

pub struct LanguageClientManager<L, B>
where
    L: LanguageKey,
    B: BufferKey,
{
    clients: FnvHashMap<(String, L), Rc<RefCell<LanguageClient<B>>>>,
    api_tx: Sender<(B, LanguageServerResponse)>,
    client_name: Option<String>,
    client_version: Option<String>,
}

impl<L, B> LanguageClientManager<L, B>
where
    L: LanguageKey,
    B: BufferKey,
{
    pub fn new(
        api_tx: Sender<(B, LanguageServerResponse)>,
        client_name: Option<String>,
        client_version: Option<String>,
    ) -> Self {
        LanguageClientManager {
            clients: FnvHashMap::default(),
            api_tx,
            client_name,
            client_version,
        }
    }

    pub fn get_client<S, P>(
        &mut self,
        language: L,
        file_path: &str,
        buffer_key: B,
        config: &LanguageConfig<S, P>,
    ) -> Option<IOResult<Rc<RefCell<LanguageClient<B>>>>>
    where
        S: AsRef<OsStr>,
        P: AsRef<Path>,
    {
        let abspath = absolute_path(file_path);
        let path = Path::new(&abspath);
        path.parent().and_then(|dirpath| {
            let mut root_path = dirpath;
            'outer: for path in dirpath.ancestors() {
                if let Ok(readdir) = read_dir(path) {
                    for entry in readdir.filter_map(|e| e.ok()) {
                        let child = entry.path();
                        for marker in &config.root_markers {
                            if child == marker.as_ref() {
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
                        &config.command,
                        &config.args,
                        self.api_tx.clone(),
                        buffer_key,
                        &path,
                        self.client_name.clone(),
                        self.client_version.clone(),
                    )
                    .map(|lc| {
                        let lc = Rc::new(RefCell::new(lc));
                        self.clients.insert((path, language), lc.clone());
                        lc
                    })
                }
            })
        })
    }
}

enum WriterMessage {
    Exit,
    Message(jsonrpc::Message),
}

pub struct LanguageClient<B>
where
    B: BufferKey,
{
    writer_thread: Option<thread::JoinHandle<()>>,
    reader_thread: Option<thread::JoinHandle<()>>,
    sync_state: Arc<Mutex<LanguageClientSyncState<B>>>,
    wmsg_tx: Sender<WriterMessage>,
    next_id: i64,
}

struct LanguageClientSyncState<B>
where
    B: BufferKey,
{
    id_buf_map: FnvHashMap<jsonrpc::Id, B>,
}

impl<B> Drop for LanguageClient<B>
where
    B: BufferKey,
{
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

impl<B> LanguageClient<B>
where
    B: BufferKey,
{
    pub fn new<S>(
        command: &str,
        args: &[S],
        api_tx: Sender<(B, LanguageServerResponse)>,
        buffer_key: B,
        root_path: &str,
        client_name: Option<String>,
        client_version: Option<String>,
    ) -> IOResult<LanguageClient<B>>
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

        let sync_state = Arc::new(Mutex::new(LanguageClientSyncState {
            id_buf_map: FnvHashMap::default(),
        }));
        let sync_state_1 = sync_state.clone();
        let sync_state_2 = sync_state.clone();

        let reader_thread = Some(thread::spawn(move || {
            language_client_reader(sync_state_1, reader, api_tx)
        }));
        let writer_thread = Some(thread::spawn(move || {
            language_client_writer(sync_state_2, writer, wmsg_rx)
        }));

        let ret = LanguageClient {
            sync_state,
            reader_thread,
            writer_thread,
            wmsg_tx,
            next_id: 1,
        };
        {}
        let root_uri = uri::Uri::from_path(&root_path).expect("failed to parse root URI");
        ret.wmsg_tx
            .send(WriterMessage::Message(jsonrpc::Message::new(
                jsonrpc::MessageContent::Call {
                    id: jsonrpc::Id::Num(0),
                    method: "initialize".to_owned(),
                    params: Some(
                        serde_json::to_value(types::InitializeParams {
                            processId: Some(process_id),
                            clientInfo: client_name.map(|name| types::ClientInfo {
                                name,
                                version: client_version,
                            }),
                            rootUri: Some(root_uri),
                            capabilities: types::ClientCapabilities {},
                            trace: None,
                        })
                        .unwrap(),
                    ),
                },
            )))
            .unwrap();

        Ok(ret)
    }

    pub fn send_command(&mut self, command: LanguageServerCommand) {
        unimplemented!()
    }
}

fn language_client_reader<B>(
    sync_state: Arc<Mutex<LanguageClientSyncState<B>>>,
    mut reader: Box<BufReader<ChildStdout>>,
    api_tx: Sender<(B, LanguageServerResponse)>,
) where
    B: BufferKey,
{
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
            if let Ok(raw_message) = serde_json::from_slice::<jsonrpc::MessageContent>(&content) {
                eprintln!(
                    "raw_message: {}",
                    serde_json::to_string_pretty(&raw_message).unwrap()
                );
            } else {
                break;
            }
        } else {
            break;
        }
    }
}

fn language_client_writer<B>(
    sync_state: Arc<Mutex<LanguageClientSyncState<B>>>,
    mut writer: Box<ChildStdin>,
    wmsg_rx: Receiver<WriterMessage>,
) where
    B: BufferKey,
{
    while let Ok(message) = wmsg_rx.recv() {
        match message {
            WriterMessage::Exit => {
                // TODO: Send exit message
                break;
            }
            WriterMessage::Message(message) => {
                if write!(&mut writer, "{}", message).is_err() {
                    break;
                }
            }
        }
    }
}

fn absolute_path(spath: &str) -> String {
    let path = std::path::Path::new(spath);
    if path.is_absolute() {
        spath.to_owned()
    } else if path.starts_with("~") {
        let mut home_dir = directories::BaseDirs::new()
            .expect("failed to get base directories")
            .home_dir()
            .to_owned();
        home_dir.push(path.strip_prefix("~").expect("failed to stip '~' prefix"));
        home_dir
            .to_str()
            .expect("failed to convert path to string")
            .to_owned()
    } else {
        let mut wdir = std::env::current_dir().expect("failed to get current directory");
        wdir.push(spath);
        wdir.to_str()
            .expect("failed to convert path to string")
            .to_owned()
    }
}
