# Cross-Agent Collaboration Protocol

This directory is the **durable communication channel** between two LLM agents working on OpenWebHMI:

- **Claude** — design, architecture, code review.
- **Codex** — development, debugging, refactoring.

The user routes messages between us. Neither agent has access to the other's conversation; this directory is the only shared context that persists across turns.

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
| `open` | Brief written, no work started | claude (when authoring brief) |
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

1. **Frontmatter** (yaml). Fields: `id`, `title`, `owner`, `phase`, `status`, `created`, `last-update`.
2. **Brief** — written once by claude. Codex must not edit this. If the brief is wrong, codex appends a question in `## Codex log` rather than editing.
3. **Codex log** — append-only by codex. Each entry is timestamped and signed. Format below.
4. **Claude review** — append-only by claude after submission.
5. **Verdict** — final disposition by claude.

### Entry format

Within `## Codex log` and `## Claude review`, each entry starts with a header line:

```markdown
### 2026-04-26 14:30  codex
<content>

### 2026-04-26 15:10  claude — review pass 1
<content>
```

Entries are appended, never edited. If something written earlier was wrong, write a new entry that supersedes it; don't edit the old one.

### Asking questions

Either agent can ask the other a question by appending an entry that begins with `### YYYY-MM-DD HH:MM  <author> — question`. The other agent responds in their own log section (codex in `## Codex log`, claude in `## Claude review`) with a header starting `— answer to <date>`.

Questions block the task: while a question is open and unanswered, the task does not advance.

### Decisions

If a task surfaces a decision that affects more than just this task, claude records it in `wiki/architecture/` (per `AGENTS.md`'s wiki rules) and links to it from the task file. Don't bury cross-cutting decisions in a task file alone.

### Out of scope for this protocol

- Code style nits → use review entries with explicit file:line references.
- Long-running side discussions → spawn a new task file or keep them in PR comments after merge.
- Routine status updates → one line in `log.md` is enough.

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

## How to add a new task

1. Pick the next id (`CODEX-E`, `CODEX-F`, …).
2. Create `tasks/<id>-<short-name>.md` with frontmatter, Brief, empty Codex log, empty Claude review, empty Verdict.
3. Add a row to `board.md`.
4. Append an `open` event to `log.md`.

## How to consume this protocol if you are…

**…the user routing messages.** Tell each agent which file to read. Examples:
- "Codex, read `docs/agents/tasks/CODEX-A-protocol-ts.md` and start the task."
- "Claude, codex submitted CODEX-B; review it."

**…claude, reading at the start of a turn.** Read `board.md` first to see overall state. Then read the specific task file the user pointed you at. Update status + log + board in the same turn as your work.

**…codex, reading at the start of a turn.** Same — board first, then task. Append your work to the Codex log section. Don't touch Brief or Claude review. Don't edit prior entries.

## Source-of-truth precedence inside this directory

If `board.md` and a task file's frontmatter disagree, the **task file frontmatter wins** and `board.md` should be corrected. `log.md` is historical; it does not override current state.

## Relationship to other docs

- **`AGENTS.md`** at repo root — rules for the engineering wiki (`wiki/`). Different scope. Both apply simultaneously: a wiki entry might be created as a side effect of a task, in which case follow `AGENTS.md` rules for the wiki edit and this protocol for the task edit.
- **`docs/contributing.md`** — for human contributors. Humans don't need this protocol; it's purely an LLM-to-LLM channel.
- **`docs/architecture.md`** / **`docs/roadmap.md`** — the substantive design context that briefs reference. Briefs link in; they do not duplicate.
