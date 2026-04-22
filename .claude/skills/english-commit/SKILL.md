---
name: english-commit
description: Ensure git commit messages in this repo are written in clear, idiomatic English (imperative subject, English body) even though the developer chats in Spanish. Invoke whenever the user asks to create, amend, review, or check a commit message; also run proactively right before calling `git commit` to validate the drafted message. Applies both to brand-new drafts and to the current HEAD commit after-the-fact.
---

# english-commit

## When to use

- User asks: "haz commit", "crea el commit", "revisa el commit", "/english-commit", "¿está bien el mensaje?".
- **Proactively**, immediately before running `git commit ...`: validate the drafted subject/body first.
- Right after a commit lands, if the user wants to double-check.

## Rules enforced

- **Subject**: English, imperative mood (`Add`, `Fix`, `Refactor`, `Document`; never `Added`, `Agrega`, `Arreglado`), ≤ 72 chars, no trailing period, no emoji prefix unless repo history already uses them.
- **Body** (if present): English, wraps around 72 columns, focuses on *why* rather than *what*, references issues/PRs as needed.
- **Trailers** (`Co-Authored-By`, `Signed-off-by`, etc.): leave untouched, they stay as-is.
- Proper nouns, API names, quoted error strings, file paths are fine even if they contain non-English tokens.

## How to run

1. Obtain the message to review:
   - If the user provided a draft → use it directly.
   - If a commit is being composed, collect the draft that's about to be passed to `git commit -m` / `-F`.
   - If the user says "el último commit" / "el commit actual" → `git log -1 --pretty=%B`.
   - If they give a range (e.g. `main..HEAD`) → inspect each commit.
2. Delegate to the `english-commit-reviewer` agent for structured feedback.
3. Relay the verdict to the user in Spanish. If issues are found, show the suggested English rewrite.
4. Only amend or recreate the commit if the user explicitly confirms. Never run `git commit --amend` or force-push on your own.

## Output

Respond in Spanish with:
- `Commit <sha|"borrador">`
- Veredicto (`✅` / `⚠️` / `❌`)
- Problemas concretos citados literalmente
- Reescritura sugerida en inglés si aplica

Keep it short. One commit = one short block.
