RUST_DIR = ..
RUST_LIB = $(RUST_DIR)/target/debug/liblibb.a

CC = gcc
CFLAGS = -Wall -Wextra -fno-builtin
TESTS = test_putchar test_printn test_printf test_execl test_ctime

all: $(TESTS)

test_putchar: test_putchar.c build_rust
	$(CC) $(CFLAGS) test_putchar.c $(RUST_LIB) -o test_putchar

test_printn: test_printn.c build_rust
	$(CC) $(CFLAGS) test_printn.c $(RUST_LIB) -o test_printn

test_printf: test_printf.c build_rust
	$(CC) $(CFLAGS) test_printf.c $(RUST_LIB) -o test_printf

test_execl: test_execl.c build_rust
	$(CC) $(CFLAGS) test_execl.c $(RUST_LIB) -o test_execl

test_ctime: test_ctime.c build_rust
	$(CC) $(CFLAGS) test_ctime.c $(RUST_LIB) -o test_ctime

build_rust:
	cargo build --manifest-path=$(RUST_DIR)/Cargo.toml

# For release mode
release: build_rust_release $(TESTS:%=%_release)

build_rust_release:
	cargo build --release --manifest-path=$(RUST_DIR)/Cargo.toml

test_putchar_release: test_putchar.c
	$(CC) $(CFLAGS) test_putchar.c $(RUST_DIR)/target/release/liblibb.a -o test_putchar

test_printn_release: test_printn.c
	$(CC) $(CFLAGS) test_printn.c $(RUST_DIR)/target/release/liblibb.a -o test_printn

test_printf_release: test_printf.c
	$(CC) $(CFLAGS) test_printf.c $(RUST_DIR)/target/release/liblibb.a -o test_printf

test_execl_release: test_execl.c
	$(CC) $(CFLAGS) test_execl.c $(RUST_DIR)/target/release/liblibb.a -o test_execl

test_ctime_release: test_ctime.c
	$(CC) $(CFLAGS) test_ctime.c $(RUST_DIR)/target/release/liblibb.a -o test_ctime

run: all
	@echo "=== Running test_putchar ==="
	./test_putchar
	@echo "=== Running test_printn ==="
	./test_printn
	@echo "=== Running test_printf ==="
	./test_printf
	@echo "=== Running test_execl ==="
	./test_execl
	@echo "=== Running test_ctime ==="
	./test_ctime

clean:
	rm -f $(TESTS) test_prog
	cargo clean --manifest-path=$(RUST_DIR)/Cargo.toml

.PHONY: all build_rust build_rust_release release run clean
