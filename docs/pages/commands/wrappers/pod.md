# cadman pod

> Scope: wrapper
> 
> Output (read-only): `table|json|yaml`
> 
> Exit codes: `0` success; `1` usage; `2` not found; `3` dependency missing; `4` validation; `5+` unexpected

## Synopsis

```bash
cadman pod [-- ...passthrough flags...]
```

## Description

Runs Podman in the correct RunMode with Cadman’s logging and context handling (FR-4).

## Global flags (honoured here)

- `--verbose` `--debug` `--quiet`

## Examples

```bash
cadman pod ps
cadman pod run -- ...
```

## Notes

- RunMode: defaults to `cadman` (least privilege).
- No clobber without `--force`.
