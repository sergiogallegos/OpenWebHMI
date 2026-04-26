# OpenWebHMI Engineering Wiki

This wiki is the **synthesized engineering knowledge base** for OpenWebHMI maintainers and AI agents working on the project. It is not user-facing documentation.

## What this wiki is for

- Capturing protocol, hardware, and library behavior we have *learned* (from validation runs, vendor docs, source reading) — distilled, cross-referenced, and dated.
- Recording **decisions** with their rationale and the alternatives that were considered.
- Surfacing **open questions** and unresolved conflicts between sources.
- Tracking per-release validation findings (real hardware behavior, perf baselines, regressions).

## What this wiki is *not*

- ❌ A replacement for `docs/` — that's user/contributor-facing reference.
- ❌ A replacement for `README.md` — that's the project's front door.
- ❌ A raw notes dump — entries are synthesized, sourced, and skimmable.
- ❌ A todo list — use GitHub issues for that.
- ❌ A blog — entries are durable references, not prose narratives.

## Layout

```
wiki/
├── README.md                  # this file — what the wiki is for
├── index.md                   # catalog of pages with one-line summaries
├── log.md                     # append-only chronological activity log
├── architecture/              # synthesized architecture decisions and tensions
├── drivers/                   # per-driver behavior, vendor quirks, integration notes
├── designer/                  # designer/IDE design notes and UX decisions
├── runtime/                   # web HMI runtime design notes
├── scripting/                 # scripting host design and limitations
├── investigations/            # open research, comparisons, prototypes
└── releases/                  # per-release validation synthesis
```

## How to use it

When working on the project (human or AI), the workflow is:

1. **Before answering a question or making a change**: search this wiki first. If the answer is here, use it. If it's stale or wrong, fix it as part of your work.
2. **After learning something durable** (validating a behavior, choosing between two designs, hitting a vendor quirk): write or update the relevant page, with sources cited.
3. **Index it**: every new page gets one line in `index.md`. Every meaningful update gets one line in `log.md`.

For agents specifically: see [`AGENTS.md`](../AGENTS.md) for the source-authority rules and page-format expectations.

## Page status markers

Every page declares its status near the top:

- `seed` — placeholder; structure exists, content is thin.
- `active` — current best understanding; trust this.
- `needs-review` — known to be possibly stale; verify before acting.
- `historical` — superseded; kept for context. Linked from the page that replaced it.

## Source authority

When sources conflict, this is the precedence order:

1. **Current code + tests in this repo** — highest authority for "what we built".
2. **Real-hardware validation runs** — highest authority for "what the world does".
3. **Vendor specifications** (Rockwell EDS, OPC UA spec, etc.) — authoritative for protocol behavior.
4. **Crate / library source** (e.g. `rust-ethernet-ip` source on GitHub) — authoritative for "what the library does".
5. **Vendor documentation** — authoritative until contradicted by 1–4.
6. **Issues, discussions, chat logs** — lowest authority. Useful for context, never load-bearing.

When 1–4 conflict, **flag the conflict on the page**. Do not silently pick a winner.

## Page format

Pages follow a skimmable structure:

```markdown
---
status: active | seed | needs-review | historical
last-validated: YYYY-MM-DD
---

# Page title

## Summary
One paragraph. What this page is about and the headline conclusion.

## Current understanding
Numbered or bulleted facts. Each claim cites a source.

## Evidence
Links to: code paths, test runs, vendor docs, validation logs, GitHub issues.

## Open questions
What's not yet known. Each question has either a "we'll learn this when…" or "blocked on…".

## Related pages
Other wiki pages; user-facing docs that depend on this.
```
