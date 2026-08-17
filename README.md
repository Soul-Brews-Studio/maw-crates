# maw-crates

Leaf crates extracted from [maw-rs](https://github.com/Soul-Brews-Studio/maw-rs)
(repo split, 2026-07). Consumed back by `maw-cli` as Cargo **git dependencies**
pinned to a `rev`:

```toml
maw-fuzzy = { git = "https://github.com/Soul-Brews-Studio/maw-crates", rev = "<sha>" }
```

All crates here are self-contained leaves with no dependencies on other maw
crates; all forbid unsafe code and remain clippy-pedantic clean. Most expose
deterministic, side-effect-free core logic. Filesystem-facing leaves such as
`maw-plugin-scaffold` and `maw-xdg` constrain I/O to their documented crate
boundaries and test it against isolated paths. Behavior is locked by
crate-local tests, including maw-js JSON fixture corpora where applicable.
Several crates re-export their fixture corpora as `pub const *_FIXTURES_JSON`
so downstream parity tests don't need filesystem paths into this repo.
`maw-activity` and `maw-matcher` expose downstream corpora only through the
nondefault `fixtures` feature, keeping them out of normal production builds.

## Crates

| Crate | Purpose |
|---|---|
| `maw-activity` | Pure terminal-snapshot activity classification |
| `maw-auto-wake` | Auto-wake scheduling helpers |
| `maw-bind` | Key/target binding logic |
| `maw-bring` | Bring-pane planning |
| `maw-feed` | Feed formatting |
| `maw-fuzzy` | Fuzzy matching |
| `maw-hub` | Hub protocol types |
| `maw-identity` | Canonical session/node identity |
| `maw-matcher` | Portable target-name matching and typed resolution |
| `maw-plugin-scaffold` | Plugin scaffolding templates |
| `maw-policy` | Plugin tier / default-active policy |
| `maw-routing` | Target routing resolution |
| `maw-schedule` | Schedule configuration and launchd rendering |
| `maw-split` | Split-pane policy |
| `maw-xdg` | XDG/legacy path resolution and layered config loading |

## Gate

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

`scripts/gen-routing-corpus.ts` regenerates the maw-routing differential
fixture corpus from a maw-js checkout
(`MAW_JS_REPO=/path/to/maw-js bun scripts/gen-routing-corpus.ts`).

## License

BUSL-1.1 — see [LICENSE](LICENSE). Not published to crates.io.
