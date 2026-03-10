# Docs and help text consistency

## Goal

After implementing the other 3 features, review and update all help text, man pages, and docs for consistency and completeness.

## Scope

- `src/cli/mod.rs` — all `///` doc comments on commands and arguments
- `docs/` — design.md, data-model.md, reference/
- Man pages — auto-generated from clap, so fixing help text fixes man pages
- `CLAUDE.md` — project structure and architecture docs
- `src/cli/tmux_help.rs` — tmux reference (probably unchanged)

## Checklist

- [x] All new flags/options have clear help text
- [x] Optional `name` argument documented as "inferred from CWD if omitted"
- [x] `--local` flag on `edit` documented in README
- [x] Dynamic completions setup instructions in README
- [x] `docs/reference/` — no changes needed (topic-specific, not command-specific)
- [x] `docs/design.md` and `docs/data-model.md` still accurate
- [x] `CLAUDE.md` architecture tree updated (added 3 missing modules)
- [x] README command tables updated (`--local`, `--force`)
- [x] README architecture section expanded (CWD inference, dynamic completions, doc links)
- [x] No inconsistencies between help text and actual behavior

## Status

Complete.
