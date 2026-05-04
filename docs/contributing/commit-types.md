# Commit Types Guide

This project uses structured commit types to keep commit history consistent, generate a useful changelog, and determine semantic version bumps.

## Commit Message Format

```text
<emoji> <type>(optional-scope): <subject>
```

Examples:

```text
✨ feat(auth): add OAuth login
🐛 fix(api): handle missing user profile
📝 docs(readme): update setup instructions
```

## General Rules

| Rule                   | Value                 |
| ---------------------- | --------------------- |
| Maximum subject length | 100 characters        |
| Emojis                 | Required              |
| Scope                  | Optional              |
| Scope restrictions     | Any scope may be used |
| Version tag prefix     | `v`                   |
| Changelog output       | `CHANGELOG.md`        |

## Commit Types

| Type       | Emoji | When to use it                                                                                   | Why we use it                                                    | Version bump | Changelog section |
| ---------- | ----: | ------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------- | ------------ | ----------------- |
| `feat`     |     ✨ | When adding a new user-facing or developer-facing feature.                                       | Signals new functionality and appears under Features.            | Minor        | Features          |
| `fix`      |    🐛 | When fixing a bug or incorrect behavior.                                                         | Identifies defect fixes clearly.                                 | Patch        | Bug Fixes         |
| `docs`     |    📝 | When changing documentation only.                                                                | Keeps documentation changes separate from code changes.          | Patch        | Documentation     |
| `style`    |    🎨 | When changing formatting, whitespace, linting, or code style without changing behavior.          | Shows the code meaning did not change.                           | Patch        | Code Style        |
| `refactor` |    ♻️ | When restructuring code without adding a feature or fixing a bug.                                | Communicates internal code improvement.                          | Patch        | Code Refactoring  |
| `perf`     |    ⚡️ | When improving runtime, memory usage, query efficiency, or other performance characteristics.    | Highlights performance-related improvements.                     | Minor        | Performance       |
| `test`     |     ✅ | When adding, updating, or correcting tests.                                                      | Separates test coverage changes from production code changes.    | Patch        | Tests             |
| `build`    |    📦 | When changing build tooling, package management, dependencies, or generated build configuration. | Tracks changes that affect how the project is built or packaged. | Patch        | Build System      |
| `ci`       |    👷 | When changing CI configuration, pipelines, workflows, or automation scripts.                     | Separates CI/CD infrastructure changes from application changes. | Patch        | CI                |
| `chore`    |    🔧 | When making maintenance changes that do not modify source or test files.                         | Captures routine project upkeep.                                 | Patch        | Chores            |
| `revert`   |    ⏪️ | When reverting a previous commit.                                                                | Makes rollback commits obvious in history and changelogs.        | Patch        | Reverts           |

## Choosing the Right Commit Type

Use the most specific type that describes the intent of the change.

### Use `feat` ✨ when

The change introduces new behavior or capability.

Examples:

```text
✨ feat(payments): add Stripe checkout support
✨ feat(cli): add --dry-run option
```

### Use `fix` 🐛 when

The change corrects broken, incorrect, or unintended behavior.

Examples:

```text
🐛 fix(auth): prevent expired tokens from being accepted
🐛 fix(ui): align submit button on mobile
```

### Use `docs` 📝 when

Only documentation changes.

Examples:

```text
📝 docs(api): document pagination parameters
📝 docs(readme): add local development steps
```

### Use `style` 🎨 when

The code changes visually or structurally, but runtime behavior stays the same.

Examples:

```text
🎨 style: format code with prettier
🎨 style(api): reorder imports
```

### Use `refactor` ♻️ when

The code is reorganized or simplified without changing external behavior.

Examples:

```text
♻️ refactor(users): extract profile mapper
♻️ refactor(api): simplify request validation
```

### Use `perf` ⚡️ when

The change improves speed, memory use, database efficiency, caching, or throughput.

Examples:

```text
⚡️ perf(search): cache frequent query results
⚡️ perf(db): reduce user lookup queries
```

### Use `test` ✅ when

The change adds or updates tests only.

Examples:

```text
✅ test(auth): add token expiration coverage
✅ test(api): update snapshot expectations
```

### Use `build` 📦 when

The change affects dependencies, packaging, compilation, or build tools.

Examples:

```text
📦 build: update node dependencies
📦 build(docker): optimize production image
```

### Use `ci` 👷 when

The change affects continuous integration or deployment automation.

Examples:

```text
👷 ci: add release workflow
👷 ci(github): cache dependency installs
```

### Use `chore` 🔧 when

The change is general maintenance and does not affect source or test files.

Examples:

```text
🔧 chore: update issue templates
🔧 chore(repo): clean up stale config
```

### Use `revert` ⏪️ when

The change reverses a previous commit.

Examples:

```text
⏪️ revert: revert feat(auth): add OAuth login
⏪️ revert(api): undo pagination change
```

## Scopes

Scopes are optional and should describe the affected area of the codebase.

Examples:

```text
✨ feat(auth): add passwordless login
🐛 fix(database): handle connection retries
📝 docs(contributing): clarify commit format
```

Good scopes are short, lowercase, and specific.

Common scope examples:

```text
auth
api
ui
db
cli
docs
deps
release
```

## Breaking Changes

Breaking changes are supported using the footer key:

```text
BREAKING CHANGE: <description>
```

Example:

```text
✨ feat(api): remove deprecated user endpoint

BREAKING CHANGE: /v1/users/:id has been removed. Use /v2/users/:id instead.
```

Breaking change headers and footers are not required by default, but when a breaking change occurs, include the footer so users and release tooling can detect it clearly.

## Changelog Behavior

The changelog is generated as Markdown and written to:

```text
CHANGELOG.md
```

Entries are grouped by commit type and ordered as:

1. `feat`
2. `fix`
3. `docs`
4. `style`
5. `refactor`
6. `perf`
7. `test`
8. `build`
9. `ci`
10. `chore`
11. `revert`

Scopes are shown in the changelog when present. Empty sections and empty scopes are hidden.

## Quick Reference

| Change                   | Commit type   |
| ------------------------ | ------------- |
| New feature              | `✨ feat`     |
| Bug fix                  | `🐛 fix`      |
| Documentation only       | `📝 docs`     |
| Formatting only          | `🎨 style`    |
| Internal code cleanup    | `♻️ refactor` |
| Performance improvement  | `⚡️ perf`     |
| Test changes             | `✅ test`     |
| Build/dependency changes | `📦 build`    |
| CI workflow changes      | `👷 ci`       |
| Maintenance              | `🔧 chore`    |
| Revert a commit          | `⏪️ revert`   |
