# cadman daemon reload

> Scope: daemon
> 
> Output (read-only): `table|json|yaml`
> 
> Exit codes: `0` success; `1` usage; `2` not found; `3` dependency missing; `4` validation; `5+` unexpected

## Synopsis

```bash
cadman daemon reload
```

## Description

Reloads configuration without downtime. Validates Caddy config before apply (FR-7, NFR-10).

## Global flags (honoured here)

- `--verbose` `--debug` `--quiet`

## Examples

```bash
cadman daemon reload
```

## Notes

- RunMode: defaults to `cadman` (least privilege).
- No clobber without `--force`.
