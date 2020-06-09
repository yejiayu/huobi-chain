CURRENT_DIR := $(shell pwd)
DOCKER_BUILD := docker run --rm -it -v $(CURRENT_DIR):/contract nervos/ckb-riscv-gnu-toolchain:xenial bash -c

# cc in docker image
CC := riscv64-unknown-elf-gcc
SRC := /contract/src
INCLUDE := /contract/include
TARGET := /contract/target
LDFLAGS := -lm -Wl,-static -fdata-sections -ffunction-sections -Wl,--gc-sections -Wl,-s


build-all: simple_storage.bin

# run the docker image and change directory to /contract which we mounted by -v $(CURRENT_DIR):/contract
# and run the blow target SEPARATELY in docker container
simple_storage.bin:
	$(DOCKER_BUILD) "cd /contract && make simple_storage"

# this target is run in docker container, not in host
# call the risc-v compiler to compile smart contract and output binary file to 'target' folder
# it must use static link and precompiled libpvm.a is preferred
simple_storage:
	$(CC) -I$(INCLUDE) $(SRC)/simple_storage.c $(INCLUDE)/libpvm.a $(LDFLAGS) -o $(TARGET)/simple_storage.bin
