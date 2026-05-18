# `wukong skills`

Management of agent skills (Claude Code, generic `.agents`).

`init` and `remove` are local-only filesystem operations with no network calls.
`publish` uploads a local skill to the internal skills registry repo via the
`publishSkill` GraphQL mutation on wukong-api-proxy.

## Subcommands

### `wukong skills init`

Interactively scaffold a new skill.

Flow:

1. Prompts for **scope** — `Project` (current directory) or `Global` (home
   directory).
2. Prompts for the **skill name** (required, non-empty).
3. Asks for confirmation showing both target paths.
4. Writes the source `SKILL.md` to `<root>/.agents/skills/<name>/SKILL.md`
   using a [`vercel-labs/skills`][vercel-skills]-style template.
5. Creates a symlink at `<root>/.claude/skills/<name>/SKILL.md` pointing to
   the source file (relative target, Unix only).

`<root>` is `./` for project scope, `~/` for global scope.

Refuses if either the source file or the Claude symlink already exists.
Symlinking is Unix-only — the command exits early on non-Unix platforms.

### `wukong skills publish`

Interactively publish a local skill to the internal skills registry repo.

Flow:

1. Scans `./.agents/skills/` (project) and `~/.agents/skills/` (global) for
   any immediate subdirectory containing a `SKILL.md`. Each entry is
   presented as `[<scope>] <name>`.
2. If exactly one skill is found, prompts `Publish <label> ?`. Otherwise
   shows a single-select picker.
3. Reads `SKILL.md` and refuses if it is missing valid YAML frontmatter
   (`---\n…\n---`).
4. Prompts for a **slug** — defaults to the source folder name. Must match
   `^[a-z0-9][a-z0-9_-]{0,63}$` (lowercase alphanumeric, underscore, dash;
   up to 64 chars). The slug becomes the folder name in the registry repo
   (`skills/<slug>/SKILL.md`).
5. Prompts for an optional **commit message**. Empty falls back to the
   server default.
6. Shows source path, slug, byte count, and commit message, then asks for
   final confirmation.
7. Calls the `publishSkill` mutation. On success, prints the registry path,
   short commit SHA, and PR URL. The server queues auto-merge — the PR
   lands once required checks pass.

Server-side behavior worth knowing:

- The server overwrites or injects the `author:` line in the frontmatter
  with the authenticated user's email.
- Slug, frontmatter shape, and commit message length (≤ 8 KB) are
  re-validated server-side. Friendly messages are shown for known errors
  (`not authorized`, `invalid slug`, `invalid SKILL.md`,
  `invalid commit_message`, `skills registry publishers not configured`).
- Publishing requires the user's email to be on the
  `:skills_registry.allowed_publishers` allowlist.

Only `.agents/` roots are scanned. The `.claude/skills/<name>/SKILL.md`
entries created by `init` are symlinks back into `.agents/`, so scanning
both would double-list every skill.

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
├── publish.rs  — handle_skills_publish
└── remove.rs   — handle_skills_remove
```

All three handlers take a `Context` (via `get_context_without_application`) and
are annotated with `#[wukong_telemetry(command_event = "skills_init" |
"skills_publish" | "skills_remove")]` so they emit telemetry events using the
existing macro. `publish` additionally calls
`WKClient::publish_skill(...)`, which wraps the SDK's `PublishSkill` GraphQL
mutation and emits an `api_event = "publish_skill"` telemetry event.

## Design notes

- **Local-only `init` / `remove`.** These commands deliberately avoid any
  HTTP / GraphQL calls. `Context` is used solely to satisfy the telemetry
  macro. `publish` is the only network-touching subcommand.
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
