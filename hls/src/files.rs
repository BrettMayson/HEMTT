use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use ropey::Rope;
use url::Url;

use crate::{TextDocumentItem, TextInformation};

#[derive(Clone)]
pub struct FileCache {
    ropes: Arc<DashMap<Url, Rope>>,
}

impl FileCache {
    pub fn get() -> Self {
        static SINGLETON: LazyLock<FileCache> = LazyLock::new(|| FileCache {
            ropes: Arc::new(DashMap::new()),
        });
        (*SINGLETON).clone()
    }

    pub fn text(&self, url: &Url) -> Option<String> {
        self.ropes.get(url).map(|r| r.value().to_string())
    }

    pub fn insert(&self, url: Url, text: Rope) {
        self.ropes.insert(url, text);
    }

    pub fn entry(&self, url: Url) -> dashmap::Entry<Url, Rope> {
        self.ropes.entry(url)
    }

    pub async fn on_change<'a>(&self, document: &TextDocumentItem<'a>) {
        match &document.text {
            TextInformation::Full(text) => {
                FileCache::get().insert(document.uri.clone(), Rope::from_str(text));
            }
            TextInformation::Changes(changes) => {
                let file_cache = FileCache::get();
                let mut rope = file_cache
                    .entry(document.uri.clone())
                    .or_insert_with(|| Rope::from_str(""));
                for change in changes {
                    if let Some(range) = change.range {
                        let start = rope.line_to_char(range.start.line as usize)
                            + range.start.character as usize;
                        let end = rope.line_to_char(range.end.line as usize)
                            + range.end.character as usize;
                        rope.remove(start..end);
                        rope.insert(start, change.text.as_str());
                    }
                }
            }
        }
    }
}
