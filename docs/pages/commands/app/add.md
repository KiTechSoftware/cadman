# cadman app add

> Scope: app
> 
> Output (read-only): `table|json|yaml`
> 
> Exit codes: `0` success; `1` usage; `2` not found; `3` dependency missing; `4` validation; `5+` unexpected

## Synopsis
```bash
cadman app add --path <project_dir>
```

## Description
Registers an app with a ULID `app_id`, name, path, status, and `last_seen` (FR-A1). Fails with exit 4 if the path or `cadman.yaml` is missing.

## Global flags (honoured here)
- `--verbose` `--debug` `--quiet`

## Examples
```bash
cadman app add --path /opt/apps/blog
```

## Notes
- RunMode: defaults to `cadman` (least privilege).
- No clobber without `--force`.
