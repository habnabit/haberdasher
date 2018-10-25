extern crate capnp;

mod haberdasher_capnp {
    include!(concat!(env!("OUT_DIR"), "/src/haberdasher_capnp.rs"));
}

pub use haberdasher_capnp::*;
