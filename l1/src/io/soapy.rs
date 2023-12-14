use num::Complex;
use soapysdr;

type StreamType = Complex<f32>;

pub struct SoapyIoConfig<'a> {
    /// Processing block length in samples
    pub blocklen: usize,
    /// RX-TX round-trip latency as a multiple of processing block length.
    /// 2 is the minimum that may work. Higher values may give some more
    /// margin for scheduling jitter and such. 3 is often a good value.
    pub latency_blocks: usize,
    /// Sample rate
    pub fs:       f64,
    /// Receive center frequency
    pub rx_freq:  f64,
    /// Transmit center frequency
    pub tx_freq:  f64,
    /// Receive channel number
    pub rx_chan:  usize,
    /// Transmit channel number
    pub tx_chan:  usize,
    /// Receive antenna
    pub rx_ant:   &'a str,
    /// Transmit antenna
    pub tx_ant:   &'a str,
    /// Receive gain
    pub rx_gain:  f64,
    /// Transmit gain
    pub tx_gain:  f64,
    /// Device arguments
    pub dev_args: &'a [(&'a str, &'a str)],
    /// Receive stream arguments
    pub rx_args:  &'a [(&'a str, &'a str)],
    /// Transmit stream arguments
    pub tx_args:  &'a [(&'a str, &'a str)],
}

pub struct SoapyIo {
    dev: soapysdr::Device,
    rx:  soapysdr::RxStream<StreamType>,
    tx:  soapysdr::TxStream<StreamType>,
    buf: Vec<StreamType>,
    /// RX-TX timestamp difference
    latency_time: i64,
}

/// Convert a slice of ("key", "value") pairs to soapysdr::Args.
/// This might not be really needed, but it makes configuration struct
/// contents easier to write, and I could not figure out a better
/// way to do it.
fn convert_args(key_value_pairs: &[(&str, &str)]) -> soapysdr::Args {
    let mut args = soapysdr::Args::new();
    for (key, value) in key_value_pairs {
        args.set(*key, *value);
    }
    args
}

/// It is annoying to repeat error handling so do that in a macro.
/// ? could be used but then it could not print which SoapySDR call failed.
macro_rules! soapycheck {
    ($text:literal, $soapysdr_call:expr) => {
        match $soapysdr_call {
            Ok(ret) => { ret },
            Err(err) => {
                eprintln!("SoapySDR: Failed to {}: {}", $text, err);
                return Err(err);
            }
        }
    }
}

fn soapysdr_setup(conf: &SoapyIoConfig) -> Result<SoapyIo, soapysdr::Error> {
    let dev = soapycheck!("open SoapySDR device",
        soapysdr::Device::new(convert_args(conf.dev_args)));
    soapycheck!("set RX sample rate",
        dev.set_sample_rate(soapysdr::Direction::Rx, conf.rx_chan, conf.fs));
    soapycheck!("set TX sample rate",
        dev.set_sample_rate(soapysdr::Direction::Tx, conf.tx_chan, conf.fs));
    soapycheck!("set RX center frequency",
        dev.set_frequency(soapysdr::Direction::Rx, conf.rx_chan, conf.rx_freq, soapysdr::Args::new()));
    soapycheck!("set TX center frequency",
        dev.set_frequency(soapysdr::Direction::Tx, conf.tx_chan, conf.tx_freq, soapysdr::Args::new()));
    soapycheck!("set RX antenna",
        dev.set_antenna(soapysdr::Direction::Rx, conf.rx_chan, conf.rx_ant));
    soapycheck!("set TX antenna",
        dev.set_antenna(soapysdr::Direction::Tx, conf.tx_chan, conf.tx_ant));
    // TODO: support setting gain elements
    soapycheck!("set RX gain",
        dev.set_gain(soapysdr::Direction::Rx, conf.rx_chan, conf.rx_gain));
    soapycheck!("set TX gain",
        dev.set_gain(soapysdr::Direction::Tx, conf.tx_chan, conf.tx_gain));
    let mut rx = soapycheck!("setup RX stream",
        dev.rx_stream_args(&[conf.rx_chan], convert_args(conf.rx_args)));
    let mut tx = soapycheck!("setup TX stream",
        dev.tx_stream_args(&[conf.tx_chan], convert_args(conf.tx_args)));
    soapycheck!("activate RX stream",
        rx.activate(None));
    soapycheck!("activate TX stream",
        tx.activate(None));
    Ok(SoapyIo {
        dev: dev,
        rx:  rx,
        tx:  tx,
        buf: vec![num::zero(); conf.blocklen],
        latency_time: ((conf.blocklen * conf.latency_blocks) as f64 * 1e9 / conf.fs).round() as i64,
    })
}

impl SoapyIo {
    pub fn new(conf: &SoapyIoConfig) -> Option<Self> {
        match soapysdr_setup(&conf) {
            Ok(soapyio) => Some(soapyio),
            Err(err) => {
                eprintln!("SoapySDR initialization failed: {}", err);
                None
            }
        }
    }

    /// Returns Some(()) on success, None on error.
    /// Maybe some proper error type would be better.
    pub fn process<F>(&mut self, mut process_signal: F) -> Option<()>
        where F: FnMut(&mut [Complex<f32>], i64, i64)
    {
        match self.rx.read_ext(&mut [&mut self.buf[..]], soapysdr::StreamFlags::default(), None, 100000) {
            Ok(result) => {
                if result.len != self.buf.len() {
                    eprintln!("Warning: expected {} samples, read {}", self.buf.len(), result.len);
                }
                let buf_slice = &mut self.buf[0..result.len];
                if let Some(time) = result.time {
                    let tx_time = time + self.latency_time;
                    process_signal(buf_slice, time, tx_time);
                    match self.tx.write_all(&[buf_slice], Some(tx_time), false, 100000) {
                        Ok(_) => Some(()),
                        Err(err) => {
                            eprintln!("Stream write error: {}", err);
                            if error_is_recoverable(&err) { Some(()) } else { None }
                        }
                    }
                } else {
                    eprintln!("Radios without timestamp support are not currently supported.");
                    None
                }
            },
            Err(err) => {
                eprintln!("Stream read error: {}", err);
                if error_is_recoverable(&err) { Some(()) } else { None }
            }
        }
    }
}

fn error_is_recoverable(err: &soapysdr::Error) -> bool {
    match err.code {
        // These errors could be caused by a momentary scheduling latency,
        // lost packet on network or bus or something like that.
        // SDR might recover from those so just "ignore" the error
        // and continue reading the stream on next call.
        soapysdr::ErrorCode::Timeout |
        soapysdr::ErrorCode::Corruption |
        soapysdr::ErrorCode::Overflow |
        soapysdr::ErrorCode::TimeError |
        soapysdr::ErrorCode::Underflow => true,
        // Others might signal a bigger problem, such as disconnected or
        // broken SDR device, so return an error for these
        // and make the program stop.
        _ => false,
    }
}
