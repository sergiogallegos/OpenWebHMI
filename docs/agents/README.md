# Cross-Agent Collaboration Protocol

This directory is the **durable communication channel** between two LLM agents working on OpenWebHMI:

- **Claude** — design, architecture, code review.
- **Codex** — development, debugging, refactoring.

The repository maintainer routes messages between us. Neither agent has access to the other's conversation; this directory is the only shared context that persists across turns.

> If you are an LLM reading this for the first time in a session, identify yourself (`claude` or `codex`) before making any changes. Read this whole file before writing to any other file in this directory.

## Why this exists

A single conversation context belongs to one agent. When two agents collaborate on the same codebase, they need a shared, append-mostly artifact to:

- Hand off task briefs without re-explaining context every turn.
- Ask each other clarifying questions that survive across sessions.
- Record decisions and review verdicts that bind both agents.
- Let either agent reconstruct project state by reading the directory cold.

## File layout

```
docs/agents/
├── README.md                       # this file — the protocol
├── board.md                        # status of every task at a glance
├── log.md                          # append-only chronological transcript
├── review-template.md              # nine-part Claude-review contract
├── notes/                          # surface-specific durable lore
│   ├── binding-write-asymmetry.md
│   ├── mqtt-tls-in-ci.md
│   ├── python-tag-write-routing.md
│   └── toolchain-drift.md
└── tasks/
    ├── CODEX-A-protocol-ts.md
    ├── CODEX-B-gateway.md
    ├── CODEX-C-runtime-web.md
    └── CODEX-D-ci.md               # one file per task, full lifecycle
```

- **`board.md`** — single table summarizing every task: id, title, owner, phase, status, last update. Kanban-style snapshot.
- **`log.md`** — append-only one-liners. Format: `YYYY-MM-DD HH:MM <author> <task-id> <event>`. Newest at bottom. Never edit prior entries.
- **`tasks/<id>.md`** — full lifecycle for one task: the brief, Codex's working notes, Claude's review, the verdict. Each task gets one file. Don't split a task across files.

## Task lifecycle

Status flow:

```
open ──▶ in-progress ──▶ submitted ──▶ under-review ──┬──▶ merged
                ▲                                     │
                └─────────── rejected ◀───────────────┘
```

| Status | Meaning | Set by |
|---|---|---|
| `open` | Brief written, no work started | task author (normally Claude; Codex when directed by the maintainer) |
| `in-progress` | Codex acknowledged and started | codex (when starting work) |
| `submitted` | Codex finished, awaiting review | codex (with commit ref or diff) |
| `under-review` | Claude has begun reviewing | claude (when starting review) |
| `merged` | Approved and integrated | claude (with merge commit ref) |
| `rejected` | Changes requested; back to `in-progress` after Codex addresses | claude (with punch list) |

Status changes are reflected in **three** places:
1. The task file's frontmatter `status:` field.
2. The corresponding row in `board.md`.
3. A new line in `log.md`.

All three are part of the same edit.

## Authoring conventions

### Sections in a task file

Every task file has these sections, in this order:

1. **Frontmatter** (yaml). Fields: `id`, `title`, `owner`, `phase`, `status`, `created`, `last-update`. `phase` is a non-negative roadmap phase; post-1.0 work may use Phase 5 or later.
2. **Brief** — written once by the task author (normally Claude; Codex may author when directed by the maintainer). After work starts, Codex must not edit it. If the brief is wrong, Codex appends a question in `## Codex log` rather than editing.
3. **Codex log** — append-only by codex. Each entry is timestamped and signed. Format below.
4. **Claude review** — append-only by claude after submission.
5. **Verdict** — final disposition by claude.

### Entry format

Within `## Codex log` and `## Claude review`, each entry starts with a header line that carries the agent **and** the underlying model in square brackets:

```markdown
### 2026-04-26 14:30  codex [gpt-5]
<content>

### 2026-04-26 15:10  claude [Opus 4.7] — review pass 1
<content>
```

Entries are appended, never edited. If something written earlier was wrong, write a new entry that supersedes it; don't edit the old one.

### Agent + model tags

Every entry written into this directory carries the underlying model so the maintainer can audit model-vs-quality over time:

