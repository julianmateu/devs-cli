# Progress

## Config split (projects/ vs local/)
- [x] Create `split_config.rs` with `PortableConfig`, `MachineLocalConfig`, `split()` (9 tests)
- [x] Refactor `TomlProjectRepository` to read/write split files (20 tests, including 11 new)
- [x] Update `delete()` to clean both locations
- [x] Backwards-compatible: v1 all-in-one files still load correctly

## Path storage as ~/...
- [x] `main.rs`: abbreviate path before passing to `new.rs` for storage
- [x] `open.rs`: expand path at start of `run()`
- [x] `claude.rs`: expand path in `start()` and `resume()`
- [x] `init.rs`: expand path before writing `.devs.toml`

## Config versioning and auto-migration
- [x] `config_version.rs`: read/write version from `config.toml` (4 tests)
- [x] `migration.rs`: `migrate_if_needed()` + `migrate_v1_to_v2()` (9 tests)
- [x] Backup original files to `backup-v1/`
- [x] Migration is idempotent
- [x] Wired into `main.rs` (runs before repo construction)
- [x] Auto-generates `.gitignore` with `local/` and `backup-v1/`

## Documentation
- [x] Updated `docs/data-model.md` to reflect v2 split format
- [x] Added git-sync documentation
- [x] Added migration documentation

## Verification
- [x] 227 unit tests pass
- [x] 12 integration tests pass
- [x] `cargo clippy -- -D warnings` clean
- [x] `cargo fmt --check` clean
