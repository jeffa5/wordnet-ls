use clap::Parser;
use lls_lib::wordnet::PartOfSpeech;
use lls_lib::wordnet::Relation;
use lls_lib::wordnet::SynSet;
use lls_lib::wordnet::WordNet;
use lsp_server::ErrorCode;
use lsp_server::Message;
use lsp_server::Notification;
use lsp_server::Response;
use lsp_server::ResponseError;
use lsp_server::{Connection, IoThreads};
use lsp_types::notification::LogMessage;
use lsp_types::notification::Notification as _;
use lsp_types::notification::ShowMessage;
use lsp_types::request::Request;
use lsp_types::CompletionItem;
use lsp_types::CompletionList;
use lsp_types::InitializeParams;
use lsp_types::InitializeResult;
use lsp_types::Location;
use lsp_types::Position;
use lsp_types::PositionEncodingKind;
use lsp_types::Range;
use lsp_types::ServerCapabilities;
use lsp_types::ServerInfo;
use lsp_types::TextDocumentSyncKind;
use lsp_types::Url;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Clone, Parser)]
struct Args {
    #[clap(long)]
    stdio: bool,
}

fn log(c: &Connection, message: impl Serialize) {
    c.sender
        .send(Message::Notification(Notification::new(
            LogMessage::METHOD.to_string(),
            message,
        )))
        .unwrap();
}

fn server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        hover_provider: Some(lsp_types::HoverProviderCapability::Simple(true)),
        definition_provider: Some(lsp_types::OneOf::Left(true)),
        completion_provider: Some(lsp_types::CompletionOptions {
            resolve_provider: Some(true),
            ..Default::default()
        }),
        text_document_sync: Some(lsp_types::TextDocumentSyncCapability::Options(
            lsp_types::TextDocumentSyncOptions {
                open_close: Some(true),
                change: Some(TextDocumentSyncKind::INCREMENTAL),
                ..Default::default()
            },
        )),
        ..Default::default()
    }
}

fn connect(stdio: bool) -> (lsp_types::InitializeParams, Connection, IoThreads) {
    let (connection, io) = if stdio {
        Connection::stdio()
    } else {
        panic!("No connection mode given, e.g. --stdio");
    };
    let mut caps = server_capabilities();
    let (id, params) = connection.initialize_start().unwrap();
    let init_params = serde_json::from_value::<InitializeParams>(params).unwrap();
    if let Some(general) = &init_params.capabilities.general {
        let pe = general
            .position_encodings
            .clone()
            .unwrap_or_default()
            .iter()
            .find(|&pe| *pe == PositionEncodingKind::UTF8)
            .cloned()
            .unwrap_or(PositionEncodingKind::UTF16);
        caps.position_encoding = Some(pe);
    }
    let init_result = InitializeResult {
        capabilities: caps,
        server_info: Some(ServerInfo {
            name: "lls".to_owned(),
            version: None,
        }),
    };
    connection
        .initialize_finish(id, serde_json::to_value(init_result).unwrap())
        .unwrap();
    // log(&c, format!("{:?}", params.initialization_options));
    (init_params, connection, io)
}

struct Server {
    dict: Dict,
    open_files: BTreeMap<String, String>,
    shutdown: bool,
}

#[derive(Serialize, Deserialize)]
struct InitializationOptions {
    wordnet: PathBuf,
}

impl Server {
    fn new(c: &Connection, params: lsp_types::InitializeParams) -> Self {
        let init_opts = if let Some(io) = params.initialization_options {
            match serde_json::from_value::<InitializationOptions>(io) {
                Ok(v) => v,
                Err(err) => {
                    c.sender
                        .send(Message::Notification(Notification::new(
                            ShowMessage::METHOD.to_string(),
                            format!("Invalid initialization options: {err}"),
                        )))
                        .unwrap();
                    panic!("Invalid initialization options: {err}")
                }
            }
        } else {
            c.sender
                .send(Message::Notification(Notification::new(
                    ShowMessage::METHOD.to_string(),
                    "No initialization options given, need it for wordnet location at least"
                        .to_string(),
                )))
                .unwrap();
            panic!("No initialization options given, need it for wordnet location at least")
        };
        let wordnet_location = if init_opts.wordnet.starts_with("~/") {
            dirs::home_dir()
                .unwrap()
                .join(init_opts.wordnet.strip_prefix("~/").unwrap())
        } else {
            init_opts.wordnet
        };
        Self {
            dict: Dict::new(&wordnet_location),
            open_files: BTreeMap::new(),
            shutdown: false,
        }
    }

