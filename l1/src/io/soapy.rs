use num::Complex;
use soapysdr;

type StreamType = Complex<f32>;

pub struct SoapyIoConfig<'a> {
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
    let rx = soapycheck!("setup RX stream",
        dev.rx_stream_args(&[conf.rx_chan], convert_args(conf.rx_args)));
    let tx = soapycheck!("setup TX stream",
        dev.tx_stream_args(&[conf.tx_chan], convert_args(conf.tx_args)));
    Ok(SoapyIo {
        dev: dev,
        rx:  rx,
        tx:  tx,
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
    pub fn process<F>(&mut self, cb: F) -> Option<()>
        where F: FnMut(i64, &mut [Complex<f32>])
    {
        // TODO
        Some(())
    }
}
