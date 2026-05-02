#!/usr/bin/env bash
set -e

# Hardcoded port for local dev
PORT=8000

docker build -t mkdocs-custom -f ./docker/docs/Dockerfile .
docker run --rm -it \
  -p ${PORT}:8000 \
  -v "$(pwd)/docs":/docs \
  mkdocs-custom \
  serve -f mkdocs.yml -a 0.0.0.0:8000
