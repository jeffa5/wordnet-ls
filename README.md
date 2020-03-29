# LLS - Language Language-Server

Language servers provide a convenient solution to the problem of language
support for various editors.

Language servers are commonly used for programming languages but why not
natural language too?

This project aims to provide an example of a language server for English, using
a dictionary for support.

## Actions

- [x] hover shows meaning of the word
- [ ] linting for spelling
- [ ] some way of doing synonyms and other things
- [ ] completion for words

## Installation

Currently, the main way to install LLS is by cloning the repo and running

```sh
cargo install --path . --force
```

or

```sh
make install
```

This adds the binary `lls` to the rust bin location.

## Configuration

Configuration is editor dependent so please see instructions for your editor.
