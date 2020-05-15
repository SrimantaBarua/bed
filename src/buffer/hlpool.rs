// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::ops::Drop;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};

use crossbeam_channel::{unbounded, Receiver, Sender};
use fnv::FnvHashMap;
use syntect::highlighting::{HighlightState, ThemeSet};
use syntect::parsing::{ParseState, SyntaxSet};

use super::view::StyledText;
use super::BufferID;

enum WorkerBroadcast {
    StartHL(
        BufferID,
        Arc<Mutex<Vec<ParseState>>>,
        Arc<Mutex<Vec<HighlightState>>>,
        Arc<Mutex<Vec<StyledText>>>,
        usize,
    ),
}

enum WorkerMessage {
    StopHL(Arc<(Mutex<bool>, Condvar)>),
    Shutdown,
}

struct Worker {
    wmtx: Sender<WorkerMessage>,
    handle: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(
        wid: usize,
        brx: Receiver<WorkerBroadcast>,
        mtx: Sender<ManagerMessage>,
        ss: Arc<SyntaxSet>,
        ts: Arc<ThemeSet>,
        theme: &str,
    ) -> Worker {
        let (wmtx, wmrx) = unbounded();
        let theme = theme.to_owned();
        Worker {
            wmtx: wmtx,
            handle: Some(thread::spawn(move || loop {
                select! {
                    recv(brx) -> job => match job.unwrap() {
                        WorkerBroadcast::StartHL(bid, ps, hs, sl, ln) => {
                            mtx.send(ManagerMessage::HLStarted(bid, wid)).unwrap();
                        }
                        mtx.send(ManagerMessage::HLDone(bid)).unwrap();
                    },
                    recv(wmrx) -> msg => match msg.unwrap() {
                        WorkerMessage::Shutdown => break,
                        WorkerMessage::StopHL(cond) => {
                            let (lock, cvar) = &*cond;
                            let mut stopped = lock.lock().unwrap();
                            *stopped = true;
                            cvar.notify_one();
                        }
                    }
                }
            })),
        }
    }
}

enum ManagerMessage {
    Shutdown,
    StartHL(BufferID),
    StopHL(BufferID, Arc<(Mutex<bool>, Condvar)>),
    HLStarted(BufferID, usize),
    HLDone(BufferID),
}

pub(super) struct HlPool {
    manager: Option<JoinHandle<()>>,
    mtx: Sender<ManagerMessage>,
}

impl HlPool {
    pub(super) fn new(
        ss: Arc<SyntaxSet>,
        ts: Arc<ThemeSet>,
        theme: &str,
        num_workers: usize,
    ) -> HlPool {
        let mut workers = Vec::new();
        let (wbtx, wbrx) = unbounded();
        let (mtx, mrx) = unbounded();
        for i in 0..num_workers {
            workers.push(Worker::new(
                i,
                wbrx.clone(),
                mtx.clone(),
                Arc::clone(&ss),
                Arc::clone(&ts),
                theme,
            ));
        }

        let manager = thread::spawn(move || {
            let mut hlmap: FnvHashMap<BufferID, usize> = FnvHashMap::default();
            while let Ok(msg) = mrx.recv() {
                match msg {
                    ManagerMessage::Shutdown => {
                        for w in &mut workers {
                            w.wmtx.send(WorkerMessage::Shutdown).unwrap();
                        }
                        for w in &mut workers {
                            if let Some(handle) = w.handle.take() {
                                handle.join().unwrap();
                            }
                        }
                    }
                    ManagerMessage::StartHL(bid) => {
                        wbtx.send(WorkerBroadcast::StartHL(bid)).unwrap()
                    }
                    ManagerMessage::StopHL(bid, cond) => {
                        if let Some(wid) = hlmap.remove(&bid) {
                            workers[wid].wmtx.send(WorkerMessage::StopHL(cond)).unwrap();
                        }
                    }
                    ManagerMessage::HLStarted(bid, wid) => {
                        hlmap.insert(bid, wid);
                    }
                    ManagerMessage::HLDone(bid) => {
                        hlmap.remove(&bid);
                    }
                }
            }
        });

        HlPool {
            mtx: mtx,
            manager: Some(manager),
        }
    }

    pub(super) fn start_highlight(
        &mut self,
        bid: BufferID,
        parse_states: Arc<Mutex<Vec<ParseState>>>,
        hl_states: Arc<Mutex<Vec<HighlightState>>>,
        styled_lines: Arc<Mutex<Vec<StyledText>>>,
        linum: usize,
    ) {
        self.mtx
            .send(ManagerMessage::StartHL(
                bid,
                parse_states,
                hl_states,
                styled_lines,
                linum,
            ))
            .unwrap();
    }

    pub(super) fn stop_highlight(&mut self, bufid: BufferID) {
        let cond = Arc::new((Mutex::new(false), Condvar::new()));
        let cond2 = Arc::clone(&cond);
        self.mtx.send(ManagerMessage::StopHL(bufid, cond2)).unwrap();
        let (lock, cvar) = &*cond;
        let mut stopped = lock.lock().unwrap();
        while !*stopped {
            stopped = cvar.wait(stopped).unwrap();
        }
    }
}

impl Drop for HlPool {
    fn drop(&mut self) {
        self.mtx.send(ManagerMessage::Shutdown).unwrap();
        if let Some(m) = self.manager.take() {
            m.join().unwrap();
        }
    }
}
