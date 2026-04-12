use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, MutexGuard},
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

pub struct AudioRenderer {
    _stream: cpal::Stream,
    resampler: Arc<Mutex<Resampler>>,
}

impl Default for AudioRenderer {
    fn default() -> Self {
        // TODO better error handling, don't panic, output nothing

        log::debug!("Initializing audio renderer...");

        let host = cpal::default_host();

        let device = host
            .default_output_device()
            .expect("No output device found");

        log::debug!("Found output device: {:?}", device.description());

        // TODO cleanup

        let preferred_rates = [48000u32, 44100, 32000, 22050];

        let configs = device
            .supported_output_configs()
            .expect("Found no supported configs")
            .collect::<Vec<_>>();

        let mut chosen = None;

        'outer: for &rate in &preferred_rates {
            for cfg in &configs {
                if cfg.channels() == 2
                    && cfg.sample_format() == cpal::SampleFormat::F32
                    && cfg.min_sample_rate() <= rate
                    && cfg.max_sample_rate() >= rate
                {
                    chosen = Some(cfg.with_sample_rate(rate));
                    break 'outer;
                }
            }
        }

        let config = chosen.unwrap();

        log::debug!("Selected config: {:?}", config);

        let resampler = Arc::new(Mutex::new(Resampler::new(
            config.sample_rate(),
            config.channels() as usize,
        )));

        let stream_resampler = resampler.clone();

        let stream = device
            .build_output_stream(
                &config.into(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    // TODO single channel possible? mix?

                    let mut resampler = stream_resampler.lock().unwrap();

                    resampler.fill(data);
                },
                move |err| {
                    log::error!("Audio stream error: {:?}", err);
                },
                None,
            )
            .expect("Failed to build audio stream");

        stream.play().expect("Failed to play audio stream");

        Self {
            _stream: stream,
            resampler,
        }
    }
}

impl AudioRenderer {
    fn lock_resampler(&'_ self) -> MutexGuard<'_, Resampler> {
        self.resampler.lock().expect("Failed to lock audio buffer")
    }

    pub fn push(&mut self, samples: &[u8]) {
        self.lock_resampler().push(samples);
    }

    pub fn queued_samples(&self) -> usize {
        self.lock_resampler().len()
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.lock_resampler().set_input_rate(sample_rate);
    }
}

struct Resampler {
    input_rate: u32,
    output_rate: u32,
    ratio: f64,
    output_channels: usize,

    phase: f64,
    previous_sample: (f32, f32),

    // Input buffer, raw bytes, 16-bit stereo = 4 bytes per sample
    input_buffer: VecDeque<u8>,

    // Converted buffer, before resampling
    converted_buffer: VecDeque<(f32, f32)>,
}

impl Resampler {
    pub fn new(output_rate: u32, output_channels: usize) -> Self {
        Self {
            input_rate: output_rate,
            output_rate,
            ratio: 1.0,
            output_channels,
            phase: 0.0,
            previous_sample: (0.0, 0.0),
            input_buffer: VecDeque::new(),
            converted_buffer: VecDeque::new(),
        }
    }

    pub fn set_input_rate(&mut self, input_rate: u32) {
        if input_rate != self.input_rate {
            log::info!(
                "Resampler input rate changed to {} (output = {})",
                input_rate,
                self.output_rate
            );
        }

        self.input_rate = input_rate;
        self.ratio = input_rate as f64 / self.output_rate as f64;
    }

    // TODO move u8?
    pub fn push(&mut self, samples: &[u8]) {
        self.input_buffer.extend(samples.iter().copied());
    }

    pub fn len(&self) -> usize {
        self.input_buffer.len()
    }

    pub fn fill(&mut self, output: &mut [f32]) {
        // Convert all the available input samples

        while self.input_buffer.len() >= 4 {
            let l_hi = self.input_buffer.pop_front().unwrap();
            let l_lo = self.input_buffer.pop_front().unwrap();
            let l = ((l_hi as u16) << 8 | l_lo as u16) as i16 as f32 / 32768.0;

            let r_hi = self.input_buffer.pop_front().unwrap();
            let r_lo = self.input_buffer.pop_front().unwrap();
            let r = ((r_hi as u16) << 8 | r_lo as u16) as i16 as f32 / 32768.0;

            self.converted_buffer.push_back((l, r));
        }

        //log::debug!("Pushing {:?} samples", self.converted_buffer);

        // log::debug!("Resampling {} frames", frames);

        // Resample some of the input samples into the output buffer

        let mut phaseTEMP = 0.0f64;

        let frames = output.len() / self.output_channels;

        for frame in 0..frames {
            let sample_index = phaseTEMP as usize;
            let sample_frac = phaseTEMP.fract();

            let sample1 = self
                .converted_buffer
                .get(sample_index)
                .copied()
                .unwrap_or((0.0, 0.0)); //self.previous_sample);

            let sample2 = self
                .converted_buffer
                .get(sample_index + 1)
                .copied()
                .unwrap_or(sample1);

            let left = sample1.0 + (sample2.0 - sample1.0) * (sample_frac as f32);

            output[frame * self.output_channels] = left;

            if self.output_channels > 1 {
                let right = sample1.1 + (sample2.1 - sample1.1) * (sample_frac as f32);

                output[frame * self.output_channels + 1] = right;

                // if left > 0.0 || right > 0.0 {
                //     log::debug!(
                //         "Resampled {}, {}, {}, {}/{}",
                //         frame,
                //         sample_index,
                //         sample_frac,
                //         left,
                //         right
                //     );
                // }
            }

            phaseTEMP += self.ratio;
        }

        // Drain the consumed input samples

        let consumed = (phaseTEMP as usize).min(self.converted_buffer.len());

        if consumed > 0 {
            self.previous_sample = self.converted_buffer[consumed - 1];
            self.converted_buffer.drain(0..consumed);
            self.phase -= consumed as f64;
        }
    }
}
