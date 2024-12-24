// This program uses io-uring to perform asynchronous IO to achieve multi-read for a single giant file.
// It assumes the existence of "/tmp/io_uring_test", which is of size 10GiB.
// Command to prepare: dd if=/dev/zero bs=1M count=10240 | tr '\0' 'a' > /tmp/io_uring_test
//
// TODO(hjiang):
// 1. Perform multi-read after stating the file size.
// 2. Make read size configurable so we could check read performance.

use std::io::Write;
use std::os::unix::io::AsRawFd;
use std::process::Command;
use std::{fs, io};

fn clear_page_cache() -> std::io::Result<()> {
    // Step 1: Perform a sync to flush dirty pages
    Command::new("sync").status()?;

    // Step 2: Write to `/proc/sys/vm/drop_caches` to clear caches
    let mut file = fs::OpenOptions::new()
        .write(true)
        .open("/proc/sys/vm/drop_caches")?;

    // Write "3" to drop caches (1 = page cache, 2 = dentries and inodes, 3 = both)
    file.write_all(b"3")?;

    Ok(())
}

fn main() -> io::Result<()> {
    // Clear the page cache before benchmark.
    clear_page_cache()?;

    const FILENAME: &str = "/tmp/io_uring_test";
    const FILESIZE: u64 = 10 * 1024 * 1024 * 1024; // 10GiB
    const READSIZE: u64 = 512 * 1024; // 512KiB
    const IO_SIZE: u32 = (FILESIZE / READSIZE) as u32;

    let mut ring = io_uring::IoUring::new(IO_SIZE)?;
    let fd = fs::File::open(FILENAME)?;

    let mut buffers: Vec<Vec<u8>> = (0..IO_SIZE).map(|_| vec![0; READSIZE as usize]).collect();

    // Submit read requests one by one.
    for (idx, buf) in buffers.iter_mut().enumerate() {
        let offset = (idx * READSIZE as usize) as u64;

        let read_entry = io_uring::opcode::Read::new(
            io_uring::types::Fd(fd.as_raw_fd()),
            buf.as_mut_ptr(),
            READSIZE as _,
        )
        .offset(offset)
        .build()
        .user_data(idx as u64);

        unsafe {
            ring.submission()
                .push(&read_entry)
                .expect("submission queue is full");
        }

        ring.submit()?;
    }

    // Reap responses one by one.
    for _ in 0..IO_SIZE {
        ring.submit_and_wait(1)?;

        if let Some(cqe) = ring.completion().next() {
            let result = cqe.result();
            assert!(result >= 0, "Checking result for crq gets {}", result);
        }
    }

    Ok(())
}
