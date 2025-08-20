# cadman purge

> Scope: project
> 
> Output (read-only): `table|json|yaml`
> 
> Exit codes: `0` success; `1` usage; `2` not found; `3` dependency missing; `4` validation; `5+` unexpected

## Synopsis
```bash
cadman purge
```

## Description
Stops and removes project artifacts managed by Cadman (FR-3). Respects No‑Clobber unless user opts in.

## Global flags (honoured here)
- `--verbose` `--debug` `--quiet`

## Examples
```bash
cadman purge
```

## Notes
- RunMode: defaults to `cadman` (least privilege).
- No clobber without `--force`.
