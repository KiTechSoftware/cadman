# cadman daemon start

> Scope: daemon
> 
> Output (read-only): `table|json|yaml`
> 
> Exit codes: `0` success; `1` usage; `2` not found; `3` dependency missing; `4` validation; `5+` unexpected

## Synopsis

```bash
cadman daemon start
```

## Description

Starts the Cadman service (FR-2). Integrates with systemd/launchd.

## Global flags (honoured here)

- `--verbose` `--debug` `--quiet`

## Examples

```bash
cadman daemon start
```

## Notes

- RunMode: defaults to `cadman` (least privilege).
- No clobber without `--force`.
