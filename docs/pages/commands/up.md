# cadman up

> Scope: project
> 
> Output (read-only): `table|json|yaml`
> 
> Exit codes: `0` success; `1` usage; `2` not found; `3` dependency missing; `4` validation; `5+` unexpected

## Synopsis

```bash
cadman up [--output table|json|yaml]
```

## Description

Starts services defined in `cadman.yaml` (FR-3). Applies service inference when needed (FR-8). Allocates ports if unspecified (FR-6).

## Global flags (honoured here)

- `--verbose` `--debug` `--quiet`
- `--output table|json|yaml` *(read-only)*

## Examples

```bash
cadman up
cadman up --output json
```

## Notes

- RunMode: defaults to `cadman` (least privilege).
- No clobber without `--force`.
