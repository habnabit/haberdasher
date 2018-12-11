#![deny(private_in_public, unused_extern_crates)]

#[macro_use] extern crate failure;

pub type Result<T> = std::result::Result<T, failure::Error>;

mod codec;
pub use self::codec::TelegramCodec;

pub mod session;
pub use self::session::{AppId, Session};
