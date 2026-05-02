.PHONY: \
	build run test lint fmt clean check-license test-coverage image-run \
	docs-build docs-serve \
	ci-check dev-setup setup crate-check

build:
	./scripts/cadman-build.sh

run:
	./scripts/cadman-run.sh $(ARGS)

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

image-run:
	./scripts/cadman-image-run.sh $(ARGS)

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