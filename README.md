A toy .ttf renderer written in Rust.

Very much work in progress.

Currently working on: Parsing .ttf files

* Translating all table fields into Rust types

    * Defining options as enums

    * Removing implied fields (e.g. `count` fields are implied by `Vec` lengths)

    * Wrapping strings into encoding-aware types
