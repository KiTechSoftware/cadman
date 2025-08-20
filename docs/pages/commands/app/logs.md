# cadman app logs

> Scope: app
> 
> Output (read-only): `table|json|yaml`
> 
> Exit codes: `0` success; `1` usage; `2` not found; `3` dependency missing; `4` validation; `5+` unexpected

## Synopsis
```bash
cadman app logs (--id <ULID> | --name <string> | --path <dir>) [--follow] [--since <ts>] [--tail <n>] [--level <lvl>]
```

## Description
Streams or snapshots app logs with common filters (FR-A7).

## Global flags (honoured here)
- `--verbose` `--debug` `--quiet`

## Examples
```bash
cadman app logs --name blog-site --follow
cadman app logs --id 01J3... --tail 200
```

## Notes
- RunMode: defaults to `cadman` (least privilege).
- No clobber without `--force`.
