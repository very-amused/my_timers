# This makefile is only for release builds
# development builds are still done via `cargo build`
rustflags=--release --quiet --message-format json
rsync-flags=-h --size-only --info=progress2
src=src Cargo.toml Cargo.lock .cargo
targets=x86_64-unknown-linux-gnu x86_64-unknown-freebsd
prefix=sed -e 's/^/\x1b[1m[$@]\x1b[0m /'
vms=freebsd-cc

# Cross compile using VMs
define cc =
$(start-vm)
$(poll-vm)
ssh $^ "[ -d my_timers ] || mkdir my_timers"
rsync $(rsync-flags) -r $(src) $^:my_timers/
ssh $^ "cd my_timers && cargo build $(rustflags) --target $@ | $(prefix)"
rm -rf target/$@
rsync $(rsync-flags) -r $^:my_timers/target/$@ target/
$(shutdown-vm)
endef

# IMPORTANT: ensure VMs are fully shutdown before this script attempts to start them
define start-vm =
sudo virsh start $^ || true
endef

# Poll a VM until port 22 is open
define poll-vm =
while ! nc -z $^ 22; do :; done
endef

define shutdown-vm =
sudo virsh shutdown $^ || true
endef

all: $(targets)

x86_64-unknown-linux-gnu:
	cargo build $(rustflags) --target x86_64-unknown-linux-gnu | $(prefix)

x86_64-unknown-freebsd: freebsd-cc
	$(cc)

clean-local:
	rm -rf target

# Start all VMs
start-vms: $(vms)
	$(start-vm)

# Poll for all VMs to start
poll-vms: $(vms)
	$(poll-vm)

# Shutdown all VMs
shutdown-vms: $(vms)
	$(shutdown-vm)

# Clean all VMs
clean-vms: $(vms)
	ssh $^ "rm -rf my_timers"

clean: clean-local start-vms poll-vms .WAIT clean-vms .WAIT shutdown-vms

.PHONY: $(vms)
