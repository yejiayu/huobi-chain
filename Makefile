ERBOSE := $(if ${CI},--verbose,)

ifneq ("$(wildcard /usr/lib/librocksdb.so)","")
	SYS_LIB_DIR := /usr/lib
else ifneq ("$(wildcard /usr/lib64/librocksdb.so)","")
	SYS_LIB_DIR := /usr/lib64
else
	USE_SYS_ROCKSDB :=
endif

SYS_ROCKSDB := $(if ${USE_SYS_ROCKSDB},ROCKSDB_LIB_DIR=${SYS_LIB_DIR},)

CARGO := env ${SYS_ROCKSDB} cargo

test:
	${CARGO} test ${VERBOSE} --all -- --nocapture

doc:
	cargo doc --all --no-deps

doc-deps:
	cargo doc --all

# generate GraphQL API documentation
doc-api:
	bash docs/build/gql_api.sh

check:
	${CARGO} check ${VERBOSE} --all

build:
	${CARGO} build ${VERBOSE} --release

prod:
	${CARGO} build ${VERBOSE} --release

prod-test:
	${CARGO} test ${VERBOSE} --all -- --nocapture

fmt:
	cargo fmt ${VERBOSE} --all -- --check

clippy:
	${CARGO} clippy ${VERBOSE} --all --all-targets --all-features -- \
		-D warnings -D clippy::clone_on_ref_ptr -D clippy::enum_glob_use


ci: fmt clippy test
	git diff --exit-code Cargo.lock

info:
	date
	pwd
	env

docker-build:
	docker build -t huobi .

# For counting lines of code
stats:
	@cargo count --version || cargo +nightly install --git https://github.com/kbknapp/cargo-count
	@cargo count --separator , --unsafe-statistics

# Use cargo-audit to audit Cargo.lock for crates with security vulnerabilities
# expecting to see "Success No vulnerable packages found"
security-audit:
	@cargo audit --version || cargo install cargo-audit
	@cargo audit

.PHONY: build prod prod-test
.PHONY: fmt test clippy doc doc-deps doc-api check stats
.PHONY: ci info security-audit

# e2e-test:
# 	@echo "-----------------------------------------------------------------"
# 	@echo "run the commands below in another window first:                  "
# 	@echo "                                                                 "
# 	@echo "rm -rf ./target/tests/e2e/data && cargo run --release -- -c tests/e2e/chain.toml -g tests/e2e/genesis.toml"
# 	@echo "-----------------------------------------------------------------"
# 	cd tests/e2e && npm i && ./wait-for-it.sh -t 300 localhost:8000 -- npm run test
e2e-test:
	@echo hello

e2e-test-via-docker:
	docker-compose -f tests/e2e/docker-compose-e2e-test.yaml up --exit-code-from e2e-test --force-recreate

change-validator-test-via-docker:
	@echo "if you changed the binary code, you may want to run `make docker-build` first"
	cd tests/e2e && npm i && npx ts-node change_validators/create_configs.ts
	docker-compose -f tests/e2e/change_validators/configs/docker-compose.yml up --exit-code-from change-validators-test --force-recreate --remove-orphans

# For riscv service
TARGET := riscv64-unknown-elf
CC := $(TARGET)-gcc
LD := $(TARGET)-gcc
CFLAGS := -Os -DCKB_NO_MMU -D__riscv_soft_float -D__riscv_float_abi_soft
LDFLAGS := -lm -Wl,-static -fdata-sections -ffunction-sections -Wl,--gc-sections -Wl,-s
CURRENT_DIR := $(shell pwd)
DOCKER_BUILD := docker run --rm -it -v $(CURRENT_DIR):/src nervos/ckb-riscv-gnu-toolchain:xenial bash -c
TEST_SRC := $(CURRENT_DIR)/services/riscv/src/tests
E2E_TEST_SRC := $(CURRENT_DIR)/tests/e2e/riscv_contracts
RISCV_SRC := $(CURRENT_DIR)/services/riscv/src/vm/c
DUKTAPE_SRC := $(RISCV_SRC)/duktape

simple_storage: libpvm.a
	$(CC) -I$(RISCV_SRC) $(TEST_SRC)/simple_storage.c $(RISCV_SRC)/libpvm.a $(LDFLAGS) -o $(TEST_SRC)/simple_storage

simple_storage_docker:
	$(DOCKER_BUILD) "cd /src && make simple_storage"

write_read: libpvm.a
	$(CC) -I$(RISCV_SRC) $(TEST_SRC)/write_read.c $(RISCV_SRC)/libpvm.a $(LDFLAGS) -o $(TEST_SRC)/write_read

write_read_docker:
	$(DOCKER_BUILD) "cd /src && make write_read"

assert: libpvm.a
	$(CC) -I$(RISCV_SRC) $(TEST_SRC)/assert.c $(RISCV_SRC)/libpvm.a $(LDFLAGS) -o $(TEST_SRC)/assert

assert_docker:
	$(DOCKER_BUILD) "cd /src && make assert"

contract_test: libpvm.a
	$(CC) -I$(RISCV_SRC) $(E2E_TEST_SRC)/contract_test.c $(RISCV_SRC)/libpvm.a $(LDFLAGS) -o $(E2E_TEST_SRC)/contract_test

contract_test_docker:
	$(DOCKER_BUILD) "cd /src && make contract_test"

libpvm.a:
	$(CC) -I$(RISCV_SRC) -c $(RISCV_SRC)/pvm.c -o /tmp/pvm.o
	ar rcs $(RISCV_SRC)/libpvm.a /tmp/pvm.o

libpvm.a_docker:
	$(DOCKER_BUILD) "cd /src && make libpvm.a"

pvm_general_test: libpvm.a
	$(CC) -I$(RISCV_SRC) $(TEST_SRC)/general.c $(RISCV_SRC)/libpvm.a $(LDFLAGS) -o $(TEST_SRC)/general

pvm_general_test_docker:
	$(DOCKER_BUILD) "cd /src && make pvm_general_test"
