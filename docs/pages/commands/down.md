# cadman down

> Scope: project
> 
> Output (read-only): `table|json|yaml`
> 
> Exit codes: `0` success; `1` usage; `2` not found; `3` dependency missing; `4` validation; `5+` unexpected

## Synopsis

```bash
cadman down
```

## Description

Stops project services defined in `cadman.yaml` (FR-3).

## Global flags (honoured here)

- `--verbose` `--debug` `--quiet`

## Examples

```bash
cadman down
```

## Notes

- RunMode: defaults to `cadman` (least privilege).
- No clobber without `--force`.
