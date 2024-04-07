use clap::Parser;
use lls_lib::wordnet::LexicalRelation;
use lls_lib::wordnet::PartOfSpeech;
use lls_lib::wordnet::SemanticRelation;
use lls_lib::wordnet::SynSet;
use lls_lib::wordnet::WordNet;
use lsp_server::ErrorCode;
use lsp_server::Message;
use lsp_server::Notification;
use lsp_server::Request;
use lsp_server::RequestId;
use lsp_server::Response;
use lsp_server::ResponseError;
use lsp_server::{Connection, IoThreads};
use lsp_types::notification::LogMessage;
use lsp_types::notification::Notification as _;
use lsp_types::notification::ShowMessage;
use lsp_types::request::Request as _;
use lsp_types::CompletionItem;
use lsp_types::CompletionList;
use lsp_types::ExecuteCommandOptions;
use lsp_types::InitializeParams;
use lsp_types::InitializeResult;
use lsp_types::Location;
use lsp_types::Position;
use lsp_types::PositionEncodingKind;
use lsp_types::Range;
use lsp_types::ServerCapabilities;
use lsp_types::ServerInfo;
use lsp_types::ShowDocumentParams;
use lsp_types::TextDocumentPositionParams;
use lsp_types::TextDocumentSyncKind;
use lsp_types::Url;
use serde::Deserialize;
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs::File;
use std::io::Write as _;
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
        code_action_provider: Some(lsp_types::CodeActionProviderCapability::Simple(true)),
        execute_command_provider: Some(ExecuteCommandOptions {
            commands: vec!["define".to_owned()],
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn connect(stdio: bool) -> (lsp_types::InitializeParams, Connection, IoThreads) {
    let (connection, io) = if stdio {
        Connection::stdio()
    } else {
        panic!("No connection mode given, e.g. --stdio");
    };
    let (id, params) = connection.initialize_start().unwrap();
    let mut caps = server_capabilities();
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
    let init_opts = if let Some(io) = &init_params.initialization_options {
        match serde_json::from_value::<InitializationOptions>(io.clone()) {
            Ok(v) => v,
            Err(err) => {
                connection
                    .sender
                    .send(Message::Notification(Notification::new(
                        ShowMessage::METHOD.to_string(),
                        format!("Invalid initialization options: {err}"),
                    )))
                    .unwrap();
                panic!("Invalid initialization options: {err}")
            }
        }
    } else {
        connection
            .sender
            .send(Message::Notification(Notification::new(
                ShowMessage::METHOD.to_string(),
                "No initialization options given, need it for wordnet location at least"
                    .to_string(),
            )))
            .unwrap();
        panic!("No initialization options given, need it for wordnet location at least")
    };
    if !init_opts.enable_completion.unwrap_or(true) {
        caps.completion_provider = None;
    }
    if !init_opts.enable_hover.unwrap_or(true) {
        caps.hover_provider = None;
    }
    if !init_opts.enable_code_actions.unwrap_or(true) {
        caps.code_action_provider = None;
        caps.execute_command_provider = None;
    }
    if !init_opts.enable_goto_definition.unwrap_or(true) {
        caps.definition_provider = None;
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
    enable_completion: Option<bool>,
    enable_hover: Option<bool>,
    enable_code_actions: Option<bool>,
    enable_goto_definition: Option<bool>,
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

                            let response = match self
                                .get_words_from_document(&tdp)
                                .into_iter()
                                .find(|w| self.dict.wordnet.lemmatize(w).any(|w| !w.is_empty()))
                            {
                                Some(w) => {
                                    if let Some(text) = self.dict.hover(&w) {
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
                                    } else {
                                        Message::Response(Response {
                                            id: r.id,
                                            result: None,
                                            error: None,
                                        })
                                    }
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

                            let words = self.get_words_from_document(&tdp);
                            let response = match self.dict.all_info_file(&words) {
                                Some(filename) => {
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
                            let response = match self
                                .get_words_from_document(&tdp)
                                .into_iter()
                                .find(|w| self.dict.wordnet.lemmatize(w).any(|w| !w.is_empty()))
                            {
                                Some(word) => {
                                    let limit = 100;
                                    let completion_items = self.dict.complete(&word, limit);
                                    let resp =
                                        lsp_types::CompletionResponse::List(CompletionList {
                                            is_incomplete: completion_items.len() == limit,
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

                            let response = if let Some(doc) = self.dict.hover(&ci.label) {
                                ci.documentation = Some(lsp_types::Documentation::MarkupContent(
                                    lsp_types::MarkupContent {
                                        kind: lsp_types::MarkupKind::Markdown,
                                        value: doc,
                                    },
                                ));
                                Message::Response(Response {
                                    id: r.id,
                                    result: serde_json::to_value(ci).ok(),
                                    error: None,
                                })
                            } else {
                                Message::Response(Response {
                                    id: r.id,
                                    result: None,
                                    error: None,
                                })
                            };

                            c.sender.send(response).unwrap()
                        }
                        lsp_types::request::CodeActionRequest::METHOD => {
                            let cap =
                                serde_json::from_value::<lsp_types::CodeActionParams>(r.params)
                                    .unwrap();

                            let tdp = TextDocumentPositionParams {
                                text_document: cap.text_document,
                                position: cap.range.start,
                            };

                            let words = self.get_words_from_document(&tdp);
                            let completion_items = words
                                .into_iter()
                                .filter(|w| self.dict.wordnet.contains(w))
                                .map(|w| {
                                    let args = serde_json::to_value(DefineCommandArguments {
                                        word: w.to_owned(),
                                    })
                                    .unwrap();
                                    lsp_types::CodeActionOrCommand::Command(lsp_types::Command {
                                        title: format!("Define {w:?}"),
                                        command: "define".to_owned(),
                                        arguments: Some(vec![args]),
                                    })
                                })
                                .collect::<Vec<_>>();
                            let response = Message::Response(Response {
                                id: r.id,
                                result: Some(serde_json::to_value(completion_items).unwrap()),
                                error: None,
                            });

                            c.sender.send(response).unwrap()
                        }
                        lsp_types::request::ExecuteCommand::METHOD => {
                            let mut cap =
                                serde_json::from_value::<lsp_types::ExecuteCommandParams>(r.params)
                                    .unwrap();

                            let response = match cap.command.as_str() {
                                "define" => {
                                    let arg = cap.arguments.swap_remove(0);
                                    match serde_json::from_value::<DefineCommandArguments>(arg) {
                                        Ok(args) => match self.dict.all_info_file(&[args.word]) {
                                            Some(filename) => {
                                                let params = ShowDocumentParams {
                                                    uri: Url::from_file_path(filename).unwrap(),
                                                    external: None,
                                                    take_focus: None,
                                                    selection: None,
                                                };
                                                c.sender
                                                    .send(Message::Request(Request {
                                                        id: RequestId::from(0),
                                                        method:
                                                            lsp_types::request::ShowDocument::METHOD
                                                                .to_owned(),
                                                        params: serde_json::to_value(params)
                                                            .unwrap(),
                                                    }))
                                                    .unwrap();
                                                Message::Response(Response {
                                                    id: r.id,
                                                    result: None,
                                                    error: None,
                                                })
                                            }
                                            None => Message::Response(Response {
                                                id: r.id,
                                                result: None,
                                                error: None,
                                            }),
                                        },
                                        _ => Message::Response(Response {
                                            id: r.id,
                                            result: None,
                                            error: Some(ResponseError {
                                                code: ErrorCode::InvalidRequest as i32,
                                                message: String::from("invalid arguments"),
                                                data: None,
                                            }),
                                        }),
                                    }
                                }
                                _ => Message::Response(Response {
                                    id: r.id,
                                    result: None,
                                    error: Some(ResponseError {
                                        code: ErrorCode::InvalidRequest as i32,
                                        message: String::from("unknown command"),
                                        data: None,
                                    }),
                                }),
                            };

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

    fn get_words_from_document(&self, tdp: &lsp_types::TextDocumentPositionParams) -> Vec<String> {
        let content = self.get_file_content(&tdp.text_document.uri);
        get_words_from_content(
            &content,
            tdp.position.line as usize,
            tdp.position.character as usize,
        )
    }
}

fn get_words_from_content(content: &str, line: usize, character: usize) -> Vec<String> {
    let line = match content.lines().nth(line) {
        None => return Vec::new(),
        Some(l) => l,
    };

    let mut words = Vec::new();
    let mut current_word = String::new();
    if let Some(word) = get_word_from_line(line, character) {
        for single_word in word.split_whitespace() {
            if !current_word.is_empty() {
                current_word.push('_');
            }
            current_word.push_str(single_word);
            words.push(current_word.clone());
            // now try and simplify the word
            for c in WORD_PUNC.chars() {
                if let Some(w) = current_word.strip_prefix(c) {
                    words.push(w.to_owned());
                    if let Some(w) = w.strip_suffix(c) {
                        words.push(w.to_owned());
                    }
                }
                if let Some(w) = current_word.strip_suffix(c) {
                    words.push(w.to_owned());
                }
            }
        }
    }
    // sort by length to try and find the simplest
    words.sort_unstable_by(|s1, s2| {
        if s1.len() < s2.len() {
            Ordering::Less
        } else {
            s1.cmp(s2)
        }
    });
    words.dedup();
    words
}

const WORD_PUNC: &str = "_-'./";

fn get_word_from_line(line: &str, character: usize) -> Option<String> {
    let mut current_word = String::new();
    let mut found = false;
    let mut match_chars = WORD_PUNC.to_owned();
    let word_char = |match_with: &str, c: char| c.is_alphanumeric() || match_with.contains(c);
    for (i, c) in line.chars().enumerate() {
        if word_char(&match_chars, c) {
            for c in c.to_lowercase() {
                current_word.push(c);
            }
        } else {
            if found {
                return Some(current_word);
            }
            current_word.clear();
        }

        if i == character {
            if word_char(&match_chars, c) {
                match_chars.push(' ');
                found = true
            } else {
                return None;
            }
        }

        if !word_char(&match_chars, c) && found {
            return Some(current_word);
        }
    }

    // got to end of line
    if found {
        return Some(current_word);
    }

    None
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
        let wn = WordNet::new(value);
        let all_words = wn.all_words();
        Self {
            wordnet: wn,
            all_words,
        }
    }

    fn hover(&self, word: &str) -> Option<String> {
        let lemmas = self.wordnet.lemmatize(word);
        if lemmas.all(|w| w.is_empty()) {
            return None;
        }
        let mut content = String::new();
        lemmas.for_each(|pos, lemmas| {
            lemmas.into_iter().for_each(|lemma| {
                let synsets = self.wordnet.synsets_for(&lemma, pos);
                let hover = self.render_hover(&lemma, synsets);
                writeln!(content, "{hover}\n").unwrap();
            });
        });
        Some(content.trim().to_owned())
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
                    &ss_pos
                        .iter()
                        .enumerate()
                        .map(|(i, ss)| {
                            let mut s = format!("{}. {}.", i + 1, ss.definition);
                            let examples = ss.examples.join("; ");
                            if !examples.is_empty() {
                                s.push_str(" e.g. ");
                                s.push_str(&examples);
                                s.push('.');
                            }
                            s
                        })
                        .collect::<Vec<String>>()
                        .join("\n"),
                );
                blocks.push(s);
            }

            let mut synonyms = ss_pos
                .iter()
                .flat_map(|ss| ss.synonyms())
                .filter(|w| *w != word)
                .collect::<Vec<_>>();
            synonyms.sort();
            synonyms.dedup();
            if !synonyms.is_empty() {
                let syns = synonyms
                    .iter()
                    .map(|x| x.replace('_', " "))
                    .collect::<Vec<String>>()
                    .join(", ");
                blocks.push(format!("**synonyms**: {syns}"));
            }

            let mut antonyms = ss_pos
                .iter()
                .flat_map(|ss| &ss.lemmas)
                .flat_map(|l| l.antonyms(&self.wordnet))
                .collect::<Vec<_>>();
            antonyms.sort();
            antonyms.dedup();
            if !antonyms.is_empty() {
                let ants = antonyms
                    .iter()
                    .map(|x| x.replace('_', " "))
                    .collect::<Vec<String>>()
                    .join(", ");
                blocks.push(format!("**antonyms**: {ants}"));
            }
        }

        blocks.join("\n\n")
    }

    fn all_info_file(&self, words: &[String]) -> Option<PathBuf> {
        let info = self.all_info(words)?;
        let filename = PathBuf::from(format!("/tmp/lls-{}.md", words[0]));
        let mut file = File::create(&filename).unwrap();
        file.write_all(info.as_bytes()).unwrap();
        Some(filename)
    }

    fn all_info(&self, words: &[String]) -> Option<String> {
        let lemmas = words
            .iter()
            .map(|w| self.wordnet.lemmatize(w))
            .filter(|pos| pos.any(|lemmas| !lemmas.is_empty()))
            .collect::<Vec<_>>();
        if lemmas.is_empty() {
            return None;
        }
        let mut content = String::new();
        lemmas.into_iter().for_each(|pos| {
            pos.for_each(|pos, lemmas| {
                lemmas.into_iter().for_each(|lemma| {
                    let synsets = self.wordnet.synsets_for(&lemma, pos);
                    writeln!(content, "# {lemma}").unwrap();
                    for (i, synset) in synsets.into_iter().enumerate() {
                        let definition = synset.definition;
                        let pos = synset.part_of_speech.to_string();

                        let i = i + 1;
                        write!(content, "\n{i}. _{pos}_ {definition}.").unwrap();
                        let examples = synset.examples.join("; ");
                        if !examples.is_empty() {
                            writeln!(content, " e.g. {examples}.").unwrap();
                        } else {
                            writeln!(content).unwrap();
                        }

                        let mut relationships: BTreeMap<SemanticRelation, BTreeSet<String>> =
                            BTreeMap::new();
                        for r in synset.relationships {
                            relationships.entry(r.relation).or_default().extend(
                                self.wordnet
                                    .resolve(r.part_of_speech, r.synset_offset)
                                    .unwrap()
                                    .synonyms(),
                            );
                        }
                        let relationships = relationships
                            .into_iter()
                            .map(|(r, w)| (r.to_string(), w))
                            .collect::<BTreeMap<_, _>>();
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

                        if !relationships_str.is_empty() {
                            writeln!(content, "{relationships_str}").unwrap()
                        }

                        let lemma_relationships = synset
                            .lemmas
                            .iter()
                            .filter(|l| l.word != lemma)
                            .map(|l| {
                                (
                                    l.word.clone(),
                                    l.relationships
                                        .iter()
                                        .map(|lr| {
                                            (
                                                lr.relation,
                                                self.wordnet
                                                    .resolve(lr.part_of_speech, lr.synset_offset)
                                                    .unwrap()
                                                    .synonyms()[lr.target]
                                                    .clone(),
                                            )
                                        })
                                        .filter(|(_, w)| *w != l.word)
                                        .collect::<BTreeMap<LexicalRelation, String>>(),
                                )
                            })
                            .collect::<BTreeMap<_, _>>();
                        let lemma_relationships_str = lemma_relationships
                            .into_iter()
                            .map(|(word, relationships)| {
                                let relationships_str = relationships
                                    .into_iter()
                                    .map(|(relation, word)| format!("- **{relation}**: {word}"))
                                    .collect::<Vec<String>>()
                                    .join("\n  ");
                                if relationships_str.is_empty() {
                                    format!("- {word}")
                                } else {
                                    format!("- {word}:\n  {relationships_str}")
                                }
                            })
                            .collect::<Vec<String>>()
                            .join("\n");

                        if !lemma_relationships_str.is_empty() {
                            writeln!(content, "**synonyms**:\n{lemma_relationships_str}").unwrap();
                        }
                    }
                    writeln!(content).unwrap();
                })
            })
        });
        Some(content.trim().to_owned())
    }

    fn complete(&self, word: &String, limit: usize) -> Vec<CompletionItem> {
        let start = match self.all_words.binary_search(word) {
            Ok(v) => v,
            Err(v) => v,
        };
        let matched_words = self
            .all_words
            .iter()
            .skip(start)
            .filter(|w| w.starts_with(word))
            .take(limit);
        matched_words
            .map(|mw| {
                let insert_text = mw.replace('_', " ");
                CompletionItem {
                    label: mw.clone(),
                    insert_text: (mw != &insert_text).then_some(insert_text),
                    ..Default::default()
                }
            })
            .collect()
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

#[derive(Debug, Serialize, Deserialize)]
struct DefineCommandArguments {
    word: String,
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;
    use expect_test::{expect, Expect};

    #[test]
    fn hover_woman() {
        let wndir = env::var("WNSEARCHDIR").unwrap();
        let dict = Dict::new(&PathBuf::from(wndir));
        let hover = dict.hover("woman").unwrap();
        let expected = expect![[r#"
            **woman** _noun_
            1. an adult female person (as opposed to a man). e.g. the woman kept house while the man hunted.
            2. a female person who plays a significant role (wife or mistress or girlfriend) in the life of a particular man. e.g. he was faithful to his woman.
            3. a human female employed to do housework. e.g. the char will clean the carpet; I have a woman who comes in four hours a day while I write.
            4. women as a class. e.g. it's an insult to American womanhood; woman is the glory of creation; the fair sex gathered on the veranda.

            **synonyms**: adult female, char, charwoman, cleaning lady, cleaning woman, fair sex, womanhood

            **antonyms**: man"#]];
        expected.assert_eq(&hover);
    }

    #[test]
    fn all_info_woman() {
        let wndir = env::var("WNSEARCHDIR").unwrap();
        let dict = Dict::new(&PathBuf::from(wndir));
        let info = dict.all_info(&["woman".to_owned()]).unwrap();
        let expected = expect![[r#"
            # woman

            1. _noun_ an adult female person (as opposed to a man). e.g. the woman kept house while the man hunted.
            **hypernym**: adult, female, female_person, grownup
            **hyponym**: B-girl, Black_woman, Cinderella, Delilah, Wac, Wave, amazon, bachelor_girl, bachelorette, baggage, ball-breaker, ball-buster, bar_girl, bas_bleu, bawd, beauty, bluestocking, bridesmaid, broad, cat, cocotte, coquette, cyprian, dame, deb, debutante, dish, divorcee, dominatrix, donna, enchantress, ex, ex-wife, eyeful, fancy_woman, femme_fatale, fille, flirt, geisha, geisha_girl, gentlewoman, girl, girlfriend, gold_digger, grass_widow, gravida, harlot, heroine, houri, inamorata, jezebel, jilt, kept_woman, knockout, lady, lady_friend, lady_of_pleasure, looker, lulu, ma'am, madam, maenad, maid_of_honor, mantrap, married_woman, materfamilias, matriarch, matron, mestiza, minx, miss, missy, mistress, mother_figure, nanny, nullipara, nurse, nursemaid, nymph, nymphet, old_woman, peach, prickteaser, prostitute, ravisher, shiksa, shikse, siren, smasher, sporting_lady, stunner, sweetheart, sylph, tart, tease, temptress, unmarried_woman, vamp, vamper, vestal, virago, white_woman, whore, widow, widow_woman, wife, woman_of_the_street, wonder_woman, working_girl, yellow_woman, young_lady, young_woman
            **instance hyponym**: Eve
            **part meronym**: adult_female_body, woman's_body
            **synonyms**:
            - adult_female

            2. _noun_ a female person who plays a significant role (wife or mistress or girlfriend) in the life of a particular man. e.g. he was faithful to his woman.
            **domain of synset usage**: colloquialism
            **hypernym**: female, female_person

            3. _noun_ a human female employed to do housework. e.g. the char will clean the carpet; I have a woman who comes in four hours a day while I write.
            **hypernym**: cleaner
            **synonyms**:
            - char
            - charwoman
            - cleaning_lady
            - cleaning_woman

            4. _noun_ women as a class. e.g. it's an insult to American womanhood; woman is the glory of creation; the fair sex gathered on the veranda.
            **hypernym**: class, social_class, socio-economic_class, stratum
            **member holonym**: womankind
            **synonyms**:
            - fair_sex
            - womanhood:
              - **derivationally related form**: woman"#]];
        expected.assert_eq(&info);
    }

    #[test]
    fn hover_run() {
        let wndir = env::var("WNSEARCHDIR").unwrap();
        let dict = Dict::new(&PathBuf::from(wndir));
        let hover = dict.hover("run").unwrap();
        let expected = expect![[r#"
            **run** _noun_
            1. a score in baseball made by a runner touching all four bases safely. e.g. the Yankees scored 3 runs in the bottom of the 9th; their first tally came in the 3rd inning.
            2. the act of testing something. e.g. in the experimental trials the amount of carbon was measured separately; he called each flip of the coin a new trial.
            3. a race run on foot. e.g. she broke the record for the half-mile run.
            4. an unbroken series of events. e.g. had a streak of bad luck; Nicklaus had a run of birdies.
            5. (American football) a play in which a player attempts to carry the ball through or past the opposing team. e.g. the defensive line braced to stop the run; the coach put great emphasis on running.
            6. a regular trip. e.g. the ship made its run in record time.
            7. the act of running; traveling on foot at a fast pace. e.g. he broke into a run; his daily run keeps him fit.
            8. the continuous period of time during which something (a machine or a factory) operates or continues in operation. e.g. the assembly line was on a 12-hour run.
            9. unrestricted freedom to use. e.g. he has the run of the house.
            10. the production achieved during a continuous period of operation (of a machine or factory etc.). e.g. a daily run of 100,000 gallons of paint.
            11. a small stream.
            12. a race between candidates for elective office. e.g. I managed his campaign for governor; he is raising money for a Senate run.
            13. a row of unravelled stitches. e.g. she got a run in her stocking.
            14. the pouring forth of a fluid.
            15. an unbroken chronological sequence. e.g. the play had a long run on Broadway; the team enjoyed a brief run of victories.
            16. a short trip. e.g. take a run into town.

            **synonyms**: campaign, discharge, foot race, footrace, ladder, outpouring, political campaign, ravel, rill, rivulet, runnel, running, running game, running play, streak, streamlet, tally, test, trial

            **run** _verb_
            1. move fast by using one's feet, with one foot off the ground at any given time. e.g. Don't run--you'll be out of breath; The children ran to the store.
            2. flee; take to one's heels; cut and run. e.g. If you see this man, run!; The burglars escaped before the police showed up.
            3. stretch out over a distance, space, time, or scope; run or extend between two points or beyond a certain point. e.g. Service runs all the way to Cranbury; His knowledge doesn't go very far; My memory extends back to my fourth year of life; The facts extend beyond a consideration of her personal assets.
            4. direct or control; projects, businesses, etc.. e.g. She is running a relief operation in the Sudan.
            5. have a particular form. e.g. the story or argument runs as follows; as the saying goes....
            6. move along, of liquids. e.g. Water flowed into the cave; the Missouri feeds into the Mississippi.
            7. perform as expected when applied. e.g. The washing machine won't go unless it's plugged in; Does this old car still run well?; This old radio doesn't work anymore.
            8. change or be different within limits. e.g. Estimates for the losses in the earthquake range as high as $2 billion; Interest rates run from 5 to 10 percent; The instruments ranged from tuba to cymbals; My students range from very bright to dull.
            9. run, stand, or compete for an office or a position. e.g. Who's running for treasurer this year?.
            10. cause to emit recorded audio or video. e.g. They ran the tapes over and over again; I'll play you my favorite record; He never tires of playing that video.
            11. move about freely and without restraint, or act as if running around in an uncontrolled way. e.g. who are these people running around in the building?; She runs around telling everyone of her troubles; let the dogs run free.
            12. have a tendency or disposition to do or be something; be inclined. e.g. She tends to be nervous before her lectures; These dresses run small; He inclined to corpulence.
            13. be operating, running or functioning. e.g. The car is still running--turn it off!.
            14. change from one state to another. e.g. run amok; run rogue; run riot.
            15. cause to perform. e.g. run a subject; run a process.
            16. be affected by; be subjected to. e.g. run a temperature; run a risk.
            17. continue to exist. e.g. These stories die hard; The legend of Elvis endures.
            18. occur persistently. e.g. Musical talent runs in the family.
            19. carry out a process or program, as on a computer or a machine. e.g. Run the dishwasher; run a new program on the Mac; the computer executed the instruction.
            20. include as the content; broadcast or publicize. e.g. We ran the ad three times; This paper carries a restaurant review; All major networks carried the press conference.
            21. carry out. e.g. run an errand.
            22. pass over, across, or through. e.g. He ran his eyes over her body; She ran her fingers along the carved figurine; He drew her hair through his fingers.
            23. cause something to pass or lead somewhere. e.g. Run the wire behind the cabinet.
            24. make without a miss.
            25. deal in illegally, such as arms or liquor.
            26. cause an animal to move fast. e.g. run the dogs.
            27. be diffused. e.g. These dyes and colors are guaranteed not to run.
            28. sail before the wind.
            29. cover by running; run a certain distance. e.g. She ran 10 miles that day.
            30. extend or continue for a certain period of time. e.g. The film runs 5 hours.
            31. set animals loose to graze.
            32. keep company. e.g. the heifers run with the bulls to produce offspring.
            33. run with the ball; in such sports as football.
            34. travel rapidly, by any (unspecified) means. e.g. Run to the store!; She always runs to Italy, because she has a lover there.
            35. travel a route regularly. e.g. Ships ply the waters near the coast.
            36. pursue for food or sport (as of wild animals). e.g. Goering often hunted wild boars in Poland; The dogs are running deer; The Duke hunted in these woods.
            37. compete in a race. e.g. he is running the Marathon this year; let's race and see who gets there first.
            38. progress by being changed. e.g. The speech has to go through several more drafts; run through your presentation before the meeting.
            39. reduce or cause to be reduced from a solid to a liquid state, usually by heating. e.g. melt butter; melt down gold; The wax melted in the sun.
            40. come unraveled or undone as if by snagging. e.g. Her nylons were running.
            41. become undone. e.g. the sweater unraveled.

            **synonyms**: be given, black market, bleed, break away, bunk, campaign, carry, consort, course, die hard, draw, endure, escape, execute, extend, feed, flow, fly the coop, function, go, guide, head for the hills, hightail it, hunt, hunt down, incline, ladder, lam, lead, lean, melt, melt down, move, operate, pass, persist, play, ply, prevail, race, range, run away, run for, scarper, scat, take to the woods, tend, track down, turn tail, unravel, work

            **antonyms**: idle, malfunction"#]];
        expected.assert_eq(&hover);
    }

    #[test]
    fn all_info_run() {
        let wndir = env::var("WNSEARCHDIR").unwrap();
        let dict = Dict::new(&PathBuf::from(wndir));
        let info = dict.all_info(&["run".to_owned()]).unwrap();
        let expected = expect![[r#"
            # run

            1. _noun_ a score in baseball made by a runner touching all four bases safely. e.g. the Yankees scored 3 runs in the bottom of the 9th; their first tally came in the 3rd inning.
            **hypernym**: score
            **hyponym**: earned_run, rbi, run_batted_in, unearned_run
            **synonyms**:
            - tally

            2. _noun_ the act of testing something. e.g. in the experimental trials the amount of carbon was measured separately; he called each flip of the coin a new trial.
            **hypernym**: attempt, effort, endeavor, endeavour, try
            **hyponym**: MOT, MOT_test, Ministry_of_Transportation_test, Snellen_test, assay, audition, clinical_test, clinical_trial, double_blind, field_trial, fitting, pilot_program, pilot_project, preclinical_phase, preclinical_test, preclinical_trial, try-on, trying_on, tryout
            **synonyms**:
            - test
            - trial

            3. _noun_ a race run on foot. e.g. she broke the record for the half-mile run.
            **hypernym**: race
            **hyponym**: fun_run, funrun, marathon, obstacle_race, steeplechase, track_event
            **synonyms**:
            - foot_race
            - footrace

            4. _noun_ an unbroken series of events. e.g. had a streak of bad luck; Nicklaus had a run of birdies.
            **hypernym**: succession
            **hyponym**: losing_streak, winning_streak
            **synonyms**:
            - streak

            5. _noun_ (American football) a play in which a player attempts to carry the ball through or past the opposing team. e.g. the defensive line braced to stop the run; the coach put great emphasis on running.
            **domain of synset topic**: American_football, American_football_game
            **hypernym**: football_play
            **hyponym**: draw, draw_play, end_run, return, reverse, rush, rushing, sweep
            **synonyms**:
            - running:
              - **derivationally related form**: run
            - running_game
            - running_play

            6. _noun_ a regular trip. e.g. the ship made its run in record time.
            **hypernym**: trip

            7. _noun_ the act of running; traveling on foot at a fast pace. e.g. he broke into a run; his daily run keeps him fit.
            **hypernym**: locomotion, travel
            **hyponym**: dash, sprint
            **synonyms**:
            - running:
              - **derivationally related form**: run

            8. _noun_ the continuous period of time during which something (a machine or a factory) operates or continues in operation. e.g. the assembly line was on a 12-hour run.
            **hypernym**: period, period_of_time, time_period
            **hyponym**: press_run, print_run, run-time

            9. _noun_ unrestricted freedom to use. e.g. he has the run of the house.
            **hypernym**: liberty

            10. _noun_ the production achieved during a continuous period of operation (of a machine or factory etc.). e.g. a daily run of 100,000 gallons of paint.
            **hypernym**: indefinite_quantity

            11. _noun_ a small stream.
            **hypernym**: stream, watercourse
            **synonyms**:
            - rill
            - rivulet
            - runnel
            - streamlet:
              - **derivationally related form**: stream

            12. _noun_ a race between candidates for elective office. e.g. I managed his campaign for governor; he is raising money for a Senate run.
            **hypernym**: race
            **hyponym**: campaign_for_governor, governor's_race, senate_campaign, senate_race
            **synonyms**:
            - campaign
            - political_campaign

            13. _noun_ a row of unravelled stitches. e.g. she got a run in her stocking.
            **hypernym**: damage, harm, impairment
            **synonyms**:
            - ladder
            - ravel

            14. _noun_ the pouring forth of a fluid.
            **hypernym**: flow, flowing
            **hyponym**: escape, jet, leak, leakage, outflow, spirt, spurt, squirt
            **synonyms**:
            - discharge
            - outpouring

            15. _noun_ an unbroken chronological sequence. e.g. the play had a long run on Broadway; the team enjoyed a brief run of victories.
            **hypernym**: chronological_sequence, chronological_succession, sequence, succession, successiveness

            16. _noun_ a short trip. e.g. take a run into town.
            **hypernym**: trip

            # run

            1. _verb_ move fast by using one's feet, with one foot off the ground at any given time. e.g. Don't run--you'll be out of breath; The children ran to the store.
            **hypernym**: hurry, speed, travel_rapidly, zip
            **hyponym**: clip, hare, jog, lope, outrun, romp, run, run_bases, rush, scamper, scurry, scuttle, skitter, sprint, streak, trot
            **verb group**: run

            2. _verb_ flee; take to one's heels; cut and run. e.g. If you see this man, run!; The burglars escaped before the police showed up.
            **hypernym**: go_away, go_forth, leave
            **hyponym**: flee, fly, skedaddle, take_flight
            **synonyms**:
            - break_away
            - bunk
            - escape
            - fly_the_coop
            - head_for_the_hills
            - hightail_it
            - lam
            - run_away:
              - **derivationally related form**: runaway
            - scarper
            - scat
            - take_to_the_woods
            - turn_tail

            3. _verb_ stretch out over a distance, space, time, or scope; run or extend between two points or beyond a certain point. e.g. Service runs all the way to Cranbury; His knowledge doesn't go very far; My memory extends back to my fourth year of life; The facts extend beyond a consideration of her personal assets.
            **hypernym**: be
            **hyponym**: come, go_deep, go_far, radiate, ray
            **verb group**: range, run
            **synonyms**:
            - extend:
              - **derivationally related form**: extent
              - **also see**: extend_to
            - go
            - lead
            - pass

            4. _verb_ direct or control; projects, businesses, etc.. e.g. She is running a relief operation in the Sudan.
            **hypernym**: direct
            **hyponym**: block, financier, warm_up, work
            **synonyms**:
            - operate:
              - **derivationally related form**: operator

            5. _verb_ have a particular form. e.g. the story or argument runs as follows; as the saying goes....
            **hypernym**: be
            **synonyms**:
            - go

            6. _verb_ move along, of liquids. e.g. Water flowed into the cave; the Missouri feeds into the Mississippi.
            **hypernym**: move
            **hyponym**: circulate, drain, dribble, eddy, filter, flush, gush, gutter, jet, ooze, pour, purl, run_down, run_off, run_out, seep, spill, stream, surge, swirl, tide, trickle, waste, well_out, whirl, whirlpool
            **synonyms**:
            - course
            - feed
            - flow:
              - **derivationally related form**: flowing
              - **also see**: flow_from

            7. _verb_ perform as expected when applied. e.g. The washing machine won't go unless it's plugged in; Does this old car still run well?; This old radio doesn't work anymore.
            **hyponym**: cut, double, roll, run, serve, service
            **verb group**: run, work
            **synonyms**:
            - function:
              - **antonym**: malfunction
              - **derivationally related form**: functioning
            - go
            - operate:
              - **derivationally related form**: operation
            - work

            8. _verb_ change or be different within limits. e.g. Estimates for the losses in the earthquake range as high as $2 billion; Interest rates run from 5 to 10 percent; The instruments ranged from tuba to cymbals; My students range from very bright to dull.
            **hypernym**: be
            **verb group**: extend, go, lead, pass, run
            **synonyms**:
            - range

            9. _verb_ run, stand, or compete for an office or a position. e.g. Who's running for treasurer this year?.
            **hypernym**: race, run
            **hyponym**: cross-file, register, rerun, stump, whistlestop
            **synonyms**:
            - campaign:
              - **derivationally related form**: campaigner

            10. _verb_ cause to emit recorded audio or video. e.g. They ran the tapes over and over again; I'll play you my favorite record; He never tires of playing that video.
            **verb group**: execute, play, run
            **synonyms**:
            - play

            11. _verb_ move about freely and without restraint, or act as if running around in an uncontrolled way. e.g. who are these people running around in the building?; She runs around telling everyone of her troubles; let the dogs run free.
            **hypernym**: go, locomote, move, travel
            **verb group**: run

            12. _verb_ have a tendency or disposition to do or be something; be inclined. e.g. She tends to be nervous before her lectures; These dresses run small; He inclined to corpulence.
            **hypernym**: be
            **hyponym**: gravitate, suffer, take_kindly_to
            **synonyms**:
            - be_given
            - incline:
              - **derivationally related form**: inclination
            - lean
            - tend:
              - **derivationally related form**: tendency

            13. _verb_ be operating, running or functioning. e.g. The car is still running--turn it off!.
            **hypernym**: function, go, operate, run, work
            **verb group**: function, go, operate, run, work

            14. _verb_ change from one state to another. e.g. run amok; run rogue; run riot.
            **hypernym**: become, get, go

            15. _verb_ cause to perform. e.g. run a subject; run a process.
            **hypernym**: process, treat
            **hyponym**: rerun
            **verb group**: play, run

            16. _verb_ be affected by; be subjected to. e.g. run a temperature; run a risk.
            **hypernym**: incur

            17. _verb_ continue to exist. e.g. These stories die hard; The legend of Elvis endures.
            **hypernym**: continue
            **hyponym**: carry_over, reverberate
            **verb group**: run
            **synonyms**:
            - die_hard:
              - **derivationally related form**: diehard
            - endure
            - persist:
              - **derivationally related form**: persistent
            - prevail:
              - **derivationally related form**: prevalent

            18. _verb_ occur persistently. e.g. Musical talent runs in the family.
            **hypernym**: occur
            **verb group**: die_hard, endure, persist, prevail, run

            19. _verb_ carry out a process or program, as on a computer or a machine. e.g. Run the dishwasher; run a new program on the Mac; the computer executed the instruction.
            **hypernym**: apply, enforce, implement
            **hyponym**: step
            **verb group**: play, run
            **synonyms**:
            - execute:
              - **derivationally related form**: executive

            20. _verb_ include as the content; broadcast or publicize. e.g. We ran the ad three times; This paper carries a restaurant review; All major networks carried the press conference.
            **hypernym**: broadcast, circularise, circularize, circulate, diffuse, disperse, disseminate, distribute, pass_around, propagate, spread
            **synonyms**:
            - carry

            21. _verb_ carry out. e.g. run an errand.
            **hypernym**: accomplish, action, carry_out, carry_through, execute, fulfil, fulfill

            22. _verb_ pass over, across, or through. e.g. He ran his eyes over her body; She ran her fingers along the carved figurine; He drew her hair through his fingers.
            **hyponym**: rub, thread
            **verb group**: draw, lead, run, string, thread
            **synonyms**:
            - draw
            - guide
            - pass:
              - **also see**: pass_around

            23. _verb_ cause something to pass or lead somewhere. e.g. Run the wire behind the cabinet.
            **hypernym**: make_pass, pass
            **verb group**: draw, guide, pass, range, run
            **synonyms**:
            - lead

            24. _verb_ make without a miss.
            **domain of synset topic**: athletics, sport
            **hypernym**: bring_home_the_bacon, come_through, deliver_the_goods, succeed, win

            25. _verb_ deal in illegally, such as arms or liquor.
            **domain of synset topic**: crime, criminal_offence, criminal_offense, law-breaking, offence, offense
            **hypernym**: merchandise, trade
            **verb group**: ply, run
            **synonyms**:
            - black_market

            26. _verb_ cause an animal to move fast. e.g. run the dogs.
            **hypernym**: displace, move
            **verb group**: hunt, hunt_down, run, track_down

            27. _verb_ be diffused. e.g. These dyes and colors are guaranteed not to run.
            **hypernym**: diffuse, fan_out, spread, spread_out
            **hyponym**: crock
            **verb group**: melt, melt_down, run
            **synonyms**:
            - bleed

            28. _verb_ sail before the wind.
            **hypernym**: sail

            29. _verb_ cover by running; run a certain distance. e.g. She ran 10 miles that day.
            **hypernym**: go_across, go_through, pass
            **verb group**: run

            30. _verb_ extend or continue for a certain period of time. e.g. The film runs 5 hours.
            **hypernym**: endure, last
            **synonyms**:
            - run_for

            31. _verb_ set animals loose to graze.
            **hypernym**: free, liberate, loose, release, unloose, unloosen
            **verb group**: run

            32. _verb_ keep company. e.g. the heifers run with the bulls to produce offspring.
            **hypernym**: accompany
            **synonyms**:
            - consort

            33. _verb_ run with the ball; in such sports as football.
            **domain of synset topic**: athletics, sport
            **hypernym**: run

            34. _verb_ travel rapidly, by any (unspecified) means. e.g. Run to the store!; She always runs to Italy, because she has a lover there.
            **hypernym**: go, locomote, move, travel
            **verb group**: run

            35. _verb_ travel a route regularly. e.g. Ships ply the waters near the coast.
            **hypernym**: jaunt, travel, trip
            **verb group**: black_market, run
            **synonyms**:
            - ply:
              - **derivationally related form**: plier

            36. _verb_ pursue for food or sport (as of wild animals). e.g. Goering often hunted wild boars in Poland; The dogs are running deer; The Duke hunted in these woods.
            **hypernym**: capture, catch
            **hyponym**: ambush, course, drive, falcon, ferret, forage, fowl, foxhunt, hawk, jack, jacklight, poach, rabbit, scrounge, seal, snipe, still-hunt, turtle, whale
            **verb group**: hunt, run
            **synonyms**:
            - hunt:
              - **derivationally related form**: hunting
            - hunt_down
            - track_down

            37. _verb_ compete in a race. e.g. he is running the Marathon this year; let's race and see who gets there first.
            **hypernym**: compete, contend, vie
            **hyponym**: boat-race, campaign, horse-race, place, run, show, speed_skate
            **synonyms**:
            - race:
              - **derivationally related form**: racing

            38. _verb_ progress by being changed. e.g. The speech has to go through several more drafts; run through your presentation before the meeting.
            **hypernym**: change
            **synonyms**:
            - go
            - move

            39. _verb_ reduce or cause to be reduced from a solid to a liquid state, usually by heating. e.g. melt butter; melt down gold; The wax melted in the sun.
            **hypernym**: break_up, dissolve, resolve
            **hyponym**: fuse, render, try
            **verb group**: bleed, run
            **synonyms**:
            - melt:
              - **derivationally related form**: melting
            - melt_down

            40. _verb_ come unraveled or undone as if by snagging. e.g. Her nylons were running.
            **hypernym**: break, come_apart, fall_apart, separate, split_up
            **verb group**: run, unravel
            **synonyms**:
            - ladder

            41. _verb_ become undone. e.g. the sweater unraveled.
            **hypernym**: disintegrate
            **verb group**: ladder, run
            **synonyms**:
            - unravel:
              - **derivationally related form**: unraveller"#]];
        expected.assert_eq(&info);
    }

    #[test]
    fn all_info_all_words() {
        let wndir = env::var("WNSEARCHDIR").unwrap();
        let dict = Dict::new(&PathBuf::from(wndir));
        let len = dict
            .all_words
            .iter()
            .map(|w| dict.all_info(&[w.clone()]).unwrap().len())
            .sum::<usize>();
        let expected = expect![[r#"
            54641063
        "#]];
        expected.assert_debug_eq(&len);
    }

    #[test]
    fn hover_axes() {
        let wndir = env::var("WNSEARCHDIR").unwrap();
        let dict = Dict::new(&PathBuf::from(wndir));
        let hover = dict.hover("axes").unwrap();
        let expected = expect![[r#"
            **ax** _noun_
            1. an edge tool with a heavy bladed head mounted across a handle.

            **synonyms**: axe

            **axe** _noun_
            1. an edge tool with a heavy bladed head mounted across a handle.

            **synonyms**: ax

            **axis** _noun_
            1. a straight line through a body or figure that satisfies certain conditions.
            2. the main stem or central part about which plant organs or plant parts such as branches are arranged.
            3. in World War II the alliance of Germany and Italy in 1936 which later included Japan and other nations. e.g. the Axis opposed the Allies in World War II.
            4. a group of countries in special alliance.
            5. the 2nd cervical vertebra; serves as a pivot for turning the head.
            6. the center around which something rotates.

            **synonyms**: Axis, axis of rotation, axis vertebra, bloc

            **ax** _verb_
            1. chop or split with an ax. e.g. axe wood.
            2. terminate. e.g. The NSF axed the research program and stopped funding it.

            **synonyms**: axe

            **axe** _verb_
            1. chop or split with an ax. e.g. axe wood.
            2. terminate. e.g. The NSF axed the research program and stopped funding it.

            **synonyms**: ax"#]];
        expected.assert_eq(&hover);
    }

    #[test]
    fn hover_is() {
        let wndir = env::var("WNSEARCHDIR").unwrap();
        let dict = Dict::new(&PathBuf::from(wndir));
        let hover = dict.hover("is").unwrap();
        let expected = expect![[r#"
            **i** _noun_
            1. a nonmetallic element belonging to the halogens; used especially in medicine and photography and in dyes; occurs naturally only in combination in small quantities (as in sea water or rocks).
            2. the smallest whole number or a numeral representing this number. e.g. he has the one but will need a two and three to go with it; they had lunch at one.
            3. the 9th letter of the Roman alphabet.

            **synonyms**: 1, I, ace, atomic number 53, iodin, iodine, one, single, unity

            **be** _verb_
            1. have the quality of being; (copula, used with an adjective or a predicate noun). e.g. John is rich; This is not a good answer.
            2. be identical to; be someone or something. e.g. The president of the company is John Smith; This is my house.
            3. occupy a certain position or area; be somewhere. e.g. Where is my umbrella?" "The toolshed is in the back; What is behind this behavior?.
            4. have an existence, be extant. e.g. Is there a God?.
            5. happen, occur, take place. e.g. I lost my wallet; this was during the visit to my parents' house; There were two hundred people at his funeral; There was a lot of noise in the kitchen.
            6. be identical or equivalent to. e.g. One dollar equals 1,000 rubles these days!.
            7. form or compose. e.g. This money is my only income; The stone wall was the backdrop for the performance; These constitute my entire belonging; The children made up the chorus; This sum represents my entire income for a year; These few men comprise his entire army.
            8. work in a specific place, with a specific subject, or in a specific function. e.g. He is a herpetologist; She is our resident philosopher.
            9. represent, as of a character on stage. e.g. Derek Jacobi was Hamlet.
            10. spend or use time. e.g. I may be an hour.
            11. have life, be alive. e.g. Our great leader is no more; My grandfather lived until the end of war.
            12. to remain unmolested, undisturbed, or uninterrupted -- used only in infinitive form. e.g. let her be.
            13. be priced at. e.g. These shoes cost $100.

            **synonyms**: comprise, constitute, cost, embody, equal, exist, follow, live, make up, personify, represent

            **antonyms**: differ"#]];
        expected.assert_eq(&hover);
    }

    #[test]
    fn all_info_axes() {
        let wndir = env::var("WNSEARCHDIR").unwrap();
        let dict = Dict::new(&PathBuf::from(wndir));
        let info = dict.all_info(&["axes".to_owned()]).unwrap();
        let expected = expect![[r#"
            # ax

            1. _noun_ an edge tool with a heavy bladed head mounted across a handle.
            **hypernym**: edge_tool
            **hyponym**: Dayton_ax, Dayton_axe, Western_ax, Western_axe, broadax, broadaxe, common_ax, common_axe, double-bitted_ax, double-bitted_axe, fireman's_ax, fireman's_axe, hatchet, ice_ax, ice_axe, piolet, poleax, poleaxe
            **part meronym**: ax_handle, ax_head, axe_handle, axe_head, blade, haft, helve
            **synonyms**:
            - axe

            # axe

            1. _noun_ an edge tool with a heavy bladed head mounted across a handle.
            **hypernym**: edge_tool
            **hyponym**: Dayton_ax, Dayton_axe, Western_ax, Western_axe, broadax, broadaxe, common_ax, common_axe, double-bitted_ax, double-bitted_axe, fireman's_ax, fireman's_axe, hatchet, ice_ax, ice_axe, piolet, poleax, poleaxe
            **part meronym**: ax_handle, ax_head, axe_handle, axe_head, blade, haft, helve
            **synonyms**:
            - ax

            # axis

            1. _noun_ a straight line through a body or figure that satisfies certain conditions.
            **hypernym**: line
            **hyponym**: coordinate_axis, major_axis, minor_axis, optic_axis, principal_axis, semimajor_axis, semiminor_axis

            2. _noun_ the main stem or central part about which plant organs or plant parts such as branches are arranged.
            **hypernym**: stalk, stem
            **hyponym**: rachis, spadix
            **part meronym**: stele

            3. _noun_ in World War II the alliance of Germany and Italy in 1936 which later included Japan and other nations. e.g. the Axis opposed the Allies in World War II.
            **hypernym**: alignment, alinement, alliance, coalition
            **synonyms**:
            - Axis

            4. _noun_ a group of countries in special alliance.
            **hypernym**: alignment, alinement, alliance, coalition
            **hyponym**: scheduled_territories, sterling_area, sterling_bloc
            **synonyms**:
            - bloc

            5. _noun_ the 2nd cervical vertebra; serves as a pivot for turning the head.
            **hypernym**: cervical_vertebra, neck_bone
            **part meronym**: odontoid_process
            **synonyms**:
            - axis_vertebra

            6. _noun_ the center around which something rotates.
            **hypernym**: mechanism
            **hyponym**: pin, pivot, rotor_head, rotor_shaft
            **synonyms**:
            - axis_of_rotation

            # ax

            1. _verb_ chop or split with an ax. e.g. axe wood.
            **hypernym**: chop, hack
            **synonyms**:
            - axe

            2. _verb_ terminate. e.g. The NSF axed the research program and stopped funding it.
            **hypernym**: end, terminate
            **synonyms**:
            - axe

            # axe

            1. _verb_ chop or split with an ax. e.g. axe wood.
            **hypernym**: chop, hack
            **synonyms**:
            - ax

            2. _verb_ terminate. e.g. The NSF axed the research program and stopped funding it.
            **hypernym**: end, terminate
            **synonyms**:
            - ax"#]];
        expected.assert_eq(&info);
    }

    #[test]
    fn all_info_multiple_words() {
        let wndir = env::var("WNSEARCHDIR").unwrap();
        let dict = Dict::new(&PathBuf::from(wndir));
        let info = dict
            .all_info(&["axes".to_owned(), "beta".to_owned()])
            .unwrap();
        let expected = expect![[r#"
            # ax

            1. _noun_ an edge tool with a heavy bladed head mounted across a handle.
            **hypernym**: edge_tool
            **hyponym**: Dayton_ax, Dayton_axe, Western_ax, Western_axe, broadax, broadaxe, common_ax, common_axe, double-bitted_ax, double-bitted_axe, fireman's_ax, fireman's_axe, hatchet, ice_ax, ice_axe, piolet, poleax, poleaxe
            **part meronym**: ax_handle, ax_head, axe_handle, axe_head, blade, haft, helve
            **synonyms**:
            - axe

            # axe

            1. _noun_ an edge tool with a heavy bladed head mounted across a handle.
            **hypernym**: edge_tool
            **hyponym**: Dayton_ax, Dayton_axe, Western_ax, Western_axe, broadax, broadaxe, common_ax, common_axe, double-bitted_ax, double-bitted_axe, fireman's_ax, fireman's_axe, hatchet, ice_ax, ice_axe, piolet, poleax, poleaxe
            **part meronym**: ax_handle, ax_head, axe_handle, axe_head, blade, haft, helve
            **synonyms**:
            - ax

            # axis

            1. _noun_ a straight line through a body or figure that satisfies certain conditions.
            **hypernym**: line
            **hyponym**: coordinate_axis, major_axis, minor_axis, optic_axis, principal_axis, semimajor_axis, semiminor_axis

            2. _noun_ the main stem or central part about which plant organs or plant parts such as branches are arranged.
            **hypernym**: stalk, stem
            **hyponym**: rachis, spadix
            **part meronym**: stele

            3. _noun_ in World War II the alliance of Germany and Italy in 1936 which later included Japan and other nations. e.g. the Axis opposed the Allies in World War II.
            **hypernym**: alignment, alinement, alliance, coalition
            **synonyms**:
            - Axis

            4. _noun_ a group of countries in special alliance.
            **hypernym**: alignment, alinement, alliance, coalition
            **hyponym**: scheduled_territories, sterling_area, sterling_bloc
            **synonyms**:
            - bloc

            5. _noun_ the 2nd cervical vertebra; serves as a pivot for turning the head.
            **hypernym**: cervical_vertebra, neck_bone
            **part meronym**: odontoid_process
            **synonyms**:
            - axis_vertebra

            6. _noun_ the center around which something rotates.
            **hypernym**: mechanism
            **hyponym**: pin, pivot, rotor_head, rotor_shaft
            **synonyms**:
            - axis_of_rotation

            # ax

            1. _verb_ chop or split with an ax. e.g. axe wood.
            **hypernym**: chop, hack
            **synonyms**:
            - axe

            2. _verb_ terminate. e.g. The NSF axed the research program and stopped funding it.
            **hypernym**: end, terminate
            **synonyms**:
            - axe

            # axe

            1. _verb_ chop or split with an ax. e.g. axe wood.
            **hypernym**: chop, hack
            **synonyms**:
            - ax

            2. _verb_ terminate. e.g. The NSF axed the research program and stopped funding it.
            **hypernym**: end, terminate
            **synonyms**:
            - ax

            # beta

            1. _noun_ the 2nd letter of the Greek alphabet.
            **hypernym**: alphabetic_character, letter, letter_of_the_alphabet
            **member holonym**: Greek_alphabet

            2. _noun_ beets.
            **hypernym**: Chenopodiaceae, caryophylloid_dicot_genus, family_Chenopodiaceae, goosefoot_family
            **member meronym**: Beta_vulgaris, beet, common_beet
            **synonyms**:
            - Beta
            - genus_Beta

            # beta

            1. _adjective_ second in order of importance. e.g. the candidate, considered a beta male, was perceived to be unable to lead his party to victory.
            **similar to**: important, of_import

            2. _adjective_ preliminary or testing stage of a software or hardware product. e.g. a beta version; beta software.
            **similar to**: explorative, exploratory"#]];
        expected.assert_eq(&info);
    }

    fn check_get_words(content: &str, expected: Expect) {
        let words = (0..content.len())
            .map(|i| (i, get_words_from_content(content, 0, i)))
            .map(|(i, ret)| format!("{i}: {ret:?}"))
            .collect::<Vec<_>>();
        expected.assert_debug_eq(&words)
    }

    #[test]
    fn get_word() {
        let text = "runner";
        let expected = expect![[r#"
            [
                "0: [\"runner\"]",
                "1: [\"runner\"]",
                "2: [\"runner\"]",
                "3: [\"runner\"]",
                "4: [\"runner\"]",
                "5: [\"runner\"]",
            ]
        "#]];
        check_get_words(text, expected)
    }

    #[test]
    fn get_words_with_spaces() {
        let text = "a runner runs";
        let expected = expect![[r#"
            [
                "0: [\"a\", \"a_runner\", \"a_runner_runs\"]",
                "1: []",
                "2: [\"runner\", \"runner_runs\"]",
                "3: [\"runner\", \"runner_runs\"]",
                "4: [\"runner\", \"runner_runs\"]",
                "5: [\"runner\", \"runner_runs\"]",
                "6: [\"runner\", \"runner_runs\"]",
                "7: [\"runner\", \"runner_runs\"]",
                "8: []",
                "9: [\"runs\"]",
                "10: [\"runs\"]",
                "11: [\"runs\"]",
                "12: [\"runs\"]",
            ]
        "#]];
        check_get_words(text, expected)
    }

    #[test]
    fn get_words_with_spaces_and_punctuation() {
        let text = "new, for sale.";
        let expected = expect![[r#"
            [
                "0: [\"new\"]",
                "1: [\"new\"]",
                "2: [\"new\"]",
                "3: []",
                "4: []",
                "5: [\"for\", \"for_sale\", \"for_sale.\"]",
                "6: [\"for\", \"for_sale\", \"for_sale.\"]",
                "7: [\"for\", \"for_sale\", \"for_sale.\"]",
                "8: []",
                "9: [\"sale\", \"sale.\"]",
                "10: [\"sale\", \"sale.\"]",
                "11: [\"sale\", \"sale.\"]",
                "12: [\"sale\", \"sale.\"]",
                "13: [\"sale\", \"sale.\"]",
            ]
        "#]];
        check_get_words(text, expected)
    }

    #[test]
    fn get_words_underscore() {
        let text = "living_thing";
        let expected = expect![[r#"
            [
                "0: [\"living_thing\"]",
                "1: [\"living_thing\"]",
                "2: [\"living_thing\"]",
                "3: [\"living_thing\"]",
                "4: [\"living_thing\"]",
                "5: [\"living_thing\"]",
                "6: [\"living_thing\"]",
                "7: [\"living_thing\"]",
                "8: [\"living_thing\"]",
                "9: [\"living_thing\"]",
                "10: [\"living_thing\"]",
                "11: [\"living_thing\"]",
            ]
        "#]];
        check_get_words(text, expected)
    }

    #[test]
    fn get_words_two_words() {
        let text = "living thing";
        let expected = expect![[r#"
            [
                "0: [\"living\", \"living_thing\"]",
                "1: [\"living\", \"living_thing\"]",
                "2: [\"living\", \"living_thing\"]",
                "3: [\"living\", \"living_thing\"]",
                "4: [\"living\", \"living_thing\"]",
                "5: [\"living\", \"living_thing\"]",
                "6: []",
                "7: [\"thing\"]",
                "8: [\"thing\"]",
                "9: [\"thing\"]",
                "10: [\"thing\"]",
                "11: [\"thing\"]",
            ]
        "#]];
        check_get_words(text, expected)
    }

    #[test]
    fn get_words_apostrophe() {
        let text = "'hood";
        let expected = expect![[r#"
            [
                "0: [\"hood\", \"'hood\"]",
                "1: [\"hood\", \"'hood\"]",
                "2: [\"hood\", \"'hood\"]",
                "3: [\"hood\", \"'hood\"]",
                "4: [\"hood\", \"'hood\"]",
            ]
        "#]];
        check_get_words(text, expected)
    }

    #[test]
    fn get_words_apostrophes() {
        let text = "'hood'";
        let expected = expect![[r#"
            [
                "0: [\"'hood\", \"hood\", \"hood'\", \"'hood'\"]",
                "1: [\"'hood\", \"hood\", \"hood'\", \"'hood'\"]",
                "2: [\"'hood\", \"hood\", \"hood'\", \"'hood'\"]",
                "3: [\"'hood\", \"hood\", \"hood'\", \"'hood'\"]",
                "4: [\"'hood\", \"hood\", \"hood'\", \"'hood'\"]",
                "5: [\"'hood\", \"hood\", \"hood'\", \"'hood'\"]",
            ]
        "#]];
        check_get_words(text, expected)
    }

    #[test]
    fn get_words_all_punctuations() {
        let wndir = env::var("WNSEARCHDIR").unwrap();
        let wn = WordNet::new(&PathBuf::from(wndir));
        let words = wn
            .all_words()
            .into_iter()
            .filter_map(|w| {
                let mut non_alpha_chars =
                    w.chars().filter(|c| !c.is_alphabetic()).collect::<Vec<_>>();
                non_alpha_chars.sort();
                non_alpha_chars.dedup();
                if non_alpha_chars.is_empty() {
                    None
                } else {
                    Some((non_alpha_chars, w))
                }
            })
            .fold(BTreeMap::new(), |mut acc, (c, w)| {
                acc.insert(c, w);
                acc
            })
            .into_values()
            .map(|word| {
                let words = get_words_from_content(&word, 0, 0);
                let found = words.contains(&word);
                (word, words, found)
            })
            .collect::<Vec<_>>();
        let expected = expect![[r#"
            [
                (
                    "ta'ziyeh",
                    [
                        "ta'ziyeh",
                    ],
                    true,
                ),
                (
                    "will-o'-the-wisp",
                    [
                        "will-o'-the-wisp",
                    ],
                    true,
                ),
                (
                    "st.-bruno's-lily",
                    [
                        "st.-bruno's-lily",
                    ],
                    true,
                ),
                (
                    "wine-maker's_yeast",
                    [
                        "wine-maker's_yeast",
                    ],
                    true,
                ),
                (
                    "st._peter's_wreath",
                    [
                        "st._peter's_wreath",
                    ],
                    true,
                ),
                (
                    "brodmann's_area_17",
                    [
                        "brodmann's_area_17",
                    ],
                    true,
                ),
                (
                    "young's_modulus",
                    [
                        "young's_modulus",
                    ],
                    true,
                ),
                (
                    "zig-zag",
                    [
                        "zig-zag",
                    ],
                    true,
                ),
                (
                    ".22-calibre",
                    [
                        "22-calibre",
                        ".22-calibre",
                    ],
                    true,
                ),
                (
                    ".38-calibre",
                    [
                        "38-calibre",
                        ".38-calibre",
                    ],
                    true,
                ),
                (
                    ".45-calibre",
                    [
                        "45-calibre",
                        ".45-calibre",
                    ],
                    true,
                ),
                (
                    "wrangell-st._elias_national_park",
                    [
                        "wrangell-st._elias_national_park",
                    ],
                    true,
                ),
                (
                    "10-membered",
                    [
                        "10-membered",
                    ],
                    true,
                ),
                (
                    "401-k",
                    [
                        "401-k",
                    ],
                    true,
                ),
                (
                    "401-k_plan",
                    [
                        "401-k_plan",
                    ],
                    true,
                ),
                (
                    "k-dur_20",
                    [
                        "k-dur_20",
                    ],
                    true,
                ),
                (
                    "v-1",
                    [
                        "v-1",
                    ],
                    true,
                ),
                (
                    "iodine-125",
                    [
                        "iodine-125",
                    ],
                    true,
                ),
                (
                    "12-tone_system",
                    [
                        "12-tone_system",
                    ],
                    true,
                ),
                (
                    "iodine-131",
                    [
                        "iodine-131",
                    ],
                    true,
                ),
                (
                    "carbon-14_dating",
                    [
                        "carbon-14_dating",
                    ],
                    true,
                ),
                (
                    "18-karat_gold",
                    [
                        "18-karat_gold",
                    ],
                    true,
                ),
                (
                    "9-11",
                    [
                        "9-11",
                    ],
                    true,
                ),
                (
                    "m-1_rifle",
                    [
                        "m-1_rifle",
                    ],
                    true,
                ),
                (
                    "r-2",
                    [
                        "r-2",
                    ],
                    true,
                ),
                (
                    "24-karat_gold",
                    [
                        "24-karat_gold",
                    ],
                    true,
                ),
                (
                    "b-52",
                    [
                        "b-52",
                    ],
                    true,
                ),
                (
                    "thorium-228",
                    [
                        "thorium-228",
                    ],
                    true,
                ),
                (
                    "guided_bomb_unit-28",
                    [
                        "guided_bomb_unit-28",
                    ],
                    true,
                ),
                (
                    "cox-2_inhibitor",
                    [
                        "cox-2_inhibitor",
                    ],
                    true,
                ),
                (
                    "omega-3",
                    [
                        "omega-3",
                    ],
                    true,
                ),
                (
                    "5-hydroxy-3-methylglutaryl-coenzyme_a_reductase",
                    [
                        "5-hydroxy-3-methylglutaryl-coenzyme_a_reductase",
                    ],
                    true,
                ),
                (
                    "omega-3_fatty_acid",
                    [
                        "omega-3_fatty_acid",
                    ],
                    true,
                ),
                (
                    "4-membered",
                    [
                        "4-membered",
                    ],
                    true,
                ),
                (
                    "5-membered",
                    [
                        "5-membered",
                    ],
                    true,
                ),
                (
                    "omega-6",
                    [
                        "omega-6",
                    ],
                    true,
                ),
                (
                    "omega-6_fatty_acid",
                    [
                        "omega-6_fatty_acid",
                    ],
                    true,
                ),
                (
                    "7-membered",
                    [
                        "7-membered",
                    ],
                    true,
                ),
                (
                    "8-membered",
                    [
                        "8-membered",
                    ],
                    true,
                ),
                (
                    "v-8_juice",
                    [
                        "v-8_juice",
                    ],
                    true,
                ),
                (
                    "9-membered",
                    [
                        "9-membered",
                    ],
                    true,
                ),
                (
                    "zollinger-ellison_syndrome",
                    [
                        "zollinger-ellison_syndrome",
                    ],
                    true,
                ),
                (
                    "w.m.d.",
                    [
                        "w.m.d",
                        "w.m.d.",
                    ],
                    true,
                ),
                (
                    "sept._11",
                    [
                        "sept._11",
                    ],
                    true,
                ),
                (
                    ".22",
                    [
                        "22",
                        ".22",
                    ],
                    true,
                ),
                (
                    ".22_calibre",
                    [
                        "22_calibre",
                        ".22_calibre",
                    ],
                    true,
                ),
                (
                    ".38_calibre",
                    [
                        "38_calibre",
                        ".38_calibre",
                    ],
                    true,
                ),
                (
                    ".45_calibre",
                    [
                        "45_calibre",
                        ".45_calibre",
                    ],
                    true,
                ),
                (
                    "winston_s._churchill",
                    [
                        "winston_s._churchill",
                    ],
                    true,
                ),
                (
                    "tcp/ip",
                    [
                        "tcp/ip",
                    ],
                    true,
                ),
                (
                    "20/20",
                    [
                        "20/20",
                    ],
                    true,
                ),
                (
                    "9/11",
                    [
                        "9/11",
                    ],
                    true,
                ),
                (
                    "24/7",
                    [
                        "24/7",
                    ],
                    true,
                ),
                (
                    "transmission_control_protocol/internet_protocol",
                    [
                        "transmission_control_protocol/internet_protocol",
                    ],
                    true,
                ),
                (
                    "0",
                    [
                        "0",
                    ],
                    true,
                ),
                (
                    "110th",
                    [
                        "110th",
                    ],
                    true,
                ),
                (
                    "120th",
                    [
                        "120th",
                    ],
                    true,
                ),
                (
                    "1820s",
                    [
                        "1820s",
                    ],
                    true,
                ),
                (
                    "1920s",
                    [
                        "1920s",
                    ],
                    true,
                ),
                (
                    "atomic_number_102",
                    [
                        "atomic_number_102",
                    ],
                    true,
                ),
                (
                    "130th",
                    [
                        "130th",
                    ],
                    true,
                ),
                (
                    "1530s",
                    [
                        "1530s",
                    ],
                    true,
                ),
                (
                    "1830s",
                    [
                        "1830s",
                    ],
                    true,
                ),
                (
                    "1930s",
                    [
                        "1930s",
                    ],
                    true,
                ),
                (
                    "atomic_number_103",
                    [
                        "atomic_number_103",
                    ],
                    true,
                ),
                (
                    "140th",
                    [
                        "140th",
                    ],
                    true,
                ),
                (
                    "1840s",
                    [
                        "1840s",
                    ],
                    true,
                ),
                (
                    "1940s",
                    [
                        "1940s",
                    ],
                    true,
                ),
                (
                    "element_104",
                    [
                        "element_104",
                    ],
                    true,
                ),
                (
                    "150th",
                    [
                        "150th",
                    ],
                    true,
                ),
                (
                    "1750s",
                    [
                        "1750s",
                    ],
                    true,
                ),
                (
                    "1850s",
                    [
                        "1850s",
                    ],
                    true,
                ),
                (
                    "1950s",
                    [
                        "1950s",
                    ],
                    true,
                ),
                (
                    "element_105",
                    [
                        "element_105",
                    ],
                    true,
                ),
                (
                    "160th",
                    [
                        "160th",
                    ],
                    true,
                ),
                (
                    "1760s",
                    [
                        "1760s",
                    ],
                    true,
                ),
                (
                    "1860s",
                    [
                        "1860s",
                    ],
                    true,
                ),
                (
                    "1960s",
                    [
                        "1960s",
                    ],
                    true,
                ),
                (
                    "element_106",
                    [
                        "element_106",
                    ],
                    true,
                ),
                (
                    "1770s",
                    [
                        "1770s",
                    ],
                    true,
                ),
                (
                    "1870s",
                    [
                        "1870s",
                    ],
                    true,
                ),
                (
                    "1970s",
                    [
                        "1970s",
                    ],
                    true,
                ),
                (
                    "element_107",
                    [
                        "element_107",
                    ],
                    true,
                ),
                (
                    "1880s",
                    [
                        "1880s",
                    ],
                    true,
                ),
                (
                    "1980s",
                    [
                        "1980s",
                    ],
                    true,
                ),
                (
                    "element_108",
                    [
                        "element_108",
                    ],
                    true,
                ),
                (
                    "1990s",
                    [
                        "1990s",
                    ],
                    true,
                ),
                (
                    "element_109",
                    [
                        "element_109",
                    ],
                    true,
                ),
                (
                    "element_110",
                    [
                        "element_110",
                    ],
                    true,
                ),
                (
                    "20th",
                    [
                        "20th",
                    ],
                    true,
                ),
                (
                    "january_20",
                    [
                        "january_20",
                    ],
                    true,
                ),
                (
                    "30th",
                    [
                        "30th",
                    ],
                    true,
                ),
                (
                    "u308",
                    [
                        "u308",
                    ],
                    true,
                ),
                (
                    "atomic_number_30",
                    [
                        "atomic_number_30",
                    ],
                    true,
                ),
                (
                    "40th",
                    [
                        "40th",
                    ],
                    true,
                ),
                (
                    "atomic_number_40",
                    [
                        "atomic_number_40",
                    ],
                    true,
                ),
                (
                    "50th",
                    [
                        "50th",
                    ],
                    true,
                ),
                (
                    "atomic_number_50",
                    [
                        "atomic_number_50",
                    ],
                    true,
                ),
                (
                    "60th",
                    [
                        "60th",
                    ],
                    true,
                ),
                (
                    "cobalt_60",
                    [
                        "cobalt_60",
                    ],
                    true,
                ),
                (
                    "70th",
                    [
                        "70th",
                    ],
                    true,
                ),
                (
                    "atomic_number_70",
                    [
                        "atomic_number_70",
                    ],
                    true,
                ),
                (
                    "80th",
                    [
                        "80th",
                    ],
                    true,
                ),
                (
                    "atomic_number_80",
                    [
                        "atomic_number_80",
                    ],
                    true,
                ),
                (
                    "90th",
                    [
                        "90th",
                    ],
                    true,
                ),
                (
                    "strontium_90",
                    [
                        "strontium_90",
                    ],
                    true,
                ),
                (
                    "ut1",
                    [
                        "ut1",
                    ],
                    true,
                ),
                (
                    "21st",
                    [
                        "21st",
                    ],
                    true,
                ),
                (
                    "125th",
                    [
                        "125th",
                    ],
                    true,
                ),
                (
                    "1728",
                    [
                        "1728",
                    ],
                    true,
                ),
                (
                    "war_of_1812",
                    [
                        "war_of_1812",
                    ],
                    true,
                ),
                (
                    "vitamin_b12",
                    [
                        "vitamin_b12",
                    ],
                    true,
                ),
                (
                    "31st",
                    [
                        "31st",
                    ],
                    true,
                ),
                (
                    "135th",
                    [
                        "135th",
                    ],
                    true,
                ),
                (
                    "cesium_137",
                    [
                        "cesium_137",
                    ],
                    true,
                ),
                (
                    "element_113",
                    [
                        "element_113",
                    ],
                    true,
                ),
                (
                    "41st",
                    [
                        "41st",
                    ],
                    true,
                ),
                (
                    "145th",
                    [
                        "145th",
                    ],
                    true,
                ),
                (
                    "8_may_1945",
                    [
                        "8_may_1945",
                    ],
                    true,
                ),
                (
                    "15_august_1945",
                    [
                        "15_august_1945",
                    ],
                    true,
                ),
                (
                    "6_june_1944",
                    [
                        "6_june_1944",
                    ],
                    true,
                ),
                (
                    "june_14",
                    [
                        "june_14",
                    ],
                    true,
                ),
                (
                    "51",
                    [
                        "51",
                    ],
                    true,
                ),
                (
                    "165th",
                    [
                        "165th",
                    ],
                    true,
                ),
                (
                    "175th",
                    [
                        "175th",
                    ],
                    true,
                ),
                (
                    "element_115",
                    [
                        "element_115",
                    ],
                    true,
                ),
                (
                    "61",
                    [
                        "61",
                    ],
                    true,
                ),
                (
                    "element_116",
                    [
                        "element_116",
                    ],
                    true,
                ),
                (
                    "71",
                    [
                        "71",
                    ],
                    true,
                ),
                (
                    "september_17",
                    [
                        "september_17",
                    ],
                    true,
                ),
                (
                    "81",
                    [
                        "81",
                    ],
                    true,
                ),
                (
                    "atomic_number_81",
                    [
                        "atomic_number_81",
                    ],
                    true,
                ),
                (
                    "91",
                    [
                        "91",
                    ],
                    true,
                ),
                (
                    "march_19",
                    [
                        "march_19",
                    ],
                    true,
                ),
                (
                    "world_war_1",
                    [
                        "world_war_1",
                    ],
                    true,
                ),
                (
                    "y2k",
                    [
                        "y2k",
                    ],
                    true,
                ),
                (
                    "32nd",
                    [
                        "32nd",
                    ],
                    true,
                ),
                (
                    "uranium_235",
                    [
                        "uranium_235",
                    ],
                    true,
                ),
                (
                    "uranium_238",
                    [
                        "uranium_238",
                    ],
                    true,
                ),
                (
                    "plutonium_239",
                    [
                        "plutonium_239",
                    ],
                    true,
                ),
                (
                    "june_23",
                    [
                        "june_23",
                    ],
                    true,
                ),
                (
                    "42nd",
                    [
                        "42nd",
                    ],
                    true,
                ),
                (
                    "october_24",
                    [
                        "october_24",
                    ],
                    true,
                ),
                (
                    "52",
                    [
                        "52",
                    ],
                    true,
                ),
                (
                    "ponte_25_de_abril",
                    [
                        "ponte_25_de_abril",
                    ],
                    true,
                ),
                (
                    "c2h6",
                    [
                        "c2h6",
                    ],
                    true,
                ),
                (
                    "atomic_number_62",
                    [
                        "atomic_number_62",
                    ],
                    true,
                ),
                (
                    "72",
                    [
                        "72",
                    ],
                    true,
                ),
                (
                    "atomic_number_72",
                    [
                        "atomic_number_72",
                    ],
                    true,
                ),
                (
                    "82",
                    [
                        "82",
                    ],
                    true,
                ),
                (
                    "atomic_number_82",
                    [
                        "atomic_number_82",
                    ],
                    true,
                ),
                (
                    "92",
                    [
                        "92",
                    ],
                    true,
                ),
                (
                    "september_29",
                    [
                        "september_29",
                    ],
                    true,
                ),
                (
                    "y2k_compliant",
                    [
                        "y2k_compliant",
                    ],
                    true,
                ),
                (
                    "m3",
                    [
                        "m3",
                    ],
                    true,
                ),
                (
                    "43rd",
                    [
                        "43rd",
                    ],
                    true,
                ),
                (
                    "atomic_number_43",
                    [
                        "atomic_number_43",
                    ],
                    true,
                ),
                (
                    "53",
                    [
                        "53",
                    ],
                    true,
                ),
                (
                    "365_days",
                    [
                        "365_days",
                    ],
                    true,
                ),
                (
                    "atomic_number_53",
                    [
                        "atomic_number_53",
                    ],
                    true,
                ),
                (
                    "63",
                    [
                        "63",
                    ],
                    true,
                ),
                (
                    "atomic_number_63",
                    [
                        "atomic_number_63",
                    ],
                    true,
                ),
                (
                    "73",
                    [
                        "73",
                    ],
                    true,
                ),
                (
                    "atomic_number_73",
                    [
                        "atomic_number_73",
                    ],
                    true,
                ),
                (
                    "83",
                    [
                        "83",
                    ],
                    true,
                ),
                (
                    "atomic_number_83",
                    [
                        "atomic_number_83",
                    ],
                    true,
                ),
                (
                    "93",
                    [
                        "93",
                    ],
                    true,
                ),
                (
                    "atomic_number_93",
                    [
                        "atomic_number_93",
                    ],
                    true,
                ),
                (
                    "vitamin_k3",
                    [
                        "vitamin_k3",
                    ],
                    true,
                ),
                (
                    "cd4",
                    [
                        "cd4",
                    ],
                    true,
                ),
                (
                    "54",
                    [
                        "54",
                    ],
                    true,
                ),
                (
                    "atomic_number_54",
                    [
                        "atomic_number_54",
                    ],
                    true,
                ),
                (
                    "64th",
                    [
                        "64th",
                    ],
                    true,
                ),
                (
                    "ru_486",
                    [
                        "ru_486",
                    ],
                    true,
                ),
                (
                    "atomic_number_64",
                    [
                        "atomic_number_64",
                    ],
                    true,
                ),
                (
                    "74",
                    [
                        "74",
                    ],
                    true,
                ),
                (
                    "atomic_number_74",
                    [
                        "atomic_number_74",
                    ],
                    true,
                ),
                (
                    "84",
                    [
                        "84",
                    ],
                    true,
                ),
                (
                    "atomic_number_84",
                    [
                        "atomic_number_84",
                    ],
                    true,
                ),
                (
                    "94",
                    [
                        "94",
                    ],
                    true,
                ),
                (
                    "atomic_number_94",
                    [
                        "atomic_number_94",
                    ],
                    true,
                ),
                (
                    "july_4",
                    [
                        "july_4",
                    ],
                    true,
                ),
                (
                    "5th",
                    [
                        "5th",
                    ],
                    true,
                ),
                (
                    "65th",
                    [
                        "65th",
                    ],
                    true,
                ),
                (
                    "atomic_number_65",
                    [
                        "atomic_number_65",
                    ],
                    true,
                ),
                (
                    "75th",
                    [
                        "75th",
                    ],
                    true,
                ),
                (
                    "atomic_number_75",
                    [
                        "atomic_number_75",
                    ],
                    true,
                ),
                (
                    "85th",
                    [
                        "85th",
                    ],
                    true,
                ),
                (
                    "atomic_number_85",
                    [
                        "atomic_number_85",
                    ],
                    true,
                ),
                (
                    "95th",
                    [
                        "95th",
                    ],
                    true,
                ),
                (
                    "atomic_number_95",
                    [
                        "atomic_number_95",
                    ],
                    true,
                ),
                (
                    "november_5",
                    [
                        "november_5",
                    ],
                    true,
                ),
                (
                    "6th",
                    [
                        "6th",
                    ],
                    true,
                ),
                (
                    "76",
                    [
                        "76",
                    ],
                    true,
                ),
                (
                    "atomic_number_76",
                    [
                        "atomic_number_76",
                    ],
                    true,
                ),
                (
                    "86",
                    [
                        "86",
                    ],
                    true,
                ),
                (
                    "atomic_number_86",
                    [
                        "atomic_number_86",
                    ],
                    true,
                ),
                (
                    "96",
                    [
                        "96",
                    ],
                    true,
                ),
                (
                    "atomic_number_96",
                    [
                        "atomic_number_96",
                    ],
                    true,
                ),
                (
                    "vitamin_b6",
                    [
                        "vitamin_b6",
                    ],
                    true,
                ),
                (
                    "7th",
                    [
                        "7th",
                    ],
                    true,
                ),
                (
                    "87",
                    [
                        "87",
                    ],
                    true,
                ),
                (
                    "atomic_number_87",
                    [
                        "atomic_number_87",
                    ],
                    true,
                ),
                (
                    "97",
                    [
                        "97",
                    ],
                    true,
                ),
                (
                    "atomic_number_97",
                    [
                        "atomic_number_97",
                    ],
                    true,
                ),
                (
                    "atomic_number_77",
                    [
                        "atomic_number_77",
                    ],
                    true,
                ),
                (
                    "cd8",
                    [
                        "cd8",
                    ],
                    true,
                ),
                (
                    "98",
                    [
                        "98",
                    ],
                    true,
                ),
                (
                    "atomic_number_98",
                    [
                        "atomic_number_98",
                    ],
                    true,
                ),
                (
                    "december_8",
                    [
                        "december_8",
                    ],
                    true,
                ),
                (
                    "9th",
                    [
                        "9th",
                    ],
                    true,
                ),
                (
                    "atomic_number_99",
                    [
                        "atomic_number_99",
                    ],
                    true,
                ),
                (
                    "zygophyllum_fabago",
                    [
                        "zygophyllum_fabago",
                    ],
                    true,
                ),
            ]
        "#]];
        expected.assert_debug_eq(&words);
    }

    #[test]
    fn complete_spaces() {
        let wndir = env::var("WNSEARCHDIR").unwrap();
        let dict = Dict::new(&PathBuf::from(wndir));
        let words = dict.complete(&"living".to_owned(), 10);
        let expected = expect![[r#"
            [
                CompletionItem {
                    label: "living",
                    label_details: None,
                    kind: None,
                    detail: None,
                    documentation: None,
                    deprecated: None,
                    preselect: None,
                    sort_text: None,
                    filter_text: None,
                    insert_text: None,
                    insert_text_format: None,
                    insert_text_mode: None,
                    text_edit: None,
                    additional_text_edits: None,
                    command: None,
                    commit_characters: None,
                    data: None,
                    tags: None,
                },
                CompletionItem {
                    label: "living-room",
                    label_details: None,
                    kind: None,
                    detail: None,
                    documentation: None,
                    deprecated: None,
                    preselect: None,
                    sort_text: None,
                    filter_text: None,
                    insert_text: None,
                    insert_text_format: None,
                    insert_text_mode: None,
                    text_edit: None,
                    additional_text_edits: None,
                    command: None,
                    commit_characters: None,
                    data: None,
                    tags: None,
                },
                CompletionItem {
                    label: "living_accommodations",
                    label_details: None,
                    kind: None,
                    detail: None,
                    documentation: None,
                    deprecated: None,
                    preselect: None,
                    sort_text: None,
                    filter_text: None,
                    insert_text: Some(
                        "living accommodations",
                    ),
                    insert_text_format: None,
                    insert_text_mode: None,
                    text_edit: None,
                    additional_text_edits: None,
                    command: None,
                    commit_characters: None,
                    data: None,
                    tags: None,
                },
                CompletionItem {
                    label: "living_arrangement",
                    label_details: None,
                    kind: None,
                    detail: None,
                    documentation: None,
                    deprecated: None,
                    preselect: None,
                    sort_text: None,
                    filter_text: None,
                    insert_text: Some(
                        "living arrangement",
                    ),
                    insert_text_format: None,
                    insert_text_mode: None,
                    text_edit: None,
                    additional_text_edits: None,
                    command: None,
                    commit_characters: None,
                    data: None,
                    tags: None,
                },
                CompletionItem {
                    label: "living_dead",
                    label_details: None,
                    kind: None,
                    detail: None,
                    documentation: None,
                    deprecated: None,
                    preselect: None,
                    sort_text: None,
                    filter_text: None,
                    insert_text: Some(
                        "living dead",
                    ),
                    insert_text_format: None,
                    insert_text_mode: None,
                    text_edit: None,
                    additional_text_edits: None,
                    command: None,
                    commit_characters: None,
                    data: None,
                    tags: None,
                },
                CompletionItem {
                    label: "living_death",
                    label_details: None,
                    kind: None,
                    detail: None,
                    documentation: None,
                    deprecated: None,
                    preselect: None,
                    sort_text: None,
                    filter_text: None,
                    insert_text: Some(
                        "living death",
                    ),
                    insert_text_format: None,
                    insert_text_mode: None,
                    text_edit: None,
                    additional_text_edits: None,
                    command: None,
                    commit_characters: None,
                    data: None,
                    tags: None,
                },
                CompletionItem {
                    label: "living_granite",
                    label_details: None,
                    kind: None,
                    detail: None,
                    documentation: None,
                    deprecated: None,
                    preselect: None,
                    sort_text: None,
                    filter_text: None,
                    insert_text: Some(
                        "living granite",
                    ),
                    insert_text_format: None,
                    insert_text_mode: None,
                    text_edit: None,
                    additional_text_edits: None,
                    command: None,
                    commit_characters: None,
                    data: None,
                    tags: None,
                },
                CompletionItem {
                    label: "living_quarters",
                    label_details: None,
                    kind: None,
                    detail: None,
                    documentation: None,
                    deprecated: None,
                    preselect: None,
                    sort_text: None,
                    filter_text: None,
                    insert_text: Some(
                        "living quarters",
                    ),
                    insert_text_format: None,
                    insert_text_mode: None,
                    text_edit: None,
                    additional_text_edits: None,
                    command: None,
                    commit_characters: None,
                    data: None,
                    tags: None,
                },
                CompletionItem {
                    label: "living_rock",
                    label_details: None,
                    kind: None,
                    detail: None,
                    documentation: None,
                    deprecated: None,
                    preselect: None,
                    sort_text: None,
                    filter_text: None,
                    insert_text: Some(
                        "living rock",
                    ),
                    insert_text_format: None,
                    insert_text_mode: None,
                    text_edit: None,
                    additional_text_edits: None,
                    command: None,
                    commit_characters: None,
                    data: None,
                    tags: None,
                },
                CompletionItem {
                    label: "living_room",
                    label_details: None,
                    kind: None,
                    detail: None,
                    documentation: None,
                    deprecated: None,
                    preselect: None,
                    sort_text: None,
                    filter_text: None,
                    insert_text: Some(
                        "living room",
                    ),
                    insert_text_format: None,
                    insert_text_mode: None,
                    text_edit: None,
                    additional_text_edits: None,
                    command: None,
                    commit_characters: None,
                    data: None,
                    tags: None,
                },
            ]
        "#]];
        expected.assert_debug_eq(&words);
    }
}
