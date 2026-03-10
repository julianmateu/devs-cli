# Docs and help text consistency

## Goal

After implementing the other 3 features, review and update all help text, man pages, and docs for consistency and completeness.

## Scope

- `src/cli/mod.rs` — all `///` doc comments on commands and arguments
- `docs/` — design.md, data-model.md, reference/
- Man pages — auto-generated from clap, so fixing help text fixes man pages
- `CLAUDE.md` — project structure and architecture docs
- `src/cli/tmux_help.rs` — tmux reference (probably unchanged)

## Checklist (to be filled after other features land)

- [ ] All new flags/options have clear help text
- [ ] Optional `name` argument documented as "inferred from CWD if omitted"
- [ ] `--local` flag on `edit` documented
- [ ] Dynamic completions setup instructions updated
- [ ] `docs/reference/` updated if commands changed
- [ ] `docs/design.md` and `docs/data-model.md` still accurate
- [ ] `CLAUDE.md` architecture section still accurate
- [ ] No inconsistencies between help text and actual behavior

## Dependencies

- Depends on all 3 other projects completing first
