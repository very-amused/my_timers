# This makefile is only for release builds
# development builds are still done via `cargo build`

# Build vars
rustflags=--release --quiet --message-format json
rsync-flags=-h --size-only --info=progress2
src=src Cargo.toml Cargo.lock .cargo
targets=x86_64-unknown-linux-gnu x86_64-unknown-linux-musl x86_64-unknown-freebsd
prefix=sed -e 's/^/\x1b[1m[$@]\x1b[0m /'
releasename=`echo $@ | sed 's/-unknown//; s/-gnu//'`
bin-ext=`case $@ in *pc-windows*) echo '.exe' ;; esac`
version=$(shell git tag | tail -n 1 | sed 's/^v//')
vms=freebsd-cc void-cc win10-ltsc

# Installation vars
ifndef PREFIX
PREFIX=/usr/local
endif
ifndef DATADIR
DATADIR=$(PREFIX)/share
endif

# Cross compile using VMs
define cc
$(call start-vm,$<)
$(call poll-vm,$<)
ssh $< "[ -d my_timers ] || mkdir my_timers"
rsync $(rsync-flags) -r $(src) $<:my_timers/
ssh $< "cd my_timers && cargo build $(rustflags) --target $@ | $(prefix)"
rm -rf target/$@
rsync $(rsync-flags) -r $<:my_timers/target/$@ target/
$(pack)
endef

# Package a release
define pack
[ -d release/$(releasename) ] || mkdir -p release/$(releasename)
cp target/$@/release/my_timers$(bin-ext) Makefile README.md LICENSE \
	release/$(releasename)/
tar czf release/$(releasename).tar.gz release/$(releasename)/*
zip -r release/$(releasename).zip release/$(releasename)/*
endef

# IMPORTANT: ensure VMs are fully shutdown before this script attempts to start them
define start-vm
sudo virsh start $1 || true
endef

# Poll a VM until port 22 is open
define poll-vm
while ! nc -z $1 22; do :; done
endef

define shutdown-vm
sudo virsh shutdown $1 || true
endef

define update-nsis
sed -e 's/^!define DISPLAY_VERSION.*$$/!define DISPLAY_VERSION "$(version)"/' -i "$1"
endef

all: clean-local .WAIT $(targets)

install: my_timers README.md LICENSE
	install -d $(DESTDIR)$(PREFIX)/bin
	install my_timers $(DESTDIR)$(PREFIX)/bin/my_timers
	install -d $(DESTDIR)$(DATADIR)/doc/my_timers
	install -m644 README.md $(DESTDIR)$(DATADIR)/doc/my_timers/README.md
	install -d $(DESTDIR)$(DATADIR)/licenses/my_timers
	install -m644 LICENSE $(DESTDIR)$(DATADIR)/licenses/my_timers/LICENSE

uninstall:
	rm -f $(DESTDIR)$(PREFIX)/bin/my_timers
	rm -rf $(DESTDIR)$(DATADIR)/doc/my_timers
	rm -rf $(DESTDIR)$(DATADIR)/licenses/my_timers

nsis: win10-ltsc README.md LICENSE installer/my_timers.nsi
	$(call update-nsis,installer/my_timers.nsi)
	cp installer/*.nsi /mnt/$</NSIS/my_timers/
	cp README.md LICENSE /mnt/$</NSIS/my_timers/
	ssh $< "cp ~/my_timers/target/x86_64-pc-windows-msvc/release/my_timers.exe ~/Desktop/Shared/NSIS/my_timers/"
	ssh $< "cd ~/Desktop/Shared/NSIS/my_timers; makensis.exe *.nsi"
.PHONY: nsis

local: x86_64-unknown-linux-gnu

x86_64-unknown-linux-gnu:
	cargo build $(rustflags) --target $@ | $(prefix)
	cp target/$@/release/my_timers my_timers
	$(pack)

x86_64-unknown-linux-musl: void-cc
	$(cc)

x86_64-unknown-freebsd: freebsd-cc
	$(cc)

x86_64-pc-windows-msvc: win10-ltsc
	$(cc)

clean-local:
	rm -rf target release

# Start all VMs
start-vms: $(vms)
	$(foreach vm,$(vms),$(call start-vm,$(vm));)

# Poll for all VMs to start
poll-vms: $(vms)
	$(foreach vm,$(vms),$(call poll-vm,$(vm));)

# Shutdown all VMs
shutdown-vms: $(vms)
	$(foreach vm,$(vms),$(call shutdown-vm,$(vm));)

# Clean all VMs
clean-vms: $(vms)
	$(foreach vm,$(vms),ssh $(vm) "rm -rf my_timers";)

clean: clean-local start-vms poll-vms .WAIT clean-vms .WAIT shutdown-vms

.PHONY: all local install uninstall $(vms) $(targets) clean-local start-vms poll-vms shutdown-vms clean-vms clean
