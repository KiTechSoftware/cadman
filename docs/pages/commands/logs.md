# cadman logs

> Scope: self-management
> 
> Output (read-only): `table|json|yaml`
> 
> Exit codes: `0` success; `1` usage; `2` not found; `3` dependency missing; `4` validation; `5+` unexpected

## Synopsis

```bash
cadman logs [--follow] [--since <ts>] [--tail <n>] [--level <lvl>]
```

## Description

Shows Cadman’s own logs (FR-S2).

## Global flags (honoured here)

- `--verbose` `--debug` `--quiet`

## Examples

```bash
cadman logs --tail 200
cadman logs --follow --level info
```

## Notes

- RunMode: defaults to `cadman` (least privilege).
- No clobber without `--force`.
