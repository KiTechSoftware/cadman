# cadman daemon status

> Scope: daemon
> 
> Output (read-only): `table|json|yaml`
> 
> Exit codes: `0` success; `1` usage; `2` not found; `3` dependency missing; `4` validation; `5+` unexpected

## Synopsis

```bash
cadman daemon status [--output table|json|yaml]
```

## Description

Shows current daemon state and health.

## Global flags (honoured here)

- `--verbose` `--debug` `--quiet`
- `--output table|json|yaml` *(read-only)*

## Examples

```bash
cadman daemon status --output json
```

## Notes

- RunMode: defaults to `cadman` (least privilege).
- No clobber without `--force`.
