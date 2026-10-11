---
id: CODEX-EI
title: Adopt Google TypeScript guidance and staged conformance checks
owner: codex
phase: 4
status: open
created: 2026-10-11
last-update: 2026-10-11 codex [GPT-6]
---

# CODEX-EI — Adopt Google TypeScript guidance and staged conformance checks

## Brief

### Goal

Establish [Google TypeScript Style Guide](https://google.github.io/styleguide/tsguide.html) as the TypeScript baseline with explicit project-specific exceptions, then migrate existing code in reviewable stages.

### Evidence and context

Root/apps/packages AGENTS.md currently require pnpm, Vitest, strict types and paired protocol changes but do not adopt Google guidance. No ESLint/Prettier/Biome configuration was found. CI's optional lint commands currently skip packages without lint scripts. Sample differences include ordinary double-quoted strings throughout TS/TSX and `Props = any` in component-library/src/types.ts. Framework config default exports and test assertions need classification, not automatic removal. This is a sample audit, not a complete compliance assessment.

Read root/scoped guidance, DU, EF, CB/BY and protocol generation conventions. Check upstream guide sections directly; optional examples must not become mandatory project rules.

### Required work and migration order

1. Write `docs/typescript-guidelines.md` with upstream reference/review date, rules, rationale and a matrix mapping each applicable rule to lint, formatter, compiler or manual review. Link from apps/packages AGENTS.md and contributing docs.
2. Audit all first-party TS/TSX and applicable Astro scripts/configs, separating generated/vendor code. Record affected files/counts and classify formatting, type safety, modules/naming and documentation gaps. Define scoped exceptions for JSX conventions, framework-required exports and wire/generated identifiers with reasons; root protocol/safety contracts take precedence.
3. Evaluate maintained formatter/lint tooling against actual guide coverage, TypeScript 7 product packages, Astro's current supported compiler and dependency footprint. Do not assume a Google-branded preset implements the full guide or install deprecated TSLint.
4. Add explicit workspace lint/format-check commands and CI enforcement. A bounded legacy baseline may permit staged cleanup only if new violations fail and the baseline shrinks. Demonstrate rejection of representative violations; no global disable or silent optional skip for adopted checks.
5. Apply formatting in isolated changes, then type safety and export/import cleanup in coherent packages. Prefer specific generics/unknown and narrowing over any; preserve legal framework exceptions. Run migration checks after each group and finish with full conformance report.

### Acceptance and validation

- [ ] Upstream-to-project rule matrix, complete gap inventory, documented exceptions and ordered migration are reviewable.
- [ ] First-party TypeScript packages run enforced lint/format checks; strict typechecking is retained, including Astro compatibility.
- [ ] Formatting is idempotent; typecheck/test/build pass, with three consecutive existing-suite passes for mechanical edits. Behavior changes have demonstrably failing pre-fix regressions.
- [ ] Designer/runtime protocol and component contracts are unchanged or updated together; generated outputs remain reproducible.

### Out of scope

Rewriting the React architecture to imitate Google's internal environment, framework upgrades owned by DU, Bun migration owned by EK and broad unrelated code cleanup.

## Codex log

## Claude review

## Verdict
