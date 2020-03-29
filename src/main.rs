use lsp_server::{Connection, IoThreads};
use lsp_types::request::Request;
use std::io::BufRead;

fn server_capabilities() -> serde_json::Value {
    let mut cap = lsp_types::ServerCapabilities::default();
    cap.hover_provider = Some(true);

    serde_json::to_value(cap).unwrap()
}

fn connect() -> (Connection, IoThreads) {
    let (c, io) = Connection::stdio();
    let caps = server_capabilities();
    let _ = c.initialize(caps).unwrap();
    (c, io)
}

fn main() {
    let (c, io) = connect();

    let dict = Dict::new(std::path::PathBuf::from("dict.txt"));

    loop {
        match c.receiver.recv().unwrap() {
            lsp_server::Message::Request(r) => {
                if r.method == lsp_types::request::HoverRequest::METHOD {
                    let tdp =
                        serde_json::from_value::<lsp_types::TextDocumentPositionParams>(r.params)
                            .unwrap();

                    let response = match get_word(tdp) {
                        Some(w) => {
                            let text = dict.info(&w);
                            let resp = lsp_types::Hover {
                                contents: lsp_types::HoverContents::Markup(
                                    lsp_types::MarkupContent {
                                        kind: lsp_types::MarkupKind::Markdown,
                                        value: text,
                                    },
                                ),
                                range: None,
                            };
                            lsp_server::Message::Response(lsp_server::Response {
                                id: r.id,
                                result: Some(serde_json::to_value(resp).unwrap()),
                                error: None,
                            })
                        }
                        None => lsp_server::Message::Response(lsp_server::Response {
                            id: r.id,
                            result: None,
                            error: None,
                        }),
                    };

                    c.sender.send(response).unwrap()
                } else {
                    panic!("{:?}", r.method)
                }
            }
            lsp_server::Message::Response(r) => panic!("{:?}", r),
            lsp_server::Message::Notification(n) => panic!("{:?}", n),
        }
    }

    io.join().unwrap()
}

struct Dict {
    file: std::path::PathBuf,
}

#[derive(Default)]
struct DictItem {
    word: String,
    kind: String,
    def: String,
}

impl DictItem {
    fn from(l: &str) -> Self {
        let parts: Vec<_> = l.split_whitespace().collect();
        let mut di = DictItem::default();
        if parts.len() > 2 {
            di.word = parts[0].to_lowercase();
            di.kind = parts[1].to_lowercase();
            di.def = parts[2..].join(" ");
        }
        di
    }

    fn render(&self) -> String {
        format!("# {}\n\n_{}_\n\n{}", self.word, self.kind, self.def)
    }
}

impl Dict {
    fn new(f: std::path::PathBuf) -> Self {
        Self { file: f }
    }

    fn info(&self, word: &str) -> String {
        let file = std::fs::File::open(&self.file).unwrap();
        let reader = std::io::BufReader::new(file);
        for l in reader.lines() {
            let l = l.unwrap();
            if l.is_empty() {
                continue;
            }
            let di = DictItem::from(&l);
            if di.word == word.to_lowercase() {
                return di.render();
            }
        }
        "".to_string()
    }
}

fn get_word(tdp: lsp_types::TextDocumentPositionParams) -> Option<String> {
    let file = std::fs::File::open(tdp.text_document.uri.to_file_path().unwrap()).unwrap();
    let reader = std::io::BufReader::new(file);
    let line = reader
        .lines()
        .nth(tdp.position.line as usize)
        .unwrap()
        .unwrap();
    let words: Vec<_> = line.split_whitespace().collect();
    let mut x = 0;
    for w in words {
        if x <= tdp.position.character as usize && x + w.len() > tdp.position.character as usize {
            return Some(w.to_string());
        }
        x += w.len() + 1;
    }
    return None;
}
