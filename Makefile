RUSTC=~/projects/src/rust/x86_64-unknown-linux-gnu/stage2/bin/rustc

log: log.rs 
	$(RUSTC) -O $<
main: main.rs 
	$(RUSTC) $<
