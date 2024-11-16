use std::collections::BTreeMap;

use lsp_types::{Position, TextDocumentContentChangeEvent};

#[derive(Default)]
pub struct OpenFiles {
    files: BTreeMap<String, String>,
}

impl OpenFiles {
    fn load(&mut self, uri: &str) {
        let content = std::fs::read_to_string(uri).unwrap();
        self.add(uri.to_string(), content);
    }

    pub fn add(&mut self, uri: String, content: String) {
        self.files.insert(uri, content);
    }

    pub fn get(&mut self, uri: &str) -> &str {
        if !self.files.contains_key(uri) {
            self.load(uri)
        }
        self.files.get(uri).unwrap()
    }

    pub fn apply_changes(&mut self, uri: &str, changes: Vec<TextDocumentContentChangeEvent>) {
        let content = self.files.get_mut(uri).unwrap();
        for change in changes {
            if let Some(range) = change.range {
                let start = resolve_position(content, range.start);
                let end = resolve_position(content, range.end);
                assert!(
                    start <= end,
                    "start {:?} {}, end {:?} {} content len: {} {:?}",
                    range.start,
                    start,
                    range.end,
                    end,
                    content.len(),
                    content,
                );
                content.replace_range(start..end, &change.text);
            } else {
                // full content replace
                *content = change.text;
            }
        }
    }

    pub fn remove(&mut self, uri: &str) {
        self.files.remove(uri);
    }
}

fn resolve_position(content: &str, pos: Position) -> usize {
    let mut count = 0;
    let mut lines = 0;
    let mut character = 0;
    for c in content.chars() {
        if lines >= pos.line && character >= pos.character {
            break;
        }
        if c == '\n' {
            lines += 1;
            character = 0;
        } else {
            character += 1;
        }
        count += 1;
    }
    count
}

#[cfg(test)]
mod tests {
    use lsp_types::Range;

    use super::*;

    #[test]
    fn changes() {
        let mut files = OpenFiles::default();
        files.add("test".to_owned(), "".to_owned());
        files.apply_changes(
            "test",
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 0,
                    },
                }),
                range_length: None,
                text: "foo\n".to_owned(),
            }],
        );
        assert_eq!(files.get("test"), "foo\n");
        files.apply_changes(
            "test",
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 1,
                        character: 0,
                    },
                    end: Position {
                        line: 1,
                        character: 0,
                    },
                }),
                range_length: None,
                text: "bar".to_owned(),
            }],
        );
        assert_eq!(files.get("test"), "foo\nbar");
        files.apply_changes(
            "test",
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 1,
                        character: 0,
                    },
                    end: Position {
                        line: 1,
                        character: 0,
                    },
                }),
                range_length: None,
                text: "\n".to_owned(),
            }],
        );
        assert_eq!(files.get("test"), "foo\n\nbar");
    }
}
