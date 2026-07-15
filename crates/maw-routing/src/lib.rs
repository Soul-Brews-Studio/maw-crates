mod core_impl;

pub use core_impl::*;

/// Routing parity fixture corpus (maw-js `routing.fixtures.json`), exported
/// so downstream parity tests need no filesystem path into this repo.
pub const ROUTING_FIXTURES_JSON: &str =
    include_str!("../tests/fixtures/routing.fixtures.json");
