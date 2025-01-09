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

# Hyperlight Guest
export RUSTC_FLAGS_HYPERLIGHT_GUEST := "-C panic=abort -C code-model=small -C link-args=-eentrypoint"
export HYPERLIGHT_GUEST_TARGET ?= x86_64-unknown-none

#===================================================================================================
# Global Build Rules
#===================================================================================================

export MAKE_DIRECTORY_COMMAND=mkdir -p $(BINARIES_DIRECTORY)
export HYPERLIGHT_HOST_RUN_COMMAND=$(BINARIES_DIRECTORY)/hyperlight-host-nanvix -listen $(HTTP_ADDR) -guest $(BINARIES_DIRECTORY)/hyperlight-guest-nanvix
export CLIENT_RUN_COMMAND=$(BINARIES_DIRECTORY)/client -connect $(HTTP_ADDR) -frequency $(FREQUENCY) -duration $(DURATION)

all: all-hyperlight-host all-hyperlight-guest all-client all-http-echo

make-directories:
ifeq ($(VERBOSE),)
	@$(MAKE_DIRECTORY_COMMAND)
else
	$(MAKE_DIRECTORY_COMMAND)
endif

run-hyperlight-host: $(BINARIES_DIRECTORY)/hyperlight-host-nanvix $(BINARIES_DIRECTORY)/hyperlight-guest-nanvix
ifeq ($(VERBOSE),)
	@$(HYPERLIGHT_HOST_RUN_COMMAND)
else
	$(HYPERLIGHT_HOST_RUN_COMMAND)
endif

run-client: $(BINARIES_DIRECTORY)/client
ifeq ($(VERBOSE),)
	@$(CLIENT_RUN_COMMAND)
else
	$(CLIENT_RUN_COMMAND)
endif

check: check-hyperlight-host check-hyperlight-guest check-client check-http-echo

clean: clean-hyperlight-host clean-hyperlight-guest clean-client clean-http-echo
	rm -rf target
	rm -rf $(BINARIES_DIRECTORY)

#===================================================================================================
# Build Rules for "Host" Project
#===================================================================================================

export HYPERLIGHT_HOST_BUILD_COMMAND=$(CARGO) build $(CARGO_FLAGS) -p hyperlight-host-nanvix
export HYPERLIGHT_HOST_CHECK_COMMAND=$(CARGO) check $(CARGO_FLAGS) -p hyperlight-host-nanvix --message-format=json
ifeq ($(RELEASE),)
export HYPERLIGHT_HOST_TARGET_DIRECTORY=target/debug
else
export HYPERLIGHT_HOST_TARGET_DIRECTORY=target/release
endif

all-hyperlight-host: make-directories
ifeq ($(VERBOSE),)
	@$(HYPERLIGHT_HOST_BUILD_COMMAND) --quiet
	@cp $(HYPERLIGHT_HOST_TARGET_DIRECTORY)/hyperlight-host-nanvix $(BINARIES_DIRECTORY)
else
	$(HYPERLIGHT_HOST_BUILD_COMMAND)
	cp $(HYPERLIGHT_HOST_TARGET_DIRECTORY)/hyperlight-host-nanvix $(BINARIES_DIRECTORY)
endif

check-hyperlight-host:
	$(CARGO) check $(CARGO_FLAGS) --message-format=json -p hyperlight-host-nanvix

clean-hyperlight-host:
	$(CARGO) clean -p hyperlight-host-nanvix
	rm -f $(BINARIES_DIRECTORY)/hyperlight-host-nanvix

#===================================================================================================
# Build Rules for "hyperlight-guest" Project
#===================================================================================================

export HYPERLIGHT_GUEST_BUILD_COMMAND=RUSTFLAGS=$(RUSTC_FLAGS_HYPERLIGHT_GUEST) $(CARGO) build $(CARGO_FLAGS) $(CARGO_FLAGS_GUEST) --target $(HYPERLIGHT_GUEST_TARGET) -p hyperlight-guest-nanvix
export HYPERLIGHT_GUEST_CHECK_COMMAND=RUSTFLAGS=$(RUSTC_FLAGS_HYPERLIGHT_GUEST) $(CARGO) check $(CARGO_FLAGS) $(CARGO_FLAGS_GUEST) --target $(HYPERLIGHT_GUEST_TARGET) -p hyperlight-guest-nanvix --message-format=json
ifeq ($(RELEASE),)
export HYPERLIGHT_GUEST_TARGET_DIRECTORY=target/$(HYPERLIGHT_GUEST_TARGET)/debug
else
  HYPERLIGHT_GUEST_TARGET_DIRECTORY=target/$(HYPERLIGHT_GUEST_TARGET)/release
endif

all-hyperlight-guest: make-directories
ifeq ($(VERBOSE),)
	@$(HYPERLIGHT_GUEST_BUILD_COMMAND) --quiet
	@cp $(HYPERLIGHT_GUEST_TARGET_DIRECTORY)/hyperlight-guest-nanvix $(BINARIES_DIRECTORY)
else
	$(HYPERLIGHT_GUEST_BUILD_COMMAND)
	cp $(HYPERLIGHT_GUEST_TARGET_DIRECTORY)/hyperlight-guest-nanvix $(BINARIES_DIRECTORY)
endif

check-hyperlight-guest:
	$(HYPERLIGHT_GUEST_CHECK_COMMAND)

clean-hyperlight-guest:
	$(CARGO) clean -p hyperlight-guest-nanvix
	rm -f $(BINARIES_DIRECTORY)/hyperlight-guest-nanvix

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
	rm -f $(BINARIES_DIRECTORY)/client

#===================================================================================================
# Build Rules for "Rust HTTP Server" Project
#===================================================================================================

export HTTP_ECHO_BUILD_COMMAND=$(CARGO) build $(CARGO_FLAGS) -p rust-http-echo
export HTTP_ECHO_CHECK_COMMAND=$(CARGO) check $(CARGO_FLAGS) -p rust-http-echo --message-format=json
ifeq ($(RELEASE),)
export HTTP_ECHO_TARGET_DIRECTORY=target/debug
else
export HTTP_ECHO_TARGET_DIRECTORY=target/release
endif

all-http-echo: make-directories
ifeq ($(VERBOSE),)
	@$(HTTP_ECHO_BUILD_COMMAND) --quiet
	@cp $(HTTP_ECHO_TARGET_DIRECTORY)/rust-http-echo $(BINARIES_DIRECTORY)
else
	$(HTTP_ECHO_BUILD_COMMAND)
	cp $(HTTP_ECHO_TARGET_DIRECTORY)/rust-http-echo $(BINARIES_DIRECTORY)
endif

check-http-echo:
	$(CARGO) check $(CARGO_FLAGS) --message-format=json -p rust-http-echo 

clean-http-echo:
	$(CARGO) clean -p rust-http-echo 
	rm -f $(BINARIES_DIRECTORY)/rust-http-echo