set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

default:
    just --list

build:
    ./scripts/cadman-build.sh

run *args:
    ./scripts/cadman-run.sh {{args}}

test:
    ./scripts/cadman-test.sh

test-coverage:
    ./scripts/cadman-test-coverage.sh

lint:
    ./scripts/cadman-lint.sh

fmt:
    ./scripts/cadman-fmt.sh

clean:
    ./scripts/cadman-clean.sh

check-license:
    ./scripts/cadman-license.sh

image-run *args:
    ./scripts/cadman-image-run.sh {{args}}

docs-build:
    ./scripts/docs-build.sh

docs-serve:
    ./scripts/docs-serve.sh

ci-check:
    ./scripts/ci-check.sh

setup:
    ./scripts/setup.sh

crate-check:
    ./scripts/crate-release-check.sh