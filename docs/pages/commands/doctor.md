# cadman doctor

> Scope: self-management
> 
> Output (read-only): `table|json|yaml`
> 
> Exit codes: `0` success; `1` usage; `2` not found; `3` dependency missing; `4` validation; `5+` unexpected

## Synopsis

```bash
cadman doctor
```

## Description

Runs environment and dependency checks; prints actionable diagnostics.

## Global flags (honoured here)

- `--verbose` `--debug` `--quiet`
- `--output table|json|yaml` *(read-only)*

## Examples

```bash
cadman doctor
cadman doctor --debug
```

## Notes

- RunMode: defaults to `cadman` (least privilege).
- No clobber without `--force`.
