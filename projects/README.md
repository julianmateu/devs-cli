# Projects Directory

This directory contains project-specific documentation and implementation guides.

## Project Structure

Each project contains:
- `plan.md` or `summary.md` — Mission brief and entry point
- Supporting documentation specific to the project
- Progress tracking files (when applicable)

## Project Lifecycle

1. **Active**: Project in development, documents in `projects/YYYY-MM-DD-project-name/`
2. **Completed**: Implementation done, learnings documented
3. **Indexed**: Add entry to INDEX.md with final commit hash for reference
4. **Archived**: **DELETE the project folder** — git history IS the archive
5. **Integrated**: Key insights added to canonical documentation in `docs/`

### Recovering Archived Projects

Projects are fully recoverable from git history:

```bash
# Find when a project was deleted
git log --all --full-history -- "projects/2026-MM-DD-project-name/"

# View files at a specific commit
git show <commit>:projects/2026-MM-DD-project-name/plan.md

# Restore entire project from history
git checkout <commit> -- projects/2026-MM-DD-project-name/
```

### IMPORTANT: No Archive Folders!

- There is **NO** `projects/archived/` folder
- **Git history IS the archive**
- Completed projects should be **DELETED**, not moved

## Private Projects

The `projects/.private/` subfolder is gitignored. Use it for project docs that contain sensitive information or personal notes that should not be in the public repo.

## Creating New Projects

When starting a new project:
1. Create folder: `projects/YYYY-MM-DD-project-name/`
2. Add a plan or summary with:
   - Mission statement
   - Current state assessment
   - Execution strategy
   - Success criteria
3. Include all project-specific documentation
4. Ensure self-contained context for fresh starts

## Design Principle

Project documentation should be **self-contained** — a fresh developer or AI agent should be able to pick up the plan and execute the project without prior context.
