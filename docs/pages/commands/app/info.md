# cadman app info

> Scope: app
> 
> Output (read-only): `table|json|yaml`
> 
> Exit codes: `0` success; `1` usage; `2` not found; `3` dependency missing; `4` validation; `5+` unexpected

## Synopsis
```bash
cadman app info (--id <ULID> | --name <string> | --path <dir>) [--output table|json|yaml]
```

## Description
Shows static information: registry entry and resolved config after precedence; derived service units; effective Caddy snippet with secrets redacted (FR-A4).

## Global flags (honoured here)
- `--verbose` `--debug` `--quiet`
- `--output table|json|yaml` *(read-only)*

## Examples
```bash
cadman app info --name blog-site --output json
```

## Notes
- RunMode: defaults to `cadman` (least privilege).
- No clobber without `--force`.
