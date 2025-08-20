# cadman daemon stop

> Scope: daemon
> 
> Output (read-only): `table|json|yaml`
> 
> Exit codes: `0` success; `1` usage; `2` not found; `3` dependency missing; `4` validation; `5+` unexpected

## Synopsis

```bash
cadman daemon stop
```

## Description

Stops the Cadman service (FR-2).

## Global flags (honoured here)

- `--verbose` `--debug` `--quiet`

## Examples

```bash
cadman daemon stop
```

## Notes

- RunMode: defaults to `cadman` (least privilege).
- No clobber without `--force`.