- **Log lines** in `log.md` — `YYYY-MM-DD HH:MM <author> [<model>] <task-id> <event>`. Example: `2026-05-25 14:30  claude [Opus 4.7]  CODEX-AM  merged at f02eef5`.
- **Task frontmatter `last-update`** — `YYYY-MM-DD <author> [<model>]`. Example: `last-update: 2026-05-25 claude [Opus 4.7]`.
- **Entry headers** in `## Codex log` and `## Claude review` — `### YYYY-MM-DD HH:MM <author> [<model>]` as shown above.

The `[<model>]` value is the underlying model name as the maintainer would say it — `Opus 4.7`, `Sonnet 4.6`, `Haiku 4.5`, `gpt-5`, `gpt-5.5`. Match what the maintainer uses in chat; don't invent variants.

This convention is **load-bearing for new entries dated 2026-05-17 or later** and is checked by `scripts/validate-agent-files`. Older entries without model tags are grandfathered — don't backfill them.

Agent + model tags belong **only** in `docs/agents/`. They must not appear in commit messages, PR descriptions, public docs, or anywhere in the git history surfaced to NuGet / crates.io / npm consumers — see the Voice section below.

### Asking questions

Either agent can ask the other a question by appending an entry that begins with `### YYYY-MM-DD HH:MM  <author> — question`. The other agent responds in their own log section (codex in `## Codex log`, claude in `## Claude review`) with a header starting `— answer to <date>`.

Open questions block the task — while a question is open and unanswered, the task does not advance.

**Ambiguity threshold — when to stop vs proceed:**

- **Stop and ask** when the ambiguity affects acceptance criteria, contradicts a brief assumption, or requires source-of-truth context the brief doesn't provide. Examples: brief pins a crate version that doesn't exist on crates.io; brief contradicts an architectural decision in `docs/architecture.md`; acceptance test description is internally inconsistent; brief specifies an API surface that conflicts with an upstream library's actual API.
- **Document and proceed** when the ambiguity is a normal implementation choice with no contract impact. Examples: variable naming, internal helper structure, log message wording, choice between two equivalent stdlib calls. Add a one-line entry to `## Codex log` recording the assumption ("Assumed X because Y; revisit if review disagrees.") so review can cheaply override.

The cost of stalling on small details is higher than the cost of a v1.1 polish item.

### Decisions

