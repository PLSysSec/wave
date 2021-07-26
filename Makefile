.PHONY: bootstrap build verify prusti

# Prusti setup and build instructions: 
# https://viperproject.github.io/prusti-dev/dev-guide/development/setup.html
# https://viperproject.github.io/prusti-dev/dev-guide/development/build.html

# Used by Prusti
JAVA_HOME ?= /usr/lib/jvm/ 

bootstrap:
	git submodule update --init --recursive
	cd prusti-dev && ./x.py setup
	cd prusti-dev && JAVA_HOME=$(JAVA_HOME) ./x.py build --release

prusti:
	cd prusti-dev && JAVA_HOME=$(JAVA_HOME) ./x.py build --release

build:
	cargo build --release

# If this command is giving you trouble, try deleting the ./target directory and retrying
verify:
	prusti-dev/target/release/cargo-prusti --features verify
