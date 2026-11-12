BIN      := nitrite
NAME     := nit
BINDIR   ?= /usr/bin
CARGO    ?= cargo
PROFILE  ?= release
TARGET   := target/$(PROFILE)/$(BIN)

# Use sudo only when the destination directory is not writable by the current user.
SUDO     := $(shell if [ -w "$(BINDIR)" ]; then echo ""; else echo sudo; fi)

.PHONY: all build install uninstall clean

all: build install

build:
	$(CARGO) build --$(PROFILE)

install: build
	$(SUDO) install -Dm755 "$(TARGET)" "$(BINDIR)/$(NAME)"

uninstall:
	$(SUDO) rm -f "$(BINDIR)/$(NAME)"

clean:
	$(CARGO) clean
