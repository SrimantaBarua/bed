// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cmp::min;
use std::fmt;
use std::fmt::Write;
use std::ops::Drop;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};

use crossbeam_channel::{unbounded, Receiver, Sender};
use fnv::FnvHashMap;
use ropey::Rope;
use syntect::highlighting::{
    FontStyle, HighlightState, Highlighter, RangedHighlightIterator, ThemeSet,
};
use syntect::parsing::{ParseState, SyntaxSet};

use crate::style::{Color, TextSlant, TextStyle, TextWeight};

use super::view::StyledText;
use super::BufferID;

const PARSE_CACHE_DIFF: usize = 1000;

enum WorkerBroadcast {
    StartHL(
        Arc<(Mutex<Option<usize>>, Condvar)>,
        BufferID,
        Rope,
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
        syntax_set: Arc<SyntaxSet>,
        theme_set: Arc<ThemeSet>,
        theme: &str,
    ) -> Worker {
        let (wmtx, wmrx) = unbounded();
        let theme = theme.to_owned();
        Worker {
            wmtx: wmtx,
            handle: Some(thread::spawn(move || 'outer: loop {
                select! {
                    recv(brx) -> job => match job.unwrap() {
                        WorkerBroadcast::StartHL(cond, bid, data, parse_states_m, hl_states_m, styled_lines_m, mut linum) => {
                            {
                                let (lock, cvar) = &*cond;
                                let mut optwid = lock.lock().unwrap();
                                *optwid = Some(wid);
                                cvar.notify_one();
                            }

                            let mut hl_states = hl_states_m.lock().unwrap();
                            let mut parse_states = parse_states_m.lock().unwrap();
                            let i = min(min(linum / PARSE_CACHE_DIFF, hl_states.len() - 1), parse_states.len() - 1);
                            hl_states.truncate(i + 1);
                            parse_states.truncate(i + 1);
                            linum = i * PARSE_CACHE_DIFF;
                            let mut buf = String::new();
                            let hl = Highlighter::new(theme_set.themes.get(&theme).unwrap());
                            let mut hlstate = hl_states[i].clone();
                            let mut parse_state = parse_states[i].clone();
                            for line in data.lines_at(linum) {

                                match wmrx.try_recv() {
                                    Ok(WorkerMessage::StopHL(cond)) => {
                                        let (lock, cvar) = &*cond;
                                        let mut stopped = lock.lock().unwrap();
                                        *stopped = true;
                                        cvar.notify_one();
                                        continue 'outer;
                                    }
                                    Ok(WorkerMessage::Shutdown) => break 'outer,
                                    _ => {}
                                }

                                buf.clear();
                                write!(&mut buf, "{}", line).unwrap();
                                let mut styled = StyledText::new();

                                let ops = parse_state.parse_line(&buf, &syntax_set);
                                for (style, txt, _) in RangedHighlightIterator::new(&mut hlstate, &ops, &buf, &hl) {
                                    // TODO Background color
                                    let clr = Color::from_syntect(style.foreground);
                                    let mut ts = TextStyle::default();
                                    if style.font_style.contains(FontStyle::BOLD) {
                                        ts.weight = TextWeight::Bold;
                                    }
                                    if style.font_style.contains(FontStyle::ITALIC) {
                                        ts.slant = TextSlant::Italic;
                                    }
                                    let under = if style.font_style.contains(FontStyle::UNDERLINE) {
                                        Some(clr)
                                    } else {
                                        None
                                    };
                                    let ccount = txt.chars().count();
                                    styled.push(ccount, ts, clr, under);
                                }

                                if styled.is_empty() {
                                    styled.push(0, TextStyle::default(), Color::new(0, 0, 0, 0xff), None);
                                }
                                {
                                    let mut styled_lines = styled_lines_m.lock().unwrap();
                                    styled_lines[linum] = styled;
                                }
                                linum += 1;
                                if linum % PARSE_CACHE_DIFF == 0 {
                                    hl_states.push(hlstate.clone());
                                    parse_states.push(parse_state.clone());
                                }
                            }

                            mtx.send(ManagerMessage::HLDone(bid)).unwrap();
                        }
                    },
                    recv(wmrx) -> msg => match msg.unwrap() {
                        WorkerMessage::Shutdown => break,
                        WorkerMessage::StopHL(cond) => {
                            let (lock, cvar) = &*cond;
                            let mut stopped = lock.lock().unwrap(); *stopped = true;
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
    StartHL(
        BufferID,
        Rope,
        Arc<Mutex<Vec<ParseState>>>,
        Arc<Mutex<Vec<HighlightState>>>,
        Arc<Mutex<Vec<StyledText>>>,
        usize,
    ),
    StopHL(BufferID, Arc<(Mutex<bool>, Condvar)>),
    HLDone(BufferID),
}

impl fmt::Debug for ManagerMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ManagerMessage::Shutdown => write!(f, "shutdown"),
            ManagerMessage::StartHL(bid, _, _, _, _, _) => write!(f, "start_hl: {:?}", bid),
            ManagerMessage::StopHL(bid, _) => write!(f, "stop_hl: {:?}", bid),
            ManagerMessage::HLDone(bid) => write!(f, "hl_done: {:?}", bid),
        }
    }
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
                        break;
                    }
                    ManagerMessage::StartHL(bid, r, p, h, s, l) => {
                        assert!(
                            !hlmap.contains_key(&bid),
                            "duplicate highlighting task for buffer"
                        );
                        let cond = Arc::new((Mutex::new(None), Condvar::new()));
                        let cond2 = Arc::clone(&cond);
                        wbtx.send(WorkerBroadcast::StartHL(cond2, bid, r, p, h, s, l))
                            .unwrap();
                        let (lock, cvar) = &*cond;
                        let mut optwid = lock.lock().unwrap();
                        while optwid.is_none() {
                            optwid = cvar.wait(optwid).unwrap();
                        }
                        let wid = optwid.unwrap();
                        hlmap.insert(bid, wid);
                    }
                    ManagerMessage::StopHL(bid, cond) => {
                        if let Some(wid) = hlmap.remove(&bid) {
                            workers[wid].wmtx.send(WorkerMessage::StopHL(cond)).unwrap();
                        } else {
                            let (lock, cvar) = &*cond;
                            let mut stopped = lock.lock().unwrap();
                            *stopped = true;
                            cvar.notify_one();
                        }
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
        data: Rope,
        parse_states: Arc<Mutex<Vec<ParseState>>>,
        hl_states: Arc<Mutex<Vec<HighlightState>>>,
        styled_lines: Arc<Mutex<Vec<StyledText>>>,
        linum: usize,
    ) {
        self.mtx
            .send(ManagerMessage::StartHL(
                bid,
                data,
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
