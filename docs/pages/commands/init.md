# cadman init

> Scope: project
> 
> Output (read-only): `table|json|yaml`
> 
> Exit codes: `0` success; `1` usage; `2` not found; `3` dependency missing; `4` validation; `5+` unexpected

## Synopsis

```bash
cadman init [--force]
```

## Description

Bootstraps a project using `cadman.yaml` as the source of truth (FR-3). Uses safe defaults; no overwrite without `--force` (NFR-8).

## Global flags (honoured here)

- `--verbose` `--debug` `--quiet`

## Examples

```bash
cadman init
cadman init --force
```

## Notes

- RunMode: defaults to `cadman` (least privilege).
- No clobber without `--force`.
