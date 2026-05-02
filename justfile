set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

default: 
    just --list

build:
    ./scripts/build.sh

run *args:
    ./scripts/run.sh {{args}}

test:
    ./scripts/test.sh

test-coverage:
    ./scripts/test-coverage.sh

lint:
    ./scripts/lint.sh

fmt:
    ./scripts/fmt.sh

clean:
    ./scripts/clean.sh

check-license:
    ./scripts/license.sh
