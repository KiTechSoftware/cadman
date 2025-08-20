# cadman info

> Scope: self-management
> 
> Output (read-only): `table|json|yaml`
> 
> Exit codes: `0` success; `1` usage; `2` not found; `3` dependency missing; `4` validation; `5+` unexpected

## Synopsis

```bash
cadman info
```

## Description

Prints effective global config after precedence (FR-S1).

## Global flags (honoured here)

- `--verbose` `--debug` `--quiet`
- `--output table|json|yaml` *(read-only)*

## Examples

```bash
cadman info --output json
```

## Notes

- RunMode: defaults to `cadman` (least privilege).
- No clobber without `--force`.
