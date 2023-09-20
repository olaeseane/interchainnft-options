.DEFAULT_GOAL := compile

COSMOWASM_WORKSPACE_OPTIMIZER_CMD = docker run --rm -v "$(shell pwd)":/code \
	--mount type=volume,source="$(shell basename "$(shell pwd)")_cache",target=/target \
	--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
	cosmwasm/workspace-optimizer:0.14.0
COSMOWASM_CHECK_CMD := cosmwasm-check target/wasm32-unknown-unknown/release/*.wasm

# Targets and rules
compile:
	$(COSMOWASM_WORKSPACE_OPTIMIZER_CMD)

check:
	$(COSMOWASM_CHECK_CMD)

co: compile
ch: check

.PHONY: compile check co cc