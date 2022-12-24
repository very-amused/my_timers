# This makefile is only for release builds
# development builds are still done via `cargo build`
rustflags=--release --quiet --message-format json
rsync-flags=-h --size-only --info=progress2
src=src Cargo.toml Cargo.lock .cargo
targets=x86_64-unknown-linux-gnu x86_64-unknown-freebsd
prefix=sed -e 's/^/\x1b[1m[$@]\x1b[0m /'

# Cross compile using VMs
define cc =
ssh $^ "[ -d my_timers ] || mkdir my_timers"
rsync $(rsync-flags) -r $(src) $^:my_timers/
ssh $^ "cd my_timers && cargo build $(rustflags) --target $@ | $(prefix)"
rm -rf target/$@
rsync $(rsync-flags) -r $^:my_timers/target/$@ target/
endef

all: $(targets)

x86_64-unknown-linux-gnu:
	cargo build $(rustflags) --target x86_64-unknown-linux-gnu | $(prefix)

x86_64-unknown-freebsd: freebsd-cc
	$(cc)

clean:
	rm -rf target
	ssh freebsd-cc "rm -rf my_timers"

.PHONY: $(targets) freebsd-cc clean
