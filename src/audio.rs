use std::{f64::consts, sync::{mpsc, Arc, atomic::{AtomicBool, self}}};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleRate;

#[derive(Debug, Clone)]
pub struct AudioWorldState {
    pub fs: [f64; 3],
    pub dc: f64,
}

pub fn run_audio(audio_state_channel: mpsc::Receiver<AudioWorldState>, exit: Arc<AtomicBool>) {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("no default host audio device!");
    let mut conf_ranges = device
        .supported_output_configs()
        .expect("could not query audio device capabilities -- audio device disconnected?");
    let conf_range = conf_ranges
        .next()
        .expect("audio device has no configurations!");
    let desired_sample_rate = conf_range.max_sample_rate().0;
    //let desired_sample_rate = u32::clamp(44100, conf_range.min_sample_rate().0, conf_range.max_sample_rate().0);
    let conf = conf_range
        .with_sample_rate(SampleRate(desired_sample_rate))
        .config();

    let sr = conf.sample_rate.0 as f64;
    #[allow(non_snake_case)]
    let Δt = sr.recip();
    let mut paddle1 = SawGen {
        current_phase: 0.0,
        freq: 0.0,
        midpoint: 0.8,
    };
    let mut paddle2 = SinGen {
        current_phase: 0.0,
        freq: 0.0,
    };
    let mut ball = SqGen {
        current_phase: 0.0,
        freq: 0.0,
        dc: 0.5,
    };

    let mut state = AudioWorldState {
        fs: [0.0; 3],
        dc: 0.5,
    };

    let mut gen_samples;
    {
        let exit = exit.clone();
        gen_samples = move |data: &mut [f32]| {
            if let Some(newstate) = audio_state_channel.try_iter().last() {
                state = newstate;
            }
            // TODO fix the error where state changes can be dropped. also see the similar case in main.rs.
            if let Err(mpsc::TryRecvError::Disconnected) = audio_state_channel.try_recv() {
                exit.store(true, atomic::Ordering::Relaxed);
            }
            println!("audio state: {state:?}");

            // update generators from state
            ball.freq = state.fs[0];
            paddle1.freq = state.fs[1];
            paddle2.freq = state.fs[2];
            ball.dc = state.dc;
            for i in 0..data.len() {
                let mut x = 0.0;
                x += paddle1.step(Δt);
                x += paddle2.step(Δt) * 0.1;
                x += ball.step(Δt) * 0.6;
                x *= 0.3;
                data[i] = x as f32;
            }
        };
    }

    let stream = device
        .build_output_stream(
            &conf,
            move |buf, _: &cpal::OutputCallbackInfo| gen_samples(buf),
            move |err| {
                println!("audio stream error: {}", err);
            },
            None,
        )
        .expect("could not create audio stream");

    stream.play().expect("could not play output stream!");

    while !exit.load(atomic::Ordering::Relaxed) {};
}

trait Gen {
    #[allow(non_snake_case)]
    fn step(&mut self, Δt: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct SqGen {
    pub current_phase: f64,
    pub freq: f64,
    pub dc: f64,
}

impl Gen for SqGen {
    #[allow(non_snake_case)]
    fn step(&mut self, Δt: f64) -> f64 {
        self.current_phase += self.freq * Δt;
        if self.current_phase > 1.0 {
            self.current_phase -= 1.0;
        }
        if self.current_phase < self.dc {
            1.0
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone)]
pub struct SinGen {
    pub current_phase: f64,
    pub freq: f64,
}

impl Gen for SinGen {
    #[allow(non_snake_case)]
    fn step(&mut self, Δt: f64) -> f64 {
        self.current_phase += self.freq * Δt;
        if self.current_phase > 1.0 {
            self.current_phase -= 1.0;
        }
        (self.current_phase * consts::TAU).sin()
    }
}

#[derive(Debug, Clone)]
pub struct SawGen {
    pub current_phase: f64,
    pub freq: f64,
    pub midpoint: f64,
}

impl Gen for SawGen {
    #[allow(non_snake_case)]
    fn step(&mut self, Δt: f64) -> f64 {
        self.current_phase += self.freq * Δt;
        if self.current_phase > 1.0 {
            self.current_phase -= 1.0;
        }
        if self.current_phase < self.midpoint {
            self.current_phase / self.midpoint
        } else {
            (1.0 - self.current_phase) / (1.0 - self.midpoint)
        }
    }
}