    fn serve(mut self, c: Connection) -> Result<(), String> {
        loop {
            match c.receiver.recv().unwrap() {
                Message::Request(r) => {
                    // log(&c, format!("Got request {r:?}"));
                    if self.shutdown {
                        c.sender
                            .send(Message::Response(Response {
                                id: r.id,
                                result: None,
                                error: Some(ResponseError {
                                    code: ErrorCode::InvalidRequest as i32,
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

                            let response = match self.get_word(&tdp) {
                                Some(w) => {
                                    let text = self.dict.hover(&w);
                                    let resp = lsp_types::Hover {
                                        contents: lsp_types::HoverContents::Markup(
                                            lsp_types::MarkupContent {
                                                kind: lsp_types::MarkupKind::Markdown,
                                                value: text,
                                            },
                                        ),
                                        range: None,
                                    };
                                    Message::Response(Response {
                                        id: r.id,
                                        result: Some(serde_json::to_value(resp).unwrap()),
                                        error: None,
                                    })
                                }
                                None => Message::Response(Response {
                                    id: r.id,
                                    result: None,
                                    error: None,
                                }),
                            };

                            c.sender.send(response).unwrap()
                        }
                        lsp_types::request::GotoDefinition::METHOD => {
                            let tdp =
                                serde_json::from_value::<lsp_types::TextDocumentPositionParams>(
                                    r.params,
                                )
                                .unwrap();

                            let response = match self.get_word(&tdp) {
                                Some(w) => {
                                    let filename = self.dict.all_info(&w);
                                    let resp =
                                        lsp_types::GotoDefinitionResponse::Scalar(Location {
                                            uri: Url::from_file_path(filename).unwrap(),
                                            range: Range::default(),
                                        });
                                    Message::Response(Response {
                                        id: r.id,
                                        result: serde_json::to_value(resp).ok(),
                                        error: None,
                                    })
                                }
                                None => Message::Response(Response {
                                    id: r.id,
                                    result: None,
                                    error: None,
                                }),
                            };

                            c.sender.send(response).unwrap()
                        }
                        lsp_types::request::Completion::METHOD => {
                            let mut tdp = serde_json::from_value::<
                                lsp_types::TextDocumentPositionParams,
                            >(r.params)
                            .unwrap();

                            tdp.position.character -= 1;
                            let response = match self.get_word(&tdp) {
                                Some(word) => {
                                    let start = match self.dict.all_words.binary_search(&word) {
                                        Ok(v) => v,
                                        Err(v) => v,
                                    };
                                    let matched_words = self
                                        .dict
                                        .all_words
                                        .iter()
                                        .skip(start)
                                        .filter(|w| w.starts_with(&word))
                                        .take(100);
                                    let completion_items = matched_words
                                        .map(|mw| CompletionItem {
                                            label: mw.clone(),
                                            ..Default::default()
                                        })
                                        .collect();
                                    let resp =
                                        lsp_types::CompletionResponse::List(CompletionList {
                                            // incomplete as we limit to the first 100 above
                                            is_incomplete: true,
                                            items: completion_items,
                                        });
                                    Message::Response(Response {
                                        id: r.id,
                                        result: serde_json::to_value(resp).ok(),
                                        error: None,
                                    })
                                }
                                None => Message::Response(Response {
                                    id: r.id,
                                    result: None,
                                    error: None,
                                }),
                            };

                            c.sender.send(response).unwrap()
                        }
                        lsp_types::request::ResolveCompletionItem::METHOD => {
                            let mut ci =
                                serde_json::from_value::<lsp_types::CompletionItem>(r.params)
                                    .unwrap();

                            let doc = self.dict.hover(&ci.label);
                            ci.documentation = Some(lsp_types::Documentation::MarkupContent(
                                lsp_types::MarkupContent {
                                    kind: lsp_types::MarkupKind::Markdown,
                                    value: doc,
                                },
                            ));
                            let response = Message::Response(Response {
                                id: r.id,
                                result: serde_json::to_value(ci).ok(),
                                error: None,
                            });

                            c.sender.send(response).unwrap()
                        }
                        lsp_types::request::Shutdown::METHOD => {
                            self.shutdown = true;
                            let none: Option<()> = None;
                            c.sender
                                .send(Message::Response(Response::new_ok(r.id, none)))
                                .unwrap()
                        }
                        _ => log(&c, format!("Unmatched request received: {}", r.method)),
                    }
                }
                Message::Response(r) => log(&c, format!("Unmatched response received: {}", r.id)),
                Message::Notification(n) => {
                    match &n.method[..] {
                        lsp_types::notification::DidOpenTextDocument::METHOD => {
                            let dotdp = serde_json::from_value::<
                                lsp_types::DidOpenTextDocumentParams,
                            >(n.params)
                            .unwrap();
                            self.open_files.insert(
                                dotdp.text_document.uri.to_string(),
                                dotdp.text_document.text,
                            );
                            // log(
                            //     &c,
                            //     format!(
                            //         "got open document notification for {:?}",
                            //         dotdp.text_document.uri
                            //     ),
                            // );
                        }
                        lsp_types::notification::DidChangeTextDocument::METHOD => {
                            let dctdp = serde_json::from_value::<
                                lsp_types::DidChangeTextDocumentParams,
                            >(n.params)
                            .unwrap();
                            let doc = dctdp.text_document.uri.to_string();
                            let content = self.open_files.get_mut(&doc).unwrap();
                            for change in dctdp.content_changes {
                                if let Some(range) = change.range {
                                    let start = resolve_position(content, range.start);
                                    let end = resolve_position(content, range.end);
                                    content.replace_range(start..end, &change.text);
                                } else {
                                    // full content replace
                                    *content = change.text;
                                }
                            }
                            // log(&c, format!("got change document notification for {doc:?}"))
                        }
                        lsp_types::notification::DidCloseTextDocument::METHOD => {
                            let dctdp = serde_json::from_value::<
                                lsp_types::DidCloseTextDocumentParams,
                            >(n.params)
                            .unwrap();
                            self.open_files.remove(&dctdp.text_document.uri.to_string());
                            // log(
                            //     &c,
                            //     format!(
                            //         "got close document notification for {:?}",
                            //         dctdp.text_document.uri
                            //     ),
                            // );
                        }
                        lsp_types::notification::Exit::METHOD => {
                            if self.shutdown {
                                return Ok(());
                            } else {
                                return Err(String::from(
                                    "Received exit notification before shutdown request",
                                ));
                            }
                        }
                        _ => log(&c, format!("Unmatched notification received: {}", n.method)),
                    }
                }
            }
        }
    }

    fn get_file_content(&self, uri: &Url) -> String {
        if let Some(content) = self.open_files.get(&uri.to_string()) {
            content.to_owned()
        } else {
            std::fs::read_to_string(uri.to_file_path().unwrap()).unwrap()
        }
    }

    fn get_word(&self, tdp: &lsp_types::TextDocumentPositionParams) -> Option<String> {
        let content = self.get_file_content(&tdp.text_document.uri);
        let line = match content.lines().nth(tdp.position.line as usize) {
            None => return None,
            Some(l) => l,
        };

        let mut current_word = String::new();
        let mut found = false;
        let word_char = |c: char| c.is_alphabetic() || c == '_';
        for (i, c) in line.chars().enumerate() {
            if word_char(c) {
                for c in c.to_lowercase() {
                    current_word.push(c);
                }
            } else {
                if found {
                    return Some(current_word);
                }
                current_word.clear();
            }

            if i == tdp.position.character as usize {
                found = true
            }

            if !word_char(c) && found {
                return Some(current_word);
            }
        }

        // got to end of line
        if found {
            return Some(current_word);
        }

        None
    }
}

fn main() {
    let args = Args::parse();
    let (p, c, io) = connect(args.stdio);
    let server = Server::new(&c, p);
    let s = server.serve(c);
    io.join().unwrap();
    match s {
        Ok(()) => (),
        Err(s) => {
            eprintln!("{}", s);
            std::process::exit(1)
        }
    }
}

struct Dict {
    wordnet: WordNet,
    all_words: Vec<String>,
}

impl Dict {
    fn new(value: &Path) -> Self {
        let wn = WordNet::new(value.to_path_buf());
        let all_words = wn.all_words();
        Self {
            wordnet: wn,
            all_words,
        }
    }

    fn hover(&mut self, word: &str) -> String {
        let synsets = self.wordnet.synsets(word);
        self.render_hover(word, synsets)
    }

    fn render_hover(&self, word: &str, synsets: Vec<SynSet>) -> String {
        let mut blocks = Vec::new();

        for pos in PartOfSpeech::iter() {
            let ss_pos = synsets
                .iter()
                .filter(|ss| ss.part_of_speech == pos)
                .collect::<Vec<_>>();

            let defs = ss_pos.iter().map(|ss| &ss.definition).collect::<Vec<_>>();
            if !defs.is_empty() {
                let mut s = String::new();
                s.push_str(&format!("**{word}** _{pos}_\n"));
                s.push_str(
                    &defs
                        .iter()
                        .enumerate()
                        .map(|(i, x)| format!("{}. {}", i + 1, x))
                        .collect::<Vec<String>>()
                        .join("\n"),
                );
                blocks.push(s);
            }

            let mut synonyms = ss_pos
                .iter()
                .flat_map(|ss| &ss.words)
                .filter(|w| **w != word)
                .collect::<Vec<_>>();
            synonyms.sort();
            synonyms.dedup();
            if !synonyms.is_empty() {
                let syns = synonyms
                    .iter()
                    .map(|x| x.replace('_', " "))
                    .collect::<Vec<String>>()
                    .join(", ");
                blocks.push(format!("**Synonyms**: {syns}"));
            }

            let mut antonyms = ss_pos
                .iter()
                .flat_map(|ss| {
                    ss.with_relationship(Relation::Antonym)
                        .into_iter()
                        .flat_map(|r| {
                            self.wordnet
                                .resolve(r.part_of_speech, r.synset_offset)
                                .map(|ss| ss.words)
                                .unwrap_or_default()
                        })
                })
                .collect::<Vec<_>>();
            antonyms.sort();
            antonyms.dedup();
            if !antonyms.is_empty() {
                let ants = antonyms
                    .iter()
                    .map(|x| x.replace('_', " "))
                    .collect::<Vec<String>>()
                    .join(", ");
                blocks.push(format!("**Antonyms**: {ants}"));
            }
        }

        blocks.join("\n\n")
    }

    fn all_info(&self, word: &str) -> PathBuf {
        let synsets = self.wordnet.synsets(word);
        let filename = PathBuf::from(format!("/tmp/lls-{word}.md"));
        let mut file = File::create(&filename).unwrap();
        file.write_all(format!("# {word}\n").as_bytes()).unwrap();
        for (i, mut synset) in synsets.into_iter().enumerate() {
            synset.words.sort_unstable();
            let synonyms = synset
                .words
                .into_iter()
                .filter(|w| w != word)
                .collect::<BTreeSet<_>>();
            let definition = synset.definition;
            let pos = synset.part_of_speech.to_string();
            let mut relationships: BTreeMap<Relation, BTreeSet<String>> = BTreeMap::new();
            for r in synset.relationships {
                relationships.entry(r.relation).or_default().extend(
                    self.wordnet
                        .resolve(r.part_of_speech, r.synset_offset)
                        .unwrap()
                        .words,
                );
            }
            let mut relationships = relationships
                .into_iter()
                .map(|(r, w)| (r.to_string(), w))
                .collect::<BTreeMap<_, _>>();
            relationships.insert("synonym".to_owned(), synonyms);
            let relationships_str = relationships
                .into_iter()
                .map(|(relation, words)| {
                    format!(
                        "**{relation}**: {}",
                        words.into_iter().collect::<Vec<_>>().join(", ")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");

            let i = i + 1;
            file.write_all(
                format!("\n{i}. _{pos}_ {definition}\n{relationships_str}\n").as_bytes(),
            )
            .unwrap();
        }
        filename
    }
}

fn resolve_position(content: &str, pos: Position) -> usize {
    let count = content
        .lines()
        .map(|l| l.len())
        .take(pos.line as usize)
        .sum::<usize>();
    pos.line as usize + count + pos.character as usize
}
