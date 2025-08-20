# cadman app ports

> Scope: app
> 
> Output (read-only): `table|json|yaml`
> 
> Exit codes: `0` success; `1` usage; `2` not found; `3` dependency missing; `4` validation; `5+` unexpected

## Synopsis
```bash
cadman app ports (--id <ULID> | --name <string> | --path <dir>) [--output table|json|yaml]
```

## Description
Shows reserved/requested ports and observed runtime bindings; highlights mismatches (FR-A6).

## Global flags (honoured here)
- `--verbose` `--debug` `--quiet`
- `--output table|json|yaml` *(read-only)*

## Examples
```bash
cadman app ports --name blog-site --output yaml
```

## Notes
- RunMode: defaults to `cadman` (least privilege).
- No clobber without `--force`.
