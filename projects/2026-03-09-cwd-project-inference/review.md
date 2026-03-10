# Skeptical Review: CWD-based Project Inference

**Reviewer**: Skeptical Reviewer Agent
**Date**: 2026-03-10
**Verdict**: PASS with caveats (no CRITICAL issues, some MEDIUM/LOW items)

---

## 1. `src/cli/resolve.rs` -- Algorithm Review

### Correctness: PASS

The algorithm is straightforward and correct:
1. Explicit name short-circuits (no repo access) -- good.
2. Iterates all projects, expands `~` paths, checks `Path::starts_with` -- correct.
3. Disambiguation picks deepest match (longest path wins) -- correct.
4. Ties at same depth produce an actionable error -- correct.

### Edge Cases Analyzed

| Case | Handled? | Notes |
|------|----------|-------|
| Explicit name given | Yes | Short-circuits, line 14-16 |
| CWD exact match | Yes | Tested |
| CWD is subdirectory | Yes | Tested |
| No registered projects | Yes | Tested (empty repo) |
| No match | Yes | Actionable error with `devs new` hint |
| Nested projects (deepest wins) | Yes | Tested |
| Same-path ambiguity | Yes | Tested |
| Tilde expansion | Yes | Tested with home_dir param |

### Potential Issues

**MEDIUM: No path canonicalization**
- `resolve_project_name` does not canonicalize paths. `std::env::current_dir()` returns a canonical path, but `expand_home` does not canonicalize. If a project path contains symlinks, matching will fail.
- Example: project registered at `~/src/proj` where `~/src` is a symlink to `/Volumes/code/src`. CWD would be `/Volumes/code/src/proj` but the expanded project path is `/Users/julian/src/proj`. These won't match.
- **Mitigating factor**: The integration tests in `cli_tests.rs` use `canonicalize()` on the temp dir (line 155) to handle the macOS `/private` symlink -- this shows awareness of the issue. But the core algorithm itself does not canonicalize.
- **Risk**: Low in practice, because most users register paths that are already canonical, and `devs new` uses CWD which is canonical. But symlinked home dirs or project dirs would break silently.

**LOW: Performance with many projects**
- Loads every project config to compare paths (N file reads for N projects). For typical use (< 20 projects) this is negligible. For 100+ projects, it could become noticeable.
- Not worth optimizing now but worth noting.

**LOW: `as_os_str().len()` as depth proxy**
- Line 26 uses `project_path.as_os_str().len()` as the depth metric. This works correctly because deeper paths are always longer, and the comparison is only within the set of matching paths (which share a common prefix with CWD). However, using `components().count()` would be more semantically correct.
- Not a bug, just slightly surprising.

---

## 2. `src/cli/mod.rs` -- Command Definitions

### Correctness: PASS

All 13 project-scoped commands have `name: Option<String>`:
- Init, Config, Edit, Remove, Open, Close, Save, Reset, Claude, Claudes, ClaudeDone, Note, Notes

`New` correctly retains `name: String` (required) -- a new project must have a name.

### Multi-positional commands

The three commands with additional positional args (`Claude`, `ClaudeDone`, `Note`) have their second arg also as `Option<String>`. This is necessary for the disambiguation logic in `main.rs`.

**MEDIUM: Help text shows `[NAME] [LABEL]` / `[NAME] [MESSAGE]`**
- When the user sees `devs claude-done [NAME] [LABEL]`, they might think they need to provide NAME first. The help text doesn't clarify that when only one arg is given, it's treated as the LABEL/MESSAGE, not the NAME.
- The help description says "Project name (inferred from current directory if omitted)" which is good, but the positional ordering could still confuse users.
- Suggestion: Consider adding an `after_help` or `after_long_help` note explaining the inference behavior.

---

## 3. `src/main.rs` -- Match Arms & Disambiguation

### Correctness: PASS

All 13 commands call `resolve(name)?`. The `resolve` closure correctly captures `cwd` and `home_dir`.

### Disambiguation Logic Analysis

**Claude command** (lines 148-164):
```rust
let (name, label) = match (name, label) {
    (Some(n), Some(l)) => (Some(n), Some(l)),  // two positionals given
    (Some(v), None) if resume.is_none() => (None, Some(v)),  // one positional, no --resume: it's the label
    (n, l) => (n, l),  // zero positionals, or one positional with --resume
};
```