If a task surfaces a decision that affects more than just this task, claude records it in `wiki/architecture/` (per `wiki/AGENTS.md`'s wiki rules) and links to it from the task file. Don't bury cross-cutting decisions in a task file alone.

### Out of scope for this protocol

- Code style nits → use review entries with explicit file:line references.
- Long-running side discussions → spawn a new task file or keep them in PR comments after merge.
- Routine status updates → one line in `log.md` is enough.

### Voice

Use neutral framing in everything written into this directory and into project docs (`CLAUDE.md`, `docs/roadmap.md`, wiki pages, task files):

- **No first-person.** Write "Codex implemented X" / "Claude-authored brief" / "the original brief" / "brief error owned by Claude". Not "I added X" / "my brief" / "I told Codex".
- **No maintainer profiling.** Write "the maintainer requested" / "per maintainer direction". Not "the user wants X" / "the user catches this" / direct quotes of maintainer chat.
- **End-user references are fine when domain-relevant.** "user-scripting layer", "the user types the device path", "case as written by the user" are correct when they refer to actual end-users of the product (operators, integrators, Python script authors). Those are domain terms, not maintainer references.
- **Paraphrase, don't quote.** If a maintainer message defines a project convention, restate it neutrally as the convention. Don't embed the original message verbatim.

This directory and its referenced docs are public artifacts. Personal phrasing leaks behavioral signals (work patterns, incidents, preferences) that belong in private agent memory, not in project history. Both agents should self-edit before committing; reviewers flag voice drift in the same pass as technical findings.

## Who edits what

| File | Claude edits | Codex edits |
|---|:-:|:-:|
| `README.md` (this file) | rarely (protocol changes) | only if asked by claude |
| `board.md` | yes, on status changes | yes, on status changes |
| `log.md` | append on events | append on events |
| `tasks/<id>.md` Brief | yes, when authoring | never |
| `tasks/<id>.md` Codex log | never | append-only |
| `tasks/<id>.md` Claude review | append-only | never |
| `tasks/<id>.md` Verdict | yes (sole author) | never |
| `tasks/<id>.md` frontmatter | yes (when status flips that claude owns) | yes (when status flips that codex owns) |

## Commit and push expectations

Both agents may stage and commit edits to task files, `board.md`, and `log.md` as part of normal task work. The lifecycle three-place update (frontmatter + board + log) should commit together.

Use `scripts/agent-commit "<message>" <file> [file...]` when practical. It unstages the full index first, stages only the named files, rejects wildcard staging, blocks obvious secret filenames, and blocks amend commits unless `--amend-anyway` is passed with explicit maintainer direction. Direct `git commit` remains valid when specific files are staged manually — `agent-commit` is the safer default for agent-driven commits, not a hard requirement.

## Local agent validation

Run `scripts/install-hooks` once per clone to opt into the repo-local pre-commit hook. The hook runs `scripts/validate-agent-files` only when staged files under `docs/agents/` change — it's a no-op for any commit that doesn't touch this directory.

CI runs the same validator (`validate-agent-files` job in `.github/workflows/ci.yml`) for every push and pull request. The validator catches:

- Missing or malformed frontmatter on task files (`id`, `title`, `owner`, `phase`, `status`, `created`, `last-update`).
- Status drift between a task file's frontmatter and the `board.md` row.
- Tasks listed under a Phase table whose status is `merged` or `rejected` (should be in Done).
- Tasks listed under Done whose status isn't `merged`.
- Done rows missing or with a malformed merge commit hash.
- Log lines that don't parse, and entries dated 2026-05-17 or later that lack the `[<model>]` tag.

Fix violations rather than disabling the check. If the validator catches drift in pre-existing files (e.g. a task file whose `status:` was never updated after merge), fix the frontmatter — the validator is the source of truth.

**Pushing to the remote is not automatic:**

- Push only when the maintainer explicitly asks ("commit and push", "ship it"), or when an unambiguous task convention requires it (e.g. backfilling a merge ref in a follow-up commit).
- Push only when the local environment permits it. Some sessions block writes to system-owned repos (Git safe-directory checks); some block network egress entirely. If push is blocked, surface the blocker — don't retry silently or work around it.
- A successful local commit is not a successful push. Always confirm the push step ran before claiming a task moved to `merged` or `submitted`.

This prevents the case where one agent's session pushes to the remote while the other agent's session has unpushed local commits, leaving the two views diverged.

## How to add a new task

1. Pick the next id (`CODEX-E`, `CODEX-F`, …).
2. Create `tasks/<id>-<short-name>.md` with frontmatter, Brief, empty Codex log, empty Claude review, empty Verdict.
3. Add a row to `board.md`.
4. Append an `open` event to `log.md`.

## How to consume this protocol if you are…

**…the maintainer routing messages.** Tell each agent which file to read. Examples:
- "Codex, read `docs/agents/tasks/CODEX-A-protocol-ts.md` and start the task."
- "Claude, codex submitted CODEX-B; review it."

**…claude, reading at the start of a turn.** Read `board.md` first to see overall state. Then read the specific task file the maintainer pointed you at. Update status + log + board in the same turn as your work.

**…codex, reading at the start of a turn.** Same — board first, then task. Append your work to the Codex log section. Don't touch Brief or Claude review. Don't edit prior entries.

## Source-of-truth precedence inside this directory

If `board.md` and a task file's frontmatter disagree, the **task file frontmatter wins** and `board.md` should be corrected. `log.md` is historical; it does not override current state.

## Relationship to other docs

- **`AGENTS.md`** at repo root — codebase-wide code, test, and dependency rules for any agent. Auto-loaded by Codex and Claude Code.
- **`wiki/AGENTS.md`** — engineering-wiki governance (layered docs, source authority, page format). Auto-loaded under `wiki/`. A wiki entry created as a side effect of a task follows `wiki/AGENTS.md` for the wiki edit and this protocol for the task edit.
- **`VISION.md`** at repo root — what OpenWebHMI is, non-goals, and what we won't merge.
- **`docs/contributing.md`** — for human contributors. Humans don't need this protocol; it's purely an LLM-to-LLM channel.
- **`docs/architecture.md`** / **`docs/roadmap.md`** — the substantive design context that briefs reference. Briefs link in; they do not duplicate.
