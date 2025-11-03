LAKE ?= lake

.PHONY: all run lake

all: run

MAP_ARRAY_DIR := ../lean/map_array

lake:
	$(LAKE) --dir=$(MAP_ARRAY_DIR) build @/MapArray:static

OUT_DIR = out
LEAN_SYSROOT ?= $(shell lean --print-prefix)
SYSROOT_LIBDIR := $(LEAN_SYSROOT)/lib
LEAN_LIBDIR := $(LEAN_SYSROOT)/lib/lean

$(OUT_DIR):
	mkdir -p $@

$(OUT_DIR)/main: main.c lake | $(OUT_DIR)
# Add library paths for Lake package and for Lean itself
#
# Note: How to find which library contains a given symbol:
# ```
# export SYMBOL=foo
# find . -name \*.a -exec bash -c "nm --defined-only {} 2>/dev/null | grep $SYMBOL && echo {}" \;
# ```
# (Reference: <https://stackoverflow.com/questions/19916119/how-do-i-find-where-a-symbol-is-defined-among-static-libraries>)
#
# Library paths are given explicitly below, instead of using `-l`,
# to avoid linking shared libraries instead of colocated static libraries.
# (Reference: <https://stackoverflow.com/questions/5604061/making-gcc-prefer-static-libs-to-shared-objects-when-linking>)
#
	cc -o $@ $< -I $(LEAN_SYSROOT)/include \
		$(MAP_ARRAY_DIR)/.lake/build/lib/libMapArray.a \
		$(LEAN_LIBDIR)/libInit.a \
		$(LEAN_LIBDIR)/libleanrt.a \
		$(SYSROOT_LIBDIR)/libuv.a \
		$(SYSROOT_LIBDIR)/libgmp.a \
		$(SYSROOT_LIBDIR)/libc++.a \
		$(SYSROOT_LIBDIR)/libc++abi.a \
		-lm

run: $(OUT_DIR)/main
ifeq ($(OS),Windows_NT)
# Add shared library paths to loader path dynamically
	env PATH="$(MAP_ARRAY_DIR)/.lake/build/lib:$(shell cygpath $(LEAN_SYSROOT))/bin:$(PATH)" $(OUT_DIR)/main
else
	$(OUT_DIR)/main
endif
