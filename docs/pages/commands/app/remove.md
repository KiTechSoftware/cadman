# cadman app remove

> Scope: app
> 
> Output (read-only): `table|json|yaml`
> 
> Exit codes: `0` success; `1` usage; `2` not found; `3` dependency missing; `4` validation; `5+` unexpected

## Synopsis
```bash
cadman app remove (--id <ULID> | --name <string> | --path <dir>) [--purge]
```

## Description
Marks the app as `deleted`, or permanently removes its registry entry and port reservations with `--purge` (FR-A2).

## Global flags (honoured here)
- `--verbose` `--debug` `--quiet`

## Examples
```bash
cadman app remove --name blog-site
cadman app remove --id 01J3... --purge
```

## Notes
- RunMode: defaults to `cadman` (least privilege).
- No clobber without `--force`.
