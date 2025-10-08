use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use libcommon::{Default_With, With, prelude::debug};
use parking_lot::RwLock;
use pinyin::ToPinyin;
use plugin_d::Launcher;
use plugin_manager::PluginInfoWrapper;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, LazyLock};

static SEARCH: LazyLock<SearchEngine> = LazyLock::new(Default::default);
static FUZZY_SEARCH: LazyLock<SkimMatcherV2> = LazyLock::new(SkimMatcherV2::default);
const COUNT_LIMIT: usize = 7;

pub fn search(word: Option<String>) -> Vec<SearchResultItem> {
    let mut res = if let Some(s) = word
        && !s.is_empty()
    {
        SEARCH.search(&s)
    } else {
        SEARCH.collect_default()
    };
    res.truncate(COUNT_LIMIT);
    res.iter().map(From::from).collect()
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
            if item.search.is_full_name && item.name.contains(q) {
                return Some((100, item.clone()));
            }
            FUZZY_SEARCH
                .fuzzy_match(&item.search.as_search, q)
                .map(|score| (score, item.clone()))
        })
        .collect::<Vec<_>>();
    result.sort_by(|a, b| b.0.cmp(&a.0));
    result.into_iter().map(|(_, item)| item).collect()
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self {
            sys: Arc::new(vec![]),
            data: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, With)]
pub struct SearchResultItem {
    name: String,
    path: String,
    page_type: Launcher,
}

impl From<&SearchItem> for SearchResultItem {
    fn from(value: &SearchItem) -> Self {
        Self {
            name: value.name.clone(),
            path: value.execute.path.clone(),
            page_type: value.execute.launcher,
        }
    }
}

#[derive(Debug, Clone, With)]
pub struct SearchItem {
    id: String,
    name: String,
    search: Search,
    execute: Execute,
}

#[derive(Debug, Clone, With)]
pub struct Execute {
    launcher: Launcher,
    path: String,
}

#[derive(Debug, Clone, With, Default_With)]
pub struct Search {
    /// 搜索词，合并`name`和`words`，用以实际使用
    #[default_with(no_default)]
    as_search: String,
    /// 是否是全名搜索，即输入文本至少要完整包含前一半的字符
    is_full_name: bool,
}

impl From<PluginInfoWrapper> for SearchItem {
    fn from(value: PluginInfoWrapper) -> Self {
        let id = value.id;
        let name = value.info.name;
        let luncher = value.info.luncher;
        let as_search = format!(
            "{} {}",
            to_pinyin(name.clone()),
            value.info.keyword.unwrap_or(String::default())
        )
        .trim()
        .to_string();
        let search = Search {
            as_search,
            is_full_name: false,
        };
        let execute = Execute {
            launcher: luncher,
            path: value.info_net.unwrap_or_default().html, // ?
        };
        Self {
            id,
            name,
            search,
            execute,
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
