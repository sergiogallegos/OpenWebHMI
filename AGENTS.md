# AGENTS.md

> Rules of engagement for **AI agents** (and humans following the same playbook) working in this repository.

This file is the contract between contributors and the OpenWebHMI knowledge surface. Read it before making non-trivial changes. It is **distinct from** `docs/contributing.md` — that file is for contributors writing code; this file is for anyone (especially AI agents) maintaining the engineering knowledge wiki and ensuring it stays load-bearing rather than ornamental.

The model here is adapted from the same author's `rust-ethernet-ip` repo, with adjustments for OpenWebHMI being a much larger surface (gateway, drivers, designer, runtime, scripting).

---

## 1. Three-layer architecture

OpenWebHMI's documentation lives in three distinct layers. **Do not blur them.**

```
┌────────────────────────────────────────────────────────────┐
│  Layer 1: Raw sources                                       │
│  - Code in this repo (gateway, drivers, designer, runtime)  │
│  - Tests + their results                                    │
│  - Vendor specifications (Rockwell EDS, OPC UA, etc.)       │
│  - Upstream crate / package source                          │
│  - Validation logs from real-hardware runs                  │
│  - GitHub issues / discussions                              │
└────────────────────────────────────────────────────────────┘
                            │
                            ▼ (synthesized)
┌────────────────────────────────────────────────────────────┐
│  Layer 2: Wiki — `wiki/`                                    │
│  - Synthesized engineering knowledge                        │
│  - Cites sources from Layer 1                               │
│  - Records decisions, open questions, conflicts             │
│  - Maintainer + agent-facing                                │
└────────────────────────────────────────────────────────────┘
                            │
                            ▼ (referenced)
┌────────────────────────────────────────────────────────────┐
│  Layer 3: User docs — `docs/` + `README.md`                 │
│  - Architecture, roadmap, feature matrix, contributing      │
│  - Contributor + user-facing                                │
│  - Stable surface; changes when behavior changes            │
└────────────────────────────────────────────────────────────┘
```

**Critical boundaries:**

- The wiki **must not** duplicate `README.md`, `docs/architecture.md`, `docs/roadmap.md`, `docs/feature-matrix.md`, or `docs/contributing.md`. If a wiki page starts re-explaining the system to users, that's a sign it should be folded back into `docs/`.
- The wiki **must not** be a raw notes dump. Entries are synthesized — they answer "what do we now know?" or "what did we decide and why?", with sources cited.
- `docs/` **must not** carry decision rationale that's still being debated. Half-decided design tensions live in the wiki until they harden.
- When user-facing docs need updating, those updates take priority over wiki entries.

## 2. Source authority hierarchy

When sources disagree, this is the precedence:

1. **Current code + tests** in this repo — highest authority for "what we built".
2. **Real-hardware validation runs** — highest authority for "what the world actually does".
3. **Vendor specifications** (Rockwell EDS, OPC UA spec, ISA-88 batch model, etc.) — authoritative for protocol/standard behavior.
4. **Upstream crate / library source** (e.g. `rust-ethernet-ip` source code) — authoritative for "what the dependency does".
5. **Vendor documentation** — authoritative until contradicted by 1–4.
6. **Issues, discussions, chat logs** — lowest authority. Useful for context, never load-bearing on their own.

When 1–4 conflict, **flag the conflict on the page**. Do not silently pick a winner. Open an issue if the conflict needs a decision.

## 3. Page format

Every wiki page follows this skimmable structure:

```markdown
---
status: active | seed | needs-review | historical
last-validated: YYYY-MM-DD
---

# Page title

## Summary
One paragraph. Headline conclusion + scope.

## Current understanding
Numbered or bulleted. Each claim cites a source.

## Evidence
Links to: code paths, test runs, vendor docs, validation logs, upstream issues.

## Open questions
What's not yet known. Each item names what would resolve it.

## Related pages
Other wiki pages; user-facing docs that depend on this.
```

Add a `Limitations`, `Upgrade workflow`, or other section as the page warrants — but the four above are the spine.

## 4. Status markers

- **`seed`** — placeholder; structure exists, content is thin. Don't trust it for decisions yet.
- **`active`** — current best understanding; trust this. Most pages should be here.
- **`needs-review`** — known to be possibly stale (recent code change in the area, time has passed since `last-validated`, etc.). Verify before acting on it.
- **`historical`** — superseded; kept for context. Should link to the page that replaced it.

When `last-validated` is older than 6 months, flip the status to `needs-review` until the next maintainer pass.

## 5. When to write or update a wiki page

**Do** create or update a wiki page when:

- Real-hardware behavior is observed (validation run, bug report from production).
- A non-obvious design decision is made (with the rejected alternatives).
- A vendor or upstream-library quirk is discovered that affects how we use it.
- Two sources disagree and one needs to be flagged as authoritative.
- A previously-open question has been resolved.

**Don't** create a wiki page for:

- Code patterns or naming conventions — those belong in code review and CONTRIBUTING.
- Architectural overviews of the whole system — that's `docs/architecture.md`.
- The list of features — that's `docs/feature-matrix.md`.
- Project status / current priorities — that's `docs/roadmap.md` and GitHub issues.
- "Notes from a debugging session" with no synthesis — write the synthesis or don't write the page.

## 6. Indexing discipline

After every wiki change:

1. **`wiki/index.md`** — if the page is new, add a one-line entry under the right category. If the page's scope changed materially, update its line.
2. **`wiki/log.md`** — append a one-line entry with date, author, page, and what changed. **Newest at bottom. Never edit prior entries.**

`index.md` is a *catalog*. `log.md` is a *journal*. They serve different purposes; both are required.

## 7. Agent responsibilities

When acting in this repo as an AI agent, you should:

### 7.1 When ingesting new sources (validation logs, vendor docs, issues, etc.)

1. Identify which wiki pages are affected.
2. Update those pages with the new information, citing the source.
3. Update `wiki/index.md` if scope changed.
4. Append to `wiki/log.md`.
5. **Preserve traceability** — every claim links to or cites its source.

### 7.2 When answering a question

1. Search the wiki first.
2. If the wiki has the answer, answer from it (and cite the wiki page).
3. If the wiki is silent, consult the underlying sources (code, tests, vendor docs).
4. **Once you've synthesized an answer, file it back into the wiki.** Don't leave durable knowledge in chat.

### 7.3 When making code changes

1. If the change invalidates a wiki claim, update the page in the **same PR**, not later.
2. If the change introduces a new, durable behavior, add it to the wiki.
3. Bump `last-validated` on any page you touched and verified.

### 7.4 When you encounter a conflict between sources

1. Don't pick a winner silently. Document both positions on the relevant page.
2. Mark the page `needs-review`.
3. Open an issue if a decision is required.

## 8. What this file is not

- **Not a code style guide.** That's in code review and language-specific tooling.
- **Not a roadmap.** That's `docs/roadmap.md`.
- **Not a how-to-contribute guide for code.** That's `docs/contributing.md`.

This file governs how the wiki stays trustworthy. Everything else has its own home.

## 9. Bootstrapping note

The wiki is currently mostly **seed**. As Phase 0 and Phase 1 work happens, pages will fill in based on what we actually learn — not on what we predict we'll need. Pages that stay `seed` for more than two phases without content should probably be deleted, not kept as aspirational stubs.

---

If you're an AI agent reading this for the first time in a session: confirm you've read it before making changes that touch the wiki, `docs/`, or any architectural decision. If you're a human: same expectation, just less ceremoniously.
