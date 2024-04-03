# LLS - Language Language-Server

_This is alpha software, use at your own risk_

Language servers provide a convenient solution to the problem of language
support for various editors.

Language servers are commonly used for programming languages but why not
natural language too?

This project aims to provide an example of a language server for English, using
wordnet for support.

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

This adds the binary `lls` to the rust bin location.

## Configuration

You'll need to download a copy of
[wordnet](https://wordnet.princeton.edu/download/current-version). The tested
version is 3.1.

To configure the location of the wordnet dictionary set the `initializationOptions` as:

```json
{
  "wordnet": "<location>",
}
```

Home dir should get expanded if needed.

## WordNet

For more information about the WordNet database see [here](https://wordnet.princeton.edu/).
