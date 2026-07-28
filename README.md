Docs can be found at https://zperk.net/file_sync/

The MSRV is 1.71.1, due to that being the MSRV of unicode-ident, which is a dependency of proc-macro2 and syn, both of which are a dependency for thiserror-impl, which is a dependency for thiserror. Thanks to [cargo-msrv](https://crates.io/crates/cargo-msrv) for helping me find that.
