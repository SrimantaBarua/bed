// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cell::RefCell;
use std::fs::read_dir;
use std::io::Result as IOResult;
use std::ops::Drop;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::rc::Rc;

use fnv::FnvHashMap;

use crate::common::abspath;
use crate::config::Config;
use crate::language::Language;

mod proto;

pub(crate) struct LangClientMgr {
    clients: FnvHashMap<(String, Language), Rc<RefCell<LangClient>>>,
    config: Rc<Config>,
}

impl LangClientMgr {
    pub(crate) fn new(config: Rc<Config>) -> LangClientMgr {
        LangClientMgr {
            clients: FnvHashMap::default(),
            config,
        }
    }

    pub(crate) fn get_client(
        &mut self,
        file_path: &str,
        language: Language,
    ) -> Option<IOResult<Rc<RefCell<LangClient>>>> {
        let config = self.config.clone();
        config
            .language
            .get(&language)
            .and_then(|lconf| lconf.language_server.as_ref())
            .and_then(|lsconf| {
                let abspath = abspath(file_path);
                let path = Path::new(&abspath);
                path.parent().and_then(|dirpath| {
                    let mut root_path = dirpath;
                    'outer: for path in dirpath.ancestors() {
                        if let Ok(readdir) = read_dir(path) {
                            for entry in readdir.filter_map(|e| e.ok()) {
                                if let Some(child) = entry.path().to_str() {
                                    for marker in config
                                        .completion_langserver_root_markers
                                        .iter()
                                        .chain(&lsconf.root_markers)
                                    {
                                        if child == marker {
                                            root_path = path;
                                            break 'outer;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    root_path.to_str().and_then(|path| {
                        let path = path.to_owned();
                        if let Some(lc) = self.clients.get(&(path.clone(), language)) {
                            Some(Ok(lc.clone()))
                        } else {
                            if let Ok(lc) = LangClient::new(&lsconf.executable, &lsconf.arguments) {
                                let lc = Rc::new(RefCell::new(lc));
                                self.clients.insert((path, language), lc.clone());
                                Some(Ok(lc))
                            } else {
                                None
                            }
                        }
                    })
                })
            })
    }
}

pub(crate) struct LangClient {
    child: Child,
}

impl LangClient {
    fn new(command: &str, args: &[String]) -> IOResult<LangClient> {
        let child = Command::new(command)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .args(args)
            .spawn()?;
        Ok(LangClient { child })
    }
}

impl Drop for LangClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}
