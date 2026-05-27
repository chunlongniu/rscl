use std::collections::HashMap;
use tower_lsp_server::lsp_types::Uri;

pub struct DocumentStore {
    docs: HashMap<String, String>,
}

impl DocumentStore {
    pub fn new() -> Self {
        Self {
            docs: HashMap::new(),
        }
    }

    pub fn open(&mut self, uri: Uri, text: String) {
        self.docs.insert(uri.to_string(), text);
    }

    pub fn change(&mut self, uri: Uri, text: String) {
        self.docs.insert(uri.to_string(), text);
    }

    pub fn close(&mut self, uri: &Uri) {
        self.docs.remove(&uri.to_string());
    }

    pub fn get(&self, uri: &Uri) -> Option<&String> {
        self.docs.get(&uri.to_string())
    }
}
