//! File I/O for signals.
//! This is useful for testing the signal processing chain.

use std::fs::File;
use std::io::Write;
use num::Complex;

type StreamType = Complex<f32>;

pub struct FileIoConfig<'a> {
    /// Processing block length in samples
    pub blocklen: usize,
    /// Sample rate
    pub fs: f64,
    /// Stop after this number of nanoseconds
    pub stop_time: i64,
    /// Output file name for transmit signal.
    pub tx_filename: &'a str,
}

pub struct FileIo {
    tx_file: File,
    time: i64,
    time_per_buf: i64,
    stop_time: i64,
    buf: Vec<StreamType>,
}

impl FileIo {
    pub fn new(conf: &FileIoConfig) -> Option<Self> {
        Some(Self {
            tx_file: match File::create(conf.tx_filename) {
                Ok(file) => file,
                Err(err) => {
                    eprintln!("Failed to open TX file: {}", err);
                    return None;
                }
            },
            time: 0,
            time_per_buf: (conf.blocklen as f64 * 1e9 / conf.fs).round() as i64,
            stop_time: conf.stop_time,
            buf: vec![num::zero(); conf.blocklen],
        })
    }

    pub fn process<F>(&mut self, mut process_signal: F) -> Option<()>
        where F: FnMut(&mut [Complex<f32>], i64, i64)
    {
        let buf_slice = &mut self.buf[..];
        // Use zero as RX signal for now
        for v in &mut *buf_slice { *v = num::zero(); }

        process_signal(&mut *buf_slice, self.time, self.time);

        // Let's be a bit lazy and use transmute to write the buffer to file.
        // Yes, the file format ends up depending on machine endianness etc,
        // so it's unsafe.
        // This is for initial testing purposes only.
        match self.tx_file.write_all(
        unsafe { std::mem::transmute::<&[StreamType], &[u8]>(buf_slice) }) {
            Ok(_) => {},
            Err(err) => {
                eprintln!("Failed to write TX file: {}", err);
                return None;
            }
        };

        self.time += self.time_per_buf;
        if self.time >= self.stop_time { None } else { Some(()) }
    }
}
