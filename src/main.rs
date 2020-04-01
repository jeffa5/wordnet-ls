use lls_lib::wordnet;
use lls_lib::wordnet::WordNet;
use lsp_server::{Connection, IoThreads};
use lsp_types::request::Request;
use std::collections::HashMap;
use std::io::BufRead;
use std::path::Path;
use std::path::PathBuf;

fn server_capabilities() -> serde_json::Value {
    let mut cap = lsp_types::ServerCapabilities::default();
    cap.hover_provider = Some(true);

    serde_json::to_value(cap).unwrap()
}

fn connect() -> (lsp_types::InitializeParams, Connection, IoThreads) {
    let (c, io) = Connection::stdio();
    let caps = server_capabilities();
    let params = c.initialize(caps).unwrap();
    let params: lsp_types::InitializeParams = serde_json::from_value(params).unwrap();
    (params, c, io)
}

struct Server {
    dict: Dict,
    shutdown: bool,
}

impl Server {
    fn new(params: lsp_types::InitializeParams) -> Self {
        let default_wordnet = PathBuf::from("wordnet");
        let wordnet_location = match params.initialization_options {
            None => default_wordnet,
            Some(l) => match serde_json::from_value::<HashMap<String, String>>(l) {
                Ok(v) => match v.get("wordnet") {
                    None => default_wordnet,
                    Some(l) => {
                        if l.starts_with("~/") {
                            dirs::home_dir().unwrap().join(l.trim_start_matches("~/"))
                        } else {
                            PathBuf::from(l)
                        }
                    }
                },
                Err(_) => default_wordnet,
            },
        };
        Self {
            dict: Dict::new(&wordnet_location),
            shutdown: false,
        }
    }

    fn serve(mut self, c: Connection, io: IoThreads) -> Result<(), String> {
        loop {
            match c.receiver.recv().unwrap() {
                lsp_server::Message::Request(r) => {
                    if self.shutdown {
                        c.sender
                            .send(lsp_server::Message::Response(lsp_server::Response {
                                id: r.id,
                                result: None,
                                error: Some(lsp_server::ResponseError {
                                    code: lsp_server::ErrorCode::InvalidRequest as i32,
                                    message: String::from("received request after shutdown"),
                                    data: None,
                                }),
                            }))
                            .unwrap();
                        continue;
                    }

                    match &r.method[..] {
                        lsp_types::request::HoverRequest::METHOD => {
                            let tdp =
                                serde_json::from_value::<lsp_types::TextDocumentPositionParams>(
                                    r.params,
                                )
                                .unwrap();

                            let response = match get_word(tdp) {
                                Some(w) => {
                                    let text = self.dict.info(&w);
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
                        }
                        lsp_types::request::Shutdown::METHOD => {
                            self.shutdown = true;
                            let none: Option<()> = None;
                            c.sender
                                .send(lsp_server::Message::Response(lsp_server::Response::new_ok(
                                    r.id, none,
                                )))
                                .unwrap()
                        }
                        _ => c
                            .sender
                            .send(lsp_server::Message::Notification(
                                lsp_server::Notification::new(
                                    "window/logMessage".to_string(),
                                    format!("Unmatched request received: {}", r.method),
                                ),
                            ))
                            .unwrap(),
                    }
                }
                lsp_server::Message::Response(r) => c
                    .sender
                    .send(lsp_server::Message::Notification(
                        lsp_server::Notification::new(
                            "window/logMessage".to_string(),
                            format!("Unmatched response received: {}", r.id),
                        ),
                    ))
                    .unwrap(),
                lsp_server::Message::Notification(n) => match &n.method[..] {
                    "exit" => {
                        if self.shutdown {
                            return Ok(());
                        } else {
                            return Err(String::from(
                                "Received exit notification before shutdown request",
                            ));
                        }
                    }
                    _ => c
                        .sender
                        .send(lsp_server::Message::Notification(
                            lsp_server::Notification::new(
                                "window/logMessage".to_string(),
                                format!("Unmatched notification received: {}", n.method),
                            ),
                        ))
                        .unwrap(),
                },
            }
        }

        // io.join().unwrap();
    }
}

fn main() {
    let (p, c, io) = connect();
    let server = Server::new(p);
    match server.serve(c, io) {
        Ok(()) => (),
        Err(s) => {
            eprintln!("{}", s);
            std::process::exit(1)
        }
    }
}

struct Dict {
    wordnet: WordNet,
}

struct DictItem {
    definitions: Vec<wordnet::Definition>,
    synonyms: Vec<String>,
}

impl DictItem {
    fn render(&self, word: &str) -> String {
        let mut blocks = Vec::new();
        blocks.push(format!("# {}", word));

        let mut defs = HashMap::new();
        for d in self.definitions.iter() {
            match defs.get_mut(&d.pos) {
                None => {
                    defs.insert(d.pos, vec![d.def.clone()]);
                }
                Some(v) => {
                    v.push(d.def.clone());
                }
            }
        }

        if defs.len() > 0 {
            blocks.push("## Definitions".to_string())
        }

        for (pos, def) in defs {
            blocks.push(format!("_{}_", pos));
            blocks.push(
                def.iter()
                    .enumerate()
                    .map(|(i, x)| format!("{}. {}", i + 1, x))
                    .collect::<Vec<String>>()
                    .join("\n"),
            )
        }

        let syns = self
            .synonyms
            .iter()
            .map(|x| x.replace("_", " "))
            .collect::<Vec<String>>()
            .join(", ");
        if syns.len() > 0 {
            blocks.push("## Synonyms".to_string());
            blocks.push(syns)
        }

        blocks.join("\n\n")
    }
}

impl Dict {
    fn new(value: &Path) -> Self {
        Self {
            wordnet: WordNet::new(value.to_path_buf()),
        }
    }

    fn info(&mut self, word: &str) -> String {
        let definitions = self.wordnet.definitions(word);
        let synonyms = self.wordnet.synonyms(word);
        let di = DictItem {
            definitions,
            synonyms,
        };
        di.render(word)
    }
}

fn get_word(tdp: lsp_types::TextDocumentPositionParams) -> Option<String> {
    let file = std::fs::File::open(tdp.text_document.uri.to_file_path().unwrap()).unwrap();
    let reader = std::io::BufReader::new(file);
    let line = match reader.lines().nth(tdp.position.line as usize) {
        None => return None,
        Some(l) => match l {
            Err(_) => return None,
            Ok(l) => l,
        },
    };
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
