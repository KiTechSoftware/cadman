# cadman app status

> Scope: app
> 
> Output (read-only): `table|json|yaml`
> 
> Exit codes: `0` success; `1` usage; `2` not found; `3` dependency missing; `4` validation; `5+` unexpected

## Synopsis
```bash
cadman app status (--id <ULID> | --name <string> | --path <dir>) [--output table|json|yaml]
```

## Description
Shows runtime: containers, health, service manager state, Caddy route health (FR-A5).

## Global flags (honoured here)
- `--verbose` `--debug` `--quiet`
- `--output table|json|yaml` *(read-only)*

## Examples
```bash
cadman app status --id 01J3... --output json
```

## Notes
- RunMode: defaults to `cadman` (least privilege).
- No clobber without `--force`.
