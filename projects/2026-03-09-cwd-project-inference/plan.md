# CWD-based project name inference

## Goal

When a user is inside a devs project directory, allow omitting the project name from commands.

## Current behavior

Every command that operates on a project requires an explicit `name: String` positional argument.

## Desired behavior

- `name` becomes `Option<String>` on all project-scoped commands
- If omitted, resolve by matching CWD against registered project paths
- Match if CWD equals or is a subdirectory of a project's expanded path
- Error with actionable message if no match or ambiguous match

## Implementation steps

1. **Add `resolve_project_name()` to `main.rs`** (or a small domain helper):
   - Takes `Option<&str>` (user-provided name) and `&dyn ProjectRepository`
   - If `Some(name)`, return it (current behavior)
   - If `None`, get CWD, iterate `repo.list()`, load each project, expand its path, check if CWD starts with it
   - Return the match, or error if 0 or >1 matches
2. **Change `name: String` → `name: Option<String>`** on all project-scoped commands in `src/cli/mod.rs`
3. **Update `main.rs` match arms** to call `resolve_project_name()` before dispatching
4. **Tests:**
   - Explicit name still works (pass-through)
   - CWD matching finds the right project
   - CWD in subdirectory of project path matches
   - No match → clear error
   - Multiple matches → clear error
   - Deepest match wins when one project is a subdirectory of another
5. **Update help text** to indicate name is optional

## Design considerations

- Project paths are stored abbreviated (`~/src/foo`). Must `expand_home()` before comparing.
- CWD comes from `std::env::current_dir()` — absolute path.
- Performance: `repo.list()` + loading each project to get its path. For typical usage (<50 projects) this is fine.
- Optimization: Add a `find_by_path()` method to `ProjectRepository` trait to avoid loading full configs? Defer — premature optimization.

## Files to change

- `src/cli/mod.rs` — change `name: String` to `name: Option<String>` on ~12 commands
- `src/main.rs` — add `resolve_project_name()`, update all match arms
- Possibly `src/ports/project_repository.rs` if we add a trait method
