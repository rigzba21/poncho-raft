#use this makefile to setup and provision local raft network

all: create_dirs cargo_build

create_dirs:
	mkdir node1
	mkdir node2
	mkdir node3
	mkdir node4
	mkdir node5

cargo_build:
	cargo build
	cp target/debug/raftkv node1
	cp target/debug/raftkv node2
	cp target/debug/raftkv node3
	cp target/debug/raftkv node4
	cp target/debug/raftkv node5

clean:
	rm -rf node1
	rm -rf node2
	rm -rf node3
	rm -rf node4
	rm -rf node5
