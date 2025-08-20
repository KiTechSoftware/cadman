# cadman compose

> Scope: wrapper
> 
> Output (read-only): `table|json|yaml`
> 
> Exit codes: `0` success; `1` usage; `2` not found; `3` dependency missing; `4` validation; `5+` unexpected

## Synopsis

```bash
cadman compose [-- ...passthrough flags...]
```

## Description

Runs Podman Compose in the correct RunMode. Compose file resolution preference: `podman-compose.yaml` → `docker-compose.yaml` (SRS §5.3).

## Global flags (honoured here)

- `--verbose` `--debug` `--quiet`

## Examples

```bash
cadman compose up -d
cadman compose logs --tail 200
```

## Notes

- RunMode: defaults to `cadman` (least privilege).
- No clobber without `--force`.
