# Projects Index

All past projects are preserved in git history.

## How to Recover

```bash
# Find project commits
git log --all --oneline -- "projects/PROJECT-NAME"

# View plan from history
git show COMMIT:projects/PROJECT-NAME/plan.md

# Restore entire project
git checkout COMMIT -- projects/PROJECT-NAME
```

## Active Projects

| Project | Summary |
|---------|---------|
| 2026-03-09-docs-consistency | Align docs/ with current codebase state |
| 2026-03-09-ux-improvements | UX improvements summary (CWD inference, completions, edit --local, docs) |

## Archived Projects

Recover with `git show <commit>:projects/<name>/plan.md`

| Project | Summary | Archived at |
|---------|---------|-------------|
| `2026-03-06-v1-1-polish` | Man pages, tmux help, shareable `.devs.toml` + `devs init` | `62ce85b` |
| `2026-03-06-multi-machine-config` | Split config into portable/local, tilde paths, auto-migration v1→v2, git-sync support | `c883108` |
| `2026-03-09-edit-local-flag` | `devs edit --local` flag to edit machine-local config | `2f6ba4f` |
| `2026-03-09-cwd-project-inference` | CWD-based project name inference for all project-scoped commands | `f6cbdf7` |
| `2026-03-09-dynamic-completions` | Dynamic shell completions for project names via CompleteEnv | `ac84ca6` |
