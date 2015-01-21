THIS_MAKEFILE_PATH:=$(word $(words $(MAKEFILE_LIST)),$(MAKEFILE_LIST))
THIS_DIR:=$(shell cd $(dir $(THIS_MAKEFILE_PATH));pwd)

test:
	cargo test

build:
	cargo build

docs:
	cd "$(THIS_DIR)"
	cp src/lib.rs code.bak
	cat README.md | sed -e 's/^/\/\/! /g' > readme.bak
	sed -i '/\/\/ DOCS/r readme.bak' src/lib.rs
	(cargo doc --no-deps && make clean) || (make clean && false)

clean:
	cd "$(THIS_DIR)"
	mv code.bak src/lib.rs || true
	rm *.bak || true

upload:
	cd "$(THIS_DIR)"
	echo '<!doctype html><title>rust-atomicwrites</title><meta http-equiv="refresh" content="0; ./atomicwrites/">' \
		> ./target/doc/index.htm
	rsync -av --chmod=755 ./target/doc/ untispace:~/virtual/rust-atomicwrites.unterwaditzer.net/
