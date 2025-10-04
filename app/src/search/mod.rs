use libcommon::prelude::{debug, trace};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, LazyLock, RwLock};

static SEARCH: LazyLock<SearchEngine> = LazyLock::new(Default::default);

pub fn search(word: Option<String>) -> Vec<SearchItem> {
    trace!("searching: {word:?}");
    if let Some(s) = word
        && !s.is_empty()
    {
        return SEARCH.search(&s);
    }
    SEARCH.collect_default()
}

pub fn on_update(items: Vec<SearchItem>) {
    debug!("updating search items {}", items.len());
    if let Ok(mut data) = SEARCH.data.write() {
        *data = items;
    }
}

#[derive(Debug)]
struct SearchEngine {
    sys: Arc<Vec<SearchItem>>,
    data: Arc<RwLock<Vec<SearchItem>>>,
}

impl SearchEngine {
    fn collect_default(&self) -> Vec<SearchItem> {
        self.sys.clone().to_vec()
    }

    fn search(&self, _q: &str) -> Vec<SearchItem> {
        vec![]
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        let sys = vec![
            SearchItem::new("sys:scan", "Scan", None),
            SearchItem::new("sys:exit", "Exit", None),
        ];
        Self {
            sys: Arc::new(sys),
            data: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchItem {
    id: String,
    name: String,
    words: (String, Option<String>),
}

impl SearchItem {
    pub fn new(id: impl ToString, name: impl ToString, keywords: Option<String>) -> Self {
        let id = id.to_string();
        let name = name.to_string();
        let words = (name.clone(), keywords);
        Self { id, name, words }
    }
}
