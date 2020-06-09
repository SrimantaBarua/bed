// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::path::Path;
use std::rc::Rc;

use directories::ProjectDirs;
use fnv::FnvHashMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum VersionControl {
    Git,
}

#[derive(Deserialize, Serialize)]
struct ProjectInner {
    vcs: Option<VersionControl>,
    indent_tabs: Option<bool>,
    tab_width: Option<usize>,
}

pub(crate) struct Project {
    pub(crate) root: String,
    pub(crate) vcs: Option<VersionControl>,
    pub(crate) indent_tabs: Option<bool>,
    pub(crate) tab_width: Option<usize>,
}

impl Project {
    fn new(root: String, inner: ProjectInner) -> Project {
        Project {
            root,
            vcs: inner.vcs,
            indent_tabs: inner.indent_tabs,
            tab_width: inner.tab_width,
        }
    }
}

#[derive(Default)]
pub(crate) struct Projects(FnvHashMap<String, Rc<Project>>);

impl Projects {
    pub(crate) fn load() -> Projects {
        if let Some(proj_dirs) = ProjectDirs::from("", "sbarua", "bed") {
            // Try loading config
            let cfg_dir_path = proj_dirs.config_dir();
            std::fs::read_to_string(cfg_dir_path.join("projects.json"))
                .ok()
                .and_then(|data| serde_json::from_str(&data).ok())
                .map(|inner: FnvHashMap<String, ProjectInner>| {
                    let mut ret = FnvHashMap::default();
                    for (k, v) in inner {
                        let root = k.clone();
                        ret.insert(k, Rc::new(Project::new(root, v)));
                    }
                    Projects(ret)
                })
                .unwrap_or_default()
        } else {
            Projects::default()
        }
    }

    pub(crate) fn project_for_path(&self, path: &str) -> Option<Rc<Project>> {
        for ancestor in Path::new(path).ancestors().filter_map(|path| path.to_str()) {
            if let Some(project) = self.0.get(ancestor) {
                return Some(project.clone());
            }
        }
        None
    }
}
