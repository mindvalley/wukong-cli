# `wukong skills`

Management of agent skills (Claude Code, generic `.agents`).

The currently implemented actions (`init`, `remove`) are local-only —
filesystem operations with no network calls. Additional actions that interact
with a remote skills registry will be added later.

## Subcommands

### `wukong skills init`

Interactively scaffold a new skill at `./.claude/skills/<name>/SKILL.md`.

- Prompts for the skill name (default: current working directory's basename).
- Asks for confirmation before writing anything to disk.
- Creates the parent directory tree (`./.claude/skills/<name>/`) if missing.
- Writes a starter `SKILL.md` (frontmatter + `When to use` + `Instructions`
  sections) modeled after the [`vercel-labs/skills`][vercel-skills] template.
- Refuses to overwrite an existing `SKILL.md`.

### `wukong skills remove`

Interactively remove installed skills.

Scans four roots for any immediate subdirectory containing a `SKILL.md`:

| Scope             | Path                         |
| ----------------- | ---------------------------- |
| `global:.claude`  | `~/.claude/skills/`          |
| `global:.agents`  | `~/.agents/skills/`          |
| `project:.claude` | `./.claude/skills/`          |
| `project:.agents` | `./.agents/skills/`          |

Each discovered skill is presented as `[<scope>] <name>` in a multi-select
prompt. After selection, the user must confirm before any deletion. Selected
skill folders are removed via `fs::remove_dir_all`. Failures are reported per
skill; successful deletions print a green `Removed` line.

## Module layout

```
cli/src/commands/skills/
├── README.md   — this file
├── mod.rs      — Skills clap Args + SkillsSubcommand dispatcher
├── init.rs     — handle_skills_init
└── remove.rs   — handle_skills_remove
```

Both handlers take a `Context` (via `get_context_without_application`) and are
annotated with `#[wukong_telemetry(command_event = "skills_init" | "skills_remove")]`
so they emit telemetry events using the existing macro — no SDK changes required.

## Design notes

- **Local-only.** These commands deliberately avoid any HTTP / GraphQL calls.
  `Context` is used solely to satisfy the telemetry macro.
- **`init` target path.** Skills are scaffolded under `./.claude/skills/<name>/`
  rather than the cwd directly (which is what the `vercel-labs/skills` CLI
  does) to keep them grouped with other Claude-specific project assets.
- **`remove` scope.** Unlike `vercel-labs/skills` — which scopes a removal to
  either project or global via a `-g` flag — we always scan all four roots and
  let the user pick across scopes in a single multi-select. Each entry is
  scope-tagged so the choice is unambiguous.
- **No overwrite.** `init` never clobbers an existing `SKILL.md`; the user is
  expected to either pick a new name or edit the file in place.

[vercel-skills]: https://github.com/vercel-labs/skills
