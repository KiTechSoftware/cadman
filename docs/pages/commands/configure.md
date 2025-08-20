# cadman configure

> Scope: self-management
> 
> Output (read-only): `table|json|yaml`
> 
> Exit codes: `0` success; `1` usage; `2` not found; `3` dependency missing; `4` validation; `5+` unexpected

## Synopsis

```bash
cadman configure
```

## Description

Interactive or non-interactive configuration of global settings (`config.toml`).

## Global flags (honoured here)

- `--verbose` `--debug` `--quiet`

## Examples

```bash
cadman configure
```

## Notes

- RunMode: defaults to `cadman` (least privilege).
- No clobber without `--force`.
