---
name: english-code
description: Ensure every code-level artifact (identifiers, comments, log/error messages, docstrings, test names) in the current working tree is written in English. Invoke this skill whenever the user asks to review, audit, or fix the language of the code; also trigger it proactively right after finishing a coding task in this repo and before creating a commit. The developer chats in Spanish but the codebase must remain in English — this skill is the enforcement point.
---

# english-code

## When to use

- User asks: "¿está el código en inglés?", "revisa que todo esté en inglés", "audita el código", "/english-code".
- **Proactively**, after completing any Edit/Write to a `.rs` / `.toml` / source file in this repo, before handing control back to the user or before a commit.
- Before running the `english-commit` skill (code should be clean English first).

## What to check

Flag non-English content in:

1. Identifiers (variables, functions, types, modules, fields, file names).
2. Comments (`//`, `/* */`, `#`, doc comments `///`).
3. `println!`, `eprintln!`, `tracing::*`, `log::*`, `anyhow!`, `bail!`, `panic!`, `assert!` strings.
4. Test names and test messages.
5. Developer-facing documentation files (`README.md`, files under `docs/`).

## What to ignore

- User-facing UI strings shown in the GUI end-user layer (e.g. `ui.label("Configuración")` when the project is deliberately localized). When in doubt, note as ambiguous rather than flagging.
- Vendored third-party code, lockfiles, binary fixtures.
- Test fixtures that intentionally contain non-English sample input (e.g. Spanish receipt bytes to exercise CP850 decoding) — the *input* can be Spanish, but the *test name* and *assertion messages* must be English.

## How to run

1. Collect the review scope:
   - Default: current diff — `git diff HEAD` + `git diff --staged` + untracked files from `git status --porcelain`.
   - If the user gives a path or glob, scope to that instead.
2. Delegate to the `english-code-reviewer` agent with the collected scope. That agent returns a structured report.
3. Relay the report to the user in Spanish.
4. Only apply fixes if the user explicitly says "aplica las correcciones" or similar. Otherwise stop at the report.

## Output

Always respond in Spanish. Keep the list tight — quote the offending fragment, give the file:line, propose the English rewrite. End with a one-line verdict (`✅` clean / `⚠️` N issues).