Traced all scenarios:
| Invocation | name | label | resume | Result |
|------------|------|-------|--------|--------|
| `devs claude label` | Some("label") | None | None | name=None, label=Some("label") -- CWD inferred, starts new session |
| `devs claude proj label` | Some("proj") | Some("label") | None | name=Some("proj"), label=Some("label") -- explicit, starts new session |
| `devs claude --resume lbl` | None | None | Some("lbl") | name=None, label=None -- CWD inferred, resumes |
| `devs claude proj --resume lbl` | Some("proj") | None | Some("lbl") | name=Some("proj"), label=None -- explicit, resumes |
| `devs claude` | None | None | None | name=None, label=None -- CWD inferred, error "label is required" |

All correct.

**LOW: Silent label discard with `devs claude proj label --resume other`**
- Clap: name=Some("proj"), label=Some("label"), resume=Some("other")
- Result: name=Some("proj"), label=Some("label"), but then resume branch runs and ignores `label`
- The positional `label` is silently discarded. This is a niche edge case and arguably correct (the user explicitly asked for `--resume`), but could be confusing.

**ClaudeDone command** (lines 170-179):
```rust
let (name, label) = match (name, label) {
    (Some(n), Some(l)) => (Some(n), l),  // two args: name + label
    (Some(v), None) => (None, v),  // one arg: it's the label
    (None, _) => bail!("session label is required"),  // zero args: error
};
```

All correct. The `(None, _)` arm handles zero args and produces an actionable error.

**Note command** (lines 181-190):
Same pattern as ClaudeDone. Correct.

### Observation

**LOW: Consistency of error sources**
- `ClaudeDone` and `Note` produce their "required" errors via `bail!` in main.rs.
- `Claude` produces its "label is required" error via `anyhow::anyhow!` inside the match.
- These are inconsistent in style but functionally equivalent.

---

## 4. `tests/cli_tests.rs` -- Integration Tests

### Coverage Assessment

| Test | What it verifies |
|------|-----------------|
| `config_infers_project_from_cwd` | CWD exact match works end-to-end |
| `config_infers_project_from_subdirectory` | Subdirectory match works end-to-end |
| `config_with_explicit_name_ignores_cwd` | Explicit name bypasses CWD inference |
| `init_without_name_outside_project_shows_error` | No-match error works end-to-end |

### Test Quality: GOOD

- `setup_project_home()` correctly handles macOS `/private` symlinks via `canonicalize()`.
- Tests use `env("HOME", ...)` to control home directory, which is the correct approach.
- Tests verify both success and error paths.

### Missing Test Coverage

**MEDIUM: No integration test for multi-positional disambiguation**
- No integration test verifies that `devs claude-done <label>` (single arg) correctly treats it as the label and infers the project from CWD.
- No integration test for `devs note <message>` with CWD inference.
- The unit tests in `resolve.rs` cover the resolver itself, but the disambiguation logic in `main.rs` is only tested through the unit tests indirectly. An integration test would catch any Clap parsing surprises.

**LOW: No integration test for nested project disambiguation**
- The unit test `deepest_match_wins` covers this in the resolver, but no integration test verifies it end-to-end with real TOML files.

---

## 5. Existing Tests Impact

**PASS**: All 253 unit tests and 15 integration tests pass. No existing tests appear broken.

The existing integration tests (`from_session_conflicts_with_from`, `completions_*`, `generate_man_*`, `tmux_help_*`, `docs_mention_all_subcommands`) are unaffected by the change since they don't test project-scoped commands with required `name` args.

---

## 6. README/Docs Not Updated

**MEDIUM: README still shows `<name>` as required for all commands**
- The command tables in README.md still show `devs config <name>`, `devs open <name>`, etc., implying name is required.
- The README does not document the CWD inference feature at all.
- Users won't discover this feature from the README.

**LOW: Design docs not updated**
- `docs/design.md` and `docs/data-model.md` don't mention CWD inference. These are secondary to the README but should stay current per project conventions.

---

## Summary

| Severity | Count | Details |
|----------|-------|---------|
| CRITICAL | 0 | -- |
| HIGH | 0 | -- |
| MEDIUM | 3 | Path canonicalization gap; No integration tests for multi-positional disambiguation; README not updated |
| LOW | 4 | Performance note; os_str len depth proxy; Silent label discard; Error style inconsistency |

### Recommendation

**Ship it.** No blocking issues. The implementation is correct, well-tested at the unit level, and handles edge cases thoughtfully. The MEDIUM items are worth addressing in a follow-up:

1. **README update** -- Most impactful. Users need to know about this feature.
2. **Integration tests for disambiguation** -- Gives confidence that Clap parsing + main.rs disambiguation work together correctly for `claude-done`, `note`, and `claude` commands.
3. **Path canonicalization** -- Low risk but a correctness gap. Consider canonicalizing the expanded project path in `resolve_project_name` before the `starts_with` check.
