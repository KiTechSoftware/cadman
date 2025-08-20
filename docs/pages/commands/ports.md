# cadman ports

> Scope: self-management
> 
> Output (read-only): `table|json|yaml`
> 
> Exit codes: `0` success; `1` usage; `2` not found; `3` dependency missing; `4` validation; `5+` unexpected

## Synopsis

```bash
cadman ports [--used] [--free] [--range A-B] [--domain <name>] [--service <name>] [--output table|json|yaml]
```

## Description

Shows allocator state: configured ranges, reserved per app, free ports, conflicts, pending reservations (FR-S3).

## Global flags (honoured here)

- `--verbose` `--debug` `--quiet`
- `--output table|json|yaml` *(read-only)*

## Examples

```bash
cadman ports --free
cadman ports --free --range 3000-3999 --output json
```

## Notes

- RunMode: defaults to `cadman` (least privilege).
- No clobber without `--force`.
