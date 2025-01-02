# Copyright(c) Microsoft Corporation.
# Licensed under the MIT License.

#===================================================================================================
# Global Configuration
#===================================================================================================

export RELEASE ?=
export VERBOSE ?= yes
export HTTP_ADDR ?= 127.0.0.1:8080
export FREQUENCY ?= 1000000000
export DURATION ?= 5

#===================================================================================================
# Directories
#===================================================================================================

export ROOT_DIRECTORY := $(CURDIR)
export BINARIES_DIRECTORY := $(ROOT_DIRECTORY)/bin

#===================================================================================================
# Toolchain Configuration
#===================================================================================================

export CARGO ?= $(HOME)/.cargo/bin/cargo

# Common Flags
export CARGO_FLAGS ?= $(if $(RELEASE),--release)

# Guest
export RUSTC_FLAGS_GUEST := "-C panic=abort -C code-model=small -C link-args=-eentrypoint"
export GUEST_TARGET ?= x86_64-unknown-none

#===================================================================================================
# Global Build Rules
#===================================================================================================

export MAKE_DIRECTORY_COMMAND=mkdir -p $(BINARIES_DIRECTORY)
export HOST_RUN_COMMAND=$(BINARIES_DIRECTORY)/host -listen $(HTTP_ADDR) -guest $(BINARIES_DIRECTORY)/guest
export CLIENT_RUN_COMMAND=$(BINARIES_DIRECTORY)/client -connect $(HTTP_ADDR) -frequency $(FREQUENCY) -duration $(DURATION)

all: all-host all-guest all-client

make-directories:
ifeq ($(VERBOSE),)
	@$(MAKE_DIRECTORY_COMMAND)
else
	$(MAKE_DIRECTORY_COMMAND)
endif

run-host: $(BINARIES_DIRECTORY)/host $(BINARIES_DIRECTORY)/guest
ifeq ($(VERBOSE),)
	@$(HOST_RUN_COMMAND)
else
	$(HOST_RUN_COMMAND)
endif

run-client: $(BINARIES_DIRECTORY)/client
ifeq ($(VERBOSE),)
	@$(CLIENT_RUN_COMMAND)
else
	$(CLIENT_RUN_COMMAND)
endif

check: check-host check-guest check-client

clean: clean-host clean-guest check-client
	rm -rf target
	rm -rf $(BINARIES_DIRECTORY)

#===================================================================================================
# Build Rules for "Host" Project
#===================================================================================================

export HOST_BUILD_COMMAND=$(CARGO) build $(CARGO_FLAGS) -p host
export HOST_CHECK_COMMAND=$(CARGO) check $(CARGO_FLAGS) -p host --message-format=json
ifeq ($(RELEASE),)
export HOST_TARGET_DIRECTORY=target/debug
else
export HOST_TARGET_DIRECTORY=target/release
endif

all-host: make-directories
ifeq ($(VERBOSE),)
	@$(HOST_BUILD_COMMAND) --quiet
	@cp $(HOST_TARGET_DIRECTORY)/host $(BINARIES_DIRECTORY)
else
	$(HOST_BUILD_COMMAND)
	cp $(HOST_TARGET_DIRECTORY)/host $(BINARIES_DIRECTORY)
endif

check-host:
	$(CARGO) check $(CARGO_FLAGS) --message-format=json -p host

clean-host:
	$(CARGO) clean -p host

#===================================================================================================
# Build Rules for "Client" Project
#===================================================================================================

export CLIENT_BUILD_COMMAND=$(CARGO) build $(CARGO_FLAGS) -p client
export CLIENT_CHECK_COMMAND=$(CARGO) check $(CARGO_FLAGS) -p client --message-format=json
ifeq ($(RELEASE),)
export CLIENT_TARGET_DIRECTORY=target/debug
else
export CLIENT_TARGET_DIRECTORY=target/release
endif

all-client: make-directories
ifeq ($(VERBOSE),)
	@$(CLIENT_BUILD_COMMAND) --quiet
	@cp $(CLIENT_TARGET_DIRECTORY)/client $(BINARIES_DIRECTORY)
else
	$(CLIENT_BUILD_COMMAND)
	cp $(CLIENT_TARGET_DIRECTORY)/client $(BINARIES_DIRECTORY)
endif

check-client:
	$(CARGO) check $(CARGO_FLAGS) --message-format=json -p client

clean-client:
	$(CARGO) clean -p client

#===================================================================================================
# Build Rules for "Guest" Project
#===================================================================================================

export GUEST_BUILD_COMMAND=RUSTFLAGS=$(RUSTC_FLAGS_GUEST) $(CARGO) build $(CARGO_FLAGS) $(CARGO_FLAGS_GUEST) --target $(GUEST_TARGET) -p guest
export GUEST_CHECK_COMMAND=RUSTFLAGS=$(RUSTC_FLAGS_GUEST) $(CARGO) check $(CARGO_FLAGS) $(CARGO_FLAGS_GUEST) --target $(GUEST_TARGET) -p guest --message-format=json
ifeq ($(RELEASE),)
export GUEST_TARGET_DIRECTORY=target/$(GUEST_TARGET)/debug
else
 GUEST_TARGET_DIRECTORY=target/$(GUEST_TARGET)/release
endif

all-guest: make-directories
ifeq ($(VERBOSE),)
	@$(GUEST_BUILD_COMMAND) --quiet
	@cp $(GUEST_TARGET_DIRECTORY)/guest $(BINARIES_DIRECTORY)
else
	$(GUEST_BUILD_COMMAND)
	cp $(GUEST_TARGET_DIRECTORY)/guest $(BINARIES_DIRECTORY)
endif

check-guest:
	$(GUEST_CHECK_COMMAND)

clean-guest:
	$(CARGO) clean -p guest
