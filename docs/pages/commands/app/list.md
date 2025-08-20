# cadman app list (ls)

> Scope: app
> 
> Output (read-only): `table|json|yaml`
> 
> Exit codes: `0` success; `1` usage; `2` not found; `3` dependency missing; `4` validation; `5+` unexpected

## Synopsis
```bash
cadman app list [--status <filter>] [--output table|json|yaml]
```

## Description
Lists apps with optional filters and output formats (FR-A3).

## Global flags (honoured here)
- `--verbose` `--debug` `--quiet`
- `--output table|json|yaml` *(read-only)*

## Examples
```bash
cadman app ls
cadman app list --status not_found --output json
```

## Notes
- RunMode: defaults to `cadman` (least privilege).
- No clobber without `--force`.
