#![feature(await_macro, async_await, futures_api)]
#![deny(private_in_public, unused_extern_crates)]

#[macro_use] extern crate failure;
#[macro_use] extern crate futures;
#[macro_use] extern crate slog;
#[macro_use] extern crate tokio;

pub mod client;
pub mod kex;

pub type Result<T> = ::std::result::Result<T, ::failure::Error>;
