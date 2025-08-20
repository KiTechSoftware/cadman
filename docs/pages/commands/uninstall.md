# cadman uninstall

> Scope: self-management
> 
> Output (read-only): `table|json|yaml`
> 
> Exit codes: `0` success; `1` usage; `2` not found; `3` dependency missing; `4` validation; `5+` unexpected

## Synopsis
```bash
cadman uninstall
```

## Description
Uninstalls Cadman. May leave user data/config unless a separate purge command is provided.

## Global flags (honoured here)
- `--verbose` `--debug` `--quiet`

## Examples
```bash
cadman uninstall
```

## Notes
- RunMode: defaults to `cadman` (least privilege).
- No clobber without `--force`.
