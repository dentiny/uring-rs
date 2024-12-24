# uring-rs
A benchmark playground for io-uring in rust.

### Executable
```shell
# Debug build and execute.
cargo build --bin io_uring_multi_read
./target/debug/io_uring_multi_read

# Optimal build and execute.
cargo build --release --bin io_uring_multi_read
./target/release/io_uring_multi_read
```

### TODO items
- [x] Basic implementation for multi-read with io uring
- [ ] Basic implementation for multi-read with mmap
- [ ] Tune the parameter for page size
- [ ] Implement concurrent read and write
