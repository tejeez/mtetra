use num::Complex;
pub mod file;
pub mod soapy;

pub enum RadioIoConfig<'a> {
    File(&'a file::FileIoConfig<'a>),
    Soapy(&'a soapy::SoapyIoConfig<'a>),
}

enum RadioIoEnum {
    File(file::FileIo),
    Soapy(soapy::SoapyIo),
}

pub struct RadioIo(RadioIoEnum);

impl RadioIo {
    pub fn new(conf: &RadioIoConfig) -> Option<Self> {
        Some(RadioIo(match conf {
            RadioIoConfig::File(conf) =>
                RadioIoEnum::File(file::FileIo::new(conf)?),
            RadioIoConfig::Soapy(conf) =>
                RadioIoEnum::Soapy(soapy::SoapyIo::new(conf)?),
        }))
    }

    pub fn process<F>(&mut self, mut process_signal: F) -> Option<()>
        where F: FnMut(&mut [Complex<f32>], i64, i64)
    {
        match self.0 {
            RadioIoEnum::File(ref mut io) => io.process(&mut process_signal),
            RadioIoEnum::Soapy(ref mut io) => io.process(&mut process_signal),
        }
    }
}
