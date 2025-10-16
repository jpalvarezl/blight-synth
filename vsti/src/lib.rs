use nih_plug::prelude::*;

mod hi_hat;
mod monophonic_oscillator;

nih_export_vst3!(hi_hat::HiHat);
