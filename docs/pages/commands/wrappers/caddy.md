# cadman caddy

> Scope: wrapper
> 
> Output (read-only): `table|json|yaml`
> 
> Exit codes: `0` success; `1` usage; `2` not found; `3` dependency missing; `4` validation; `5+` unexpected

## Synopsis

```bash
cadman caddy [-- ...passthrough flags...]
```

## Description

Generates config and interacts with Caddy via API/CLI. JSON is default; Caddyfile on request. Validates before reload (FR-7, NFR-10).

## Global flags (honoured here)

- `--verbose` `--debug` `--quiet`

## Examples

```bash
cadman caddy version
cadman caddy reload
```

## Notes

- RunMode: defaults to `cadman` (least privilege).
- No clobber without `--force`.
