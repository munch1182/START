use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use libcommon::prelude::debug;
use parking_lot::RwLock;
use pinyin::ToPinyin;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, LazyLock};

static SEARCH: LazyLock<SearchEngine> = LazyLock::new(Default::default);
static FUZZY_SEARCH: LazyLock<SkimMatcherV2> = LazyLock::new(SkimMatcherV2::default);
const COUNT_LIMIT: usize = 7;

pub fn search(word: Option<String>) -> Vec<SearchItem> {
    let mut res = if let Some(s) = word
        && !s.is_empty()
    {
        SEARCH.search(&s)
    } else {
        SEARCH.collect_default()
    };
    res.truncate(COUNT_LIMIT);
    res
}

pub fn on_update(items: Vec<SearchItem>) {
    debug!("updating search items {}", items.len());
    {
        *SEARCH.data.write() = items;
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

    fn search(&self, q: &str) -> Vec<SearchItem> {
        let q = to_pinyin(q);
        let mut res = vec![];
        res.extend(self.search_sys(&q));
        res.extend(self.search_data(&q));
        res
    }

    fn search_data(&self, q: &str) -> Vec<SearchItem> {
        fuzzy_search(q, self.data.read().to_vec())
    }

    fn search_sys(&self, q: &str) -> Vec<SearchItem> {
        fuzzy_search(q, self.sys.to_vec())
    }
}

fn fuzzy_search(q: &str, data: Vec<SearchItem>) -> Vec<SearchItem> {
    let mut result = data
        .iter()
        .filter_map(|item| {
            FUZZY_SEARCH
                .fuzzy_match(&item.as_search, q)
                .map(|score| (score, item.clone()))
        })
        .collect::<Vec<_>>();
    result.sort_by(|a, b| b.0.cmp(&a.0));
    result.into_iter().map(|(_, item)| item).collect()
}

impl Default for SearchEngine {
    fn default() -> Self {
        let sys = vec![
            SearchItem::new("sys:scan", "Scan", None),
            SearchItem::new("sys:debug", "Debug", None),
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
    #[serde(skip_serializing_if = "Option::is_none")]
    words: Option<String>,
    /// 搜索词，合并`name`和`words`，用以实际使用
    as_search: String,
}

impl SearchItem {
    pub fn new(id: impl ToString, name: impl ToString, words: Option<String>) -> Self {
        let id = id.to_string();
        let name = name.to_string();
        let as_search = match &words {
            Some(s) => format!("{name} {s}"),
            None => name.clone(),
        };
        Self {
            id,
            name,
            words,
            as_search: to_pinyin(as_search),
        }
    }
}

fn to_pinyin(s: impl ToString) -> String {
    s.to_string()
        .chars()
        .flat_map(|c| match c.to_pinyin() {
            Some(s) => s.plain().chars().collect::<Vec<_>>(),
            None => vec![c],
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::search::{SearchEngine, SearchItem, to_pinyin};

    #[test]
    fn test_search() {
        let candidates = vec!["apple", "anpai", "排列", "安排", "an'pao", "anppai"];
        let input = to_pinyin("安排");
        let search = SearchEngine::default();
        search
            .data
            .write()
            .extend(candidates.iter().map(|s| SearchItem::new(s, s, None)));
        let results = search.search(&input);

        for r in results {
            println!("{} {}", r.name, r.words.unwrap_or_default());
        }
    }
}
