#!/usr/bin/env bash
set -e

# Hardcoded port for local dev
HTTP_PORT=8080
HTTPS_PORT=8443
ADMIN_PORT=8000


docker build -t cadman -f ./docker/base/Dockerfile .
docker run --rm -it \
  -p ${HTTP_PORT}:80 \
  -p ${HTTPS_PORT}:443 \
  -p ${ADMIN_PORT}:8080 \
  cadman
