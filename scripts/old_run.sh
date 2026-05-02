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

# docker run -d -p 8080:8080 -p 80:80 -p 443:443 \
#     -v $PWD/cadman.yml:/etc/cadman/cadman.yml \
#     -v /var/run/docker.sock:/var/run/docker.sock \
#     cadman:v0.1.0