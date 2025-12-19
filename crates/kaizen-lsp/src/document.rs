use dashmap::DashMap;
use kaizen_core::parser::ParsedFile;
use tower_lsp::lsp_types::Url;

pub struct DocumentStore {
    documents: DashMap<Url, ParsedFile>,
}

impl DocumentStore {
    pub fn new() -> Self {
        Self {
            documents: DashMap::new(),
        }
    }

    pub fn open(&self, uri: Url, content: &str) {
        let filename = uri_to_filename(&uri);
        let parsed = ParsedFile::from_source(&filename, content);
        self.documents.insert(uri, parsed);
    }

    pub fn update(&self, uri: &Url, content: &str) {
        let filename = uri_to_filename(uri);
        let parsed = ParsedFile::from_source(&filename, content);
        self.documents.insert(uri.clone(), parsed);
    }

    pub fn close(&self, uri: &Url) {
        self.documents.remove(uri);
    }

    #[allow(dead_code)]
    pub fn get(&self, uri: &Url) -> Option<dashmap::mapref::one::Ref<'_, Url, ParsedFile>> {
        self.documents.get(uri)
    }

    #[allow(dead_code)]
    pub fn contains(&self, uri: &Url) -> bool {
        self.documents.contains_key(uri)
    }
}

impl Default for DocumentStore {
    fn default() -> Self {
        Self::new()
    }
}

fn uri_to_filename(uri: &Url) -> String {
    uri.path().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_uri(filename: &str) -> Url {
        Url::parse(&format!("file:///test/{}", filename)).unwrap()
    }

    #[test]
    fn did_open_stores_document() {
        let store = DocumentStore::new();
        let uri = test_uri("test.js");
        let content = "const x = 1;";

        store.open(uri.clone(), content);

        assert!(store.contains(&uri));
        let doc = store.get(&uri).unwrap();
        assert_eq!(doc.source(), content);
    }

    #[test]
    fn did_open_parses_document() {
        let store = DocumentStore::new();
        let uri = test_uri("test.js");
        let content = "const x = 1;";

        store.open(uri.clone(), content);

        let doc = store.get(&uri).unwrap();
        assert!(doc.module().is_some());
        assert!(!doc.metadata().has_errors);
    }

    #[test]
    fn did_open_detects_language() {
        let store = DocumentStore::new();
        let js_uri = test_uri("test.js");
        let ts_uri = test_uri("test.ts");
        let jsx_uri = test_uri("test.jsx");
        let tsx_uri = test_uri("test.tsx");

        store.open(js_uri.clone(), "const x = 1;");
        store.open(ts_uri.clone(), "const x: number = 1;");
        store.open(jsx_uri.clone(), "const x = <div />;");
        store.open(tsx_uri.clone(), "const x: JSX.Element = <div />;");

        assert_eq!(
            store.get(&js_uri).unwrap().metadata().language,
            kaizen_core::parser::Language::JavaScript
        );
        assert_eq!(
            store.get(&ts_uri).unwrap().metadata().language,
            kaizen_core::parser::Language::TypeScript
        );
        assert_eq!(
            store.get(&jsx_uri).unwrap().metadata().language,
            kaizen_core::parser::Language::Jsx
        );
        assert_eq!(
            store.get(&tsx_uri).unwrap().metadata().language,
            kaizen_core::parser::Language::Tsx
        );
    }

    #[test]
    fn did_change_updates_content() {
        let store = DocumentStore::new();
        let uri = test_uri("test.js");
        let initial_content = "const x = 1;";
        let updated_content = "const x = 2;\nconst y = 3;";

        store.open(uri.clone(), initial_content);
        store.update(&uri, updated_content);

        let doc = store.get(&uri).unwrap();
        assert_eq!(doc.source(), updated_content);
        assert_eq!(doc.metadata().line_count, 2);
    }

    #[test]
    fn did_change_reparses_document() {
        let store = DocumentStore::new();
        let uri = test_uri("test.js");
        let initial_content = "const x = 1;";
        let updated_content = "function foo() { return 42; }";

        store.open(uri.clone(), initial_content);
        store.update(&uri, updated_content);

        let doc = store.get(&uri).unwrap();
        let module = doc.module().unwrap();
        assert_eq!(module.body.len(), 1);
    }

    #[test]
    fn did_close_removes_document() {
        let store = DocumentStore::new();
        let uri = test_uri("test.js");
        let content = "const x = 1;";

        store.open(uri.clone(), content);
        assert!(store.contains(&uri));

        store.close(&uri);
        assert!(!store.contains(&uri));
    }

    #[test]
    fn get_returns_none_for_unknown_document() {
        let store = DocumentStore::new();
        let uri = test_uri("unknown.js");

        assert!(store.get(&uri).is_none());
    }

    #[test]
    fn document_store_handles_parse_errors() {
        let store = DocumentStore::new();
        let uri = test_uri("test.js");
        let invalid_content = "const = ;";

        store.open(uri.clone(), invalid_content);

        let doc = store.get(&uri).unwrap();
        assert!(doc.metadata().has_errors);
        assert!(!doc.errors().is_empty());
    }

    #[test]
    fn update_nonexistent_document_creates_it() {
        let store = DocumentStore::new();
        let uri = test_uri("test.js");
        let content = "const x = 1;";

        store.update(&uri, content);

        assert!(store.contains(&uri));
        let doc = store.get(&uri).unwrap();
        assert_eq!(doc.source(), content);
    }

    #[test]
    fn document_store_is_thread_safe() {
        use std::sync::Arc;
        use std::thread;

        let store = Arc::new(DocumentStore::new());
        let mut handles = vec![];

        for i in 0..10 {
            let store_clone = Arc::clone(&store);
            let handle = thread::spawn(move || {
                let uri = Url::parse(&format!("file:///test/file{}.js", i)).unwrap();
                let content = format!("const x{} = {};", i, i);
                store_clone.open(uri.clone(), &content);
                assert!(store_clone.contains(&uri));
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        for i in 0..10 {
            let uri = Url::parse(&format!("file:///test/file{}.js", i)).unwrap();
            assert!(store.contains(&uri));
        }
    }
}
