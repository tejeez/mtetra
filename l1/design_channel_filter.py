#!/usr/bin/env python3
import math
import numpy as np

def rrc(f, fs, excess_bandwidth = 0.35, samples_per_symbol = 4):
    """Frequency response of a root raised cosine filter.

    Frequency f is given so that f=1.0 at symbol rate."""

    f = abs(f) * samples_per_symbol / fs

    transition_band_start = 0.5 - excess_bandwidth * 0.5
    transition_band_end   = 0.5 + excess_bandwidth * 0.5

    if f < transition_band_start:
        return 1.0
    elif f < transition_band_end:
        return math.sin(math.pi * 0.5 * (transition_band_end - f) / excess_bandwidth)
    else:
        return 0.0

def sinc(v):
    if abs(v) == 0.0:
        return 1.0
    else:
        return math.sin(v) / v

def ciccomp(f, fs, cic_stages = 5):
    """Frequency response of a CIC compensation filter.
    The exact response would depend on CIC resampling factor but
    an approximation for high resampling factors is used here.
    See https://cdrdv2-public.intel.com/653906/an455.pdf page 5.
    """
    return sinc(math.pi * f / fs) ** (-cic_stages)

def combined(f, fs):
    """Frequency response of a combined pulse shaping
    and CIC compensation filter."""
    return ciccomp(f, fs) * rrc(f, fs)

def design_fir(response, fs = 1.0, ntaps = 25, fftsize = 256):
    """Compute FIR coefficients as a Fourier transform of frequency response."""
    # Frequency response bins
    r = np.zeros(fftsize)
    for i in range(0, fftsize):
        r[i] = response(fs / fftsize * (i - fftsize/2), fs)
    # Transform
    t = np.fft.ifftshift(np.fft.ifft(np.fft.fftshift(r)))
    # Truncate to number of taps.
    # A window function could be added here if needed.
    return t[fftsize//2 - ntaps//2 : fftsize//2 + (ntaps+1)//2]

def main():
    taps = design_fir(combined)
    print(len(taps))
    print(',\n'.join('%11.8f' % t.real for t in taps))

if __name__ == '__main__':
    main()
