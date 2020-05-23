// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::cmp::min;
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

pub(super) const PARSE_CACHE_DIFF: usize = 500;

enum WorkerBroadcast {
    StartHL(
        Arc<(Mutex<Option<usize>>, Condvar)>,
        BufferID,
        Rope,
        Arc<Mutex<Vec<ParseState>>>,
        Arc<Mutex<Vec<HighlightState>>>,
        Arc<Mutex<Vec<StyledText>>>,
        usize,
        usize,
    ),
}

enum WorkerMessage {
    StopHL(Arc<(Mutex<bool>, Condvar)>),
    Shutdown,
}

enum WorkerReply {
    HLDone(BufferID),
}

struct Worker {
    wmtx: Sender<WorkerMessage>,
    handle: Option<JoinHandle<()>>,
    hl_ckpt: Arc<Mutex<usize>>,
}

impl Worker {
    fn new(
        wid: usize,
        brx: Receiver<WorkerBroadcast>,
        rtx: Sender<WorkerReply>,
        syntax_set: Arc<SyntaxSet>,
        theme_set: Arc<ThemeSet>,
        theme: &str,
    ) -> Worker {
        let (wmtx, wmrx) = unbounded();
        let theme = theme.to_owned();
        let hl_ckpt = Arc::new(Mutex::new(0));
        let hl_ckpt2 = Arc::clone(&hl_ckpt);

        let handle = thread::spawn(move || 'mainloop: loop {
            select! {
                recv(brx) -> job => match job.unwrap() {
                    WorkerBroadcast::StartHL(cond, bid, data, pssm, hssm, slsm, mut ln, mut su) => {
                        assert!(su >= ln);
                        if su > data.len_lines() {
                            su = data.len_lines();
                        }

                        let mut hss = hssm.lock().unwrap();
                        let mut pss = pssm.lock().unwrap();
                        let i = min(min(ln / PARSE_CACHE_DIFF, hss.len() - 1), pss.len() - 1);
                        hss.truncate(i + 1);
                        pss.truncate(i + 1);
                        ln = i * PARSE_CACHE_DIFF;
                        let mut buf = String::new();
                        let hl = Highlighter::new(theme_set.themes.get(&theme).unwrap());
                        let mut hs = hss[i].clone();
                        let mut ps = pss[i].clone();

                        // Sync
                        {
                            let mut sls = slsm.lock().unwrap();
                            for line in data.lines_at(ln) {
                                buf.clear();
                                write!(&mut buf, "{}", line).unwrap();
                                let mut styled = StyledText::new();

                                let ops = ps.parse_line(&buf, &syntax_set);
                                for (style, txt, _) in RangedHighlightIterator::new(&mut hs, &ops, &buf, &hl) {
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
                                    sls[ln] = styled;
                                ln += 1;
                                if ln % PARSE_CACHE_DIFF == 0 {
                                    hss.push(hs.clone());
                                    pss.push(ps.clone());
                                }
                                if ln >= su {
                                    break;
                                }
                            }
                        }

                        // Denote sync end
                        {
                            let (lock, cvar) = &*cond;
                            let mut optwid = lock.lock().unwrap();
                            *optwid = Some(wid);
                            cvar.notify_one();
                        }
                        {
                            let mut hl_ckpt = hl_ckpt.lock().unwrap();
                            *hl_ckpt = ln;
                        }

                        // Async
                        for line in data.lines_at(ln) {
                            match wmrx.try_recv() {
                                Ok(WorkerMessage::StopHL(cond)) => {
                                    let (lock, cvar) = &*cond;
                                    let mut stopped = lock.lock().unwrap();
                                    *stopped = true;
                                    cvar.notify_one();
                                    continue 'mainloop;
                                }
                                Ok(WorkerMessage::Shutdown) => break 'mainloop,
                                Err(_) => {}
                            }

                            buf.clear();
                            write!(&mut buf, "{}", line).unwrap();
                            let mut styled = StyledText::new();

                            let ops = ps.parse_line(&buf, &syntax_set);
                            for (style, txt, _) in RangedHighlightIterator::new(&mut hs, &ops, &buf, &hl) {
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
                                let mut sls = slsm.lock().unwrap();
                                sls[ln] = styled;
                            }
                            ln += 1;
                            if ln % PARSE_CACHE_DIFF == 0 {
                                hss.push(hs.clone());
                                pss.push(ps.clone());
                            }

                            {
                                let mut hl_ckpt = hl_ckpt.lock().unwrap();
                                *hl_ckpt = ln;
                            }
                        }

                        rtx.send(WorkerReply::HLDone(bid)).unwrap();
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
        });

        Worker {
            wmtx: wmtx,
            handle: Some(handle),
            hl_ckpt: hl_ckpt2,
        }
    }
}

pub(super) struct HlPool {
    workers: Vec<Worker>,
    hlmap: FnvHashMap<BufferID, usize>,
    wbtx: Sender<WorkerBroadcast>,
    wrrx: Receiver<WorkerReply>,
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
        let (wrtx, wrrx) = unbounded();
        for i in 0..num_workers {
            workers.push(Worker::new(
                i,
                wbrx.clone(),
                wrtx.clone(),
                Arc::clone(&ss),
                Arc::clone(&ts),
                theme,
            ));
        }
        HlPool {
            workers,
            hlmap: FnvHashMap::default(),
            wbtx,
            wrrx,
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
        sync_upto: usize,
    ) {
        self.handle_messages();
        assert!(
            !self.hlmap.contains_key(&bid),
            "duplicate highlighting task for buffer"
        );
        let cond = Arc::new((Mutex::new(None), Condvar::new()));
        let cond2 = Arc::clone(&cond);
        self.wbtx
            .send(WorkerBroadcast::StartHL(
                cond2,
                bid,
                data,
                parse_states,
                hl_states,
                styled_lines,
                linum,
                sync_upto,
            ))
            .unwrap();
        let (lock, cvar) = &*cond;
        let mut optwid = lock.lock().unwrap();
        while optwid.is_none() {
            optwid = cvar.wait(optwid).unwrap();
        }
        let wid = optwid.unwrap();
        self.hlmap.insert(bid, wid);
    }

    pub(super) fn stop_highlight(&mut self, bufid: BufferID) {
        self.handle_messages();
        if let Some(wid) = self.hlmap.remove(&bufid) {
            let cond = Arc::new((Mutex::new(false), Condvar::new()));
            let cond2 = Arc::clone(&cond);

            self.workers[wid]
                .wmtx
                .send(WorkerMessage::StopHL(cond2))
                .unwrap();

            let (lock, cvar) = &*cond;
            let mut stopped = lock.lock().unwrap();
            while !*stopped {
                stopped = cvar.wait(stopped).unwrap();
            }
        }
    }

    pub(super) fn highlight_checkpoint(&self, bufid: BufferID) -> Option<usize> {
        self.hlmap
            .get(&bufid)
            .map(|wid| *self.workers[*wid].hl_ckpt.lock().unwrap())
    }

    fn handle_messages(&mut self) {
        while let Ok(msg) = self.wrrx.try_recv() {
            match msg {
                WorkerReply::HLDone(bid) => {
                    self.hlmap.remove(&bid);
                }
            }
        }
    }
}

impl Drop for HlPool {
    fn drop(&mut self) {
        for w in &mut self.workers {
            w.wmtx.send(WorkerMessage::Shutdown).unwrap();
        }
        for w in &mut self.workers {
            if let Some(handle) = w.handle.take() {
                handle.join().unwrap();
            }
        }
    }
}
