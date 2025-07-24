// A slightly modified version of https://github.com/irh/freeverb-rs
// License:
// Copyright (c) 2018 Ian Hobson

// Permission is hereby granted, free of charge, to any person obtaining a copy of
// this software and associated documentation files (the "Software"), to deal in
// the Software without restriction, including without limitation the rights to
// use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
// of the Software, and to permit persons to whom the Software is furnished to do
// so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
use pure_audio::{parameter, Event, State, StereoEffectData};

#[parameter(min = 0.0, max = 1.0, default = 0.5)]
struct Dampening(f32);

#[parameter(min = 0.0, max = 1.0, default = 0.0)]
struct Dry(f32);

#[parameter(default = false)]
struct Freeze(bool);

#[parameter(min = 0.0, max = 1.0, default = 0.5)]
struct RoomSize(f32);

#[parameter(min = 0.0, max = 1.0, default = 1.0)]
struct Wet(f32);

#[parameter(min = 0.0, max = 1.0, default = 0.5)]
struct Width(f32);

struct DelayLine {
    base_length: usize,
    buffer: Vec<f32>,
    index: usize,
}

impl DelayLine {
    fn new(base_length: usize) -> Self {
        Self {
            base_length,
            buffer: vec![0.0; base_length],
            index: 0,
        }
    }

    fn read(&self) -> f32 {
        self.buffer[self.index]
    }

    fn write_and_advance(&mut self, value: f32) {
        self.buffer[self.index] = value;
        self.index = if self.index == self.buffer.len() - 1 {
            0
        } else {
            self.index + 1
        };
    }

    fn resize(&mut self, sample_rate: f64) {
        let new_length = (self.base_length as f64 * sample_rate / 44100.0) as usize;
        self.buffer.resize(new_length, 0.0);
    }
}

struct Comb {
    delay_line: DelayLine,
    feedback: f32,
    filter_state: f32,
    dampening: f32,
    dampening_inverse: f32,
}

impl Comb {
    fn new(base_delay_length: usize) -> Self {
        Self {
            delay_line: DelayLine::new(base_delay_length),
            feedback: 0.5,
            filter_state: 0.0,
            dampening: 0.5,
            dampening_inverse: 0.5,
        }
    }

    fn set_dampening(&mut self, value: f32) {
        self.dampening = value;
        self.dampening_inverse = 1.0 - value;
    }

    fn set_feedback(&mut self, value: f32) {
        self.feedback = value;
    }

    fn resize(&mut self, sample_rate: f64) {
        self.delay_line.resize(sample_rate);
    }

    fn tick(&mut self, input: f32) -> f32 {
        let output = self.delay_line.read();
        self.filter_state = output * self.dampening_inverse + self.filter_state * self.dampening;
        self.delay_line
            .write_and_advance(input + self.filter_state * self.feedback);

        output
    }
}

const ALLPASS_FEEDBACK: f32 = 0.5;
struct AllPass {
    delay_line: DelayLine,
}

impl AllPass {
    fn new(base_delay_length: usize) -> Self {
        Self {
            delay_line: DelayLine::new(base_delay_length),
        }
    }

    fn resize(&mut self, sample_rate: f64) {
        self.delay_line.resize(sample_rate);
    }

    fn tick(&mut self, input: f32) -> f32 {
        let delayed = self.delay_line.read();
        let output = -input + delayed;
        self.delay_line
            .write_and_advance(input + delayed * ALLPASS_FEEDBACK);

        output
    }
}

const FIXED_GAIN: f32 = 0.015;

const SCALE_WET: f32 = 3.0;
const SCALE_DAMPENING: f32 = 0.4;

const SCALE_ROOM: f32 = 0.28;
const OFFSET_ROOM: f32 = 0.7;

const STEREO_SPREAD: usize = 23;

const COMB_TUNING_L1: usize = 1116;
const COMB_TUNING_R1: usize = COMB_TUNING_L1 + STEREO_SPREAD;
const COMB_TUNING_L2: usize = 1188;
const COMB_TUNING_R2: usize = COMB_TUNING_L2 + STEREO_SPREAD;
const COMB_TUNING_L3: usize = 1277;
const COMB_TUNING_R3: usize = COMB_TUNING_L3 + STEREO_SPREAD;
const COMB_TUNING_L4: usize = 1356;
const COMB_TUNING_R4: usize = COMB_TUNING_L4 + STEREO_SPREAD;
const COMB_TUNING_L5: usize = 1422;
const COMB_TUNING_R5: usize = COMB_TUNING_L5 + STEREO_SPREAD;
const COMB_TUNING_L6: usize = 1491;
const COMB_TUNING_R6: usize = COMB_TUNING_L6 + STEREO_SPREAD;
const COMB_TUNING_L7: usize = 1557;
const COMB_TUNING_R7: usize = COMB_TUNING_L7 + STEREO_SPREAD;
const COMB_TUNING_L8: usize = 1617;
const COMB_TUNING_R8: usize = COMB_TUNING_L8 + STEREO_SPREAD;

const ALLPASS_TUNING_L1: usize = 556;
const ALLPASS_TUNING_R1: usize = ALLPASS_TUNING_L1 + STEREO_SPREAD;
const ALLPASS_TUNING_L2: usize = 441;
const ALLPASS_TUNING_R2: usize = ALLPASS_TUNING_L2 + STEREO_SPREAD;
const ALLPASS_TUNING_L3: usize = 341;
const ALLPASS_TUNING_R3: usize = ALLPASS_TUNING_L3 + STEREO_SPREAD;
const ALLPASS_TUNING_L4: usize = 225;
const ALLPASS_TUNING_R4: usize = ALLPASS_TUNING_L4 + STEREO_SPREAD;

struct Freeverb {
    combs: [(Comb, Comb); 8],
    allpasses: [(AllPass, AllPass); 4],
    wet_gains: (f32, f32),
    wet: f32,
    width: f32,
    dry: f32,
    input_gain: f32,
    dampening: f32,
    room_size: f32,
    frozen: bool,
}

impl Default for Freeverb {
    fn default() -> Self {
        let mut freeverb = Self {
            combs: [
                (
                    Comb::new(COMB_TUNING_L1),
                    Comb::new(COMB_TUNING_R1),
                ),
                (
                    Comb::new(COMB_TUNING_L2),
                    Comb::new(COMB_TUNING_R2),
                ),
                (
                    Comb::new(COMB_TUNING_L3),
                    Comb::new(COMB_TUNING_R3),
                ),
                (
                    Comb::new(COMB_TUNING_L4),
                    Comb::new(COMB_TUNING_R4),
                ),
                (
                    Comb::new(COMB_TUNING_L5),
                    Comb::new(COMB_TUNING_R5),
                ),
                (
                    Comb::new(COMB_TUNING_L6),
                    Comb::new(COMB_TUNING_R6),
                ),
                (
                    Comb::new(COMB_TUNING_L7),
                    Comb::new(COMB_TUNING_R7),
                ),
                (
                    Comb::new(COMB_TUNING_L8),
                    Comb::new(COMB_TUNING_R8),
                )
            ],
            allpasses: [
                (
                    AllPass::new(ALLPASS_TUNING_L1),
                    AllPass::new(ALLPASS_TUNING_R1),
                ),
                (
                    AllPass::new(ALLPASS_TUNING_L2),
                    AllPass::new(ALLPASS_TUNING_R2),
                ),
                (
                    AllPass::new(ALLPASS_TUNING_L3),
                    AllPass::new(ALLPASS_TUNING_R3),
                ),
                (
                    AllPass::new(ALLPASS_TUNING_L4),
                    AllPass::new(ALLPASS_TUNING_R4),
                )
            ],
            wet_gains: (0.0, 0.0),
            wet: 0.0,
            dry: 0.0,
            input_gain: 1.0,
            width: 0.0,
            dampening: 0.0,
            room_size: 0.0,
            frozen: false,
        };

        freeverb.set_wet(1.0);
        freeverb.set_width(0.5);
        freeverb.set_dampening(0.5);
        freeverb.set_room_size(0.5);

        freeverb
    }
}

impl State for Freeverb {
    fn activate(&mut self, sample_rate: f64, _min_frame_count: usize, _max_frame_count: usize) {
        self.resize(sample_rate);
    }
}

impl Freeverb {
    fn set_dampening(&mut self, value: f32) {
        self.dampening = value * SCALE_DAMPENING;
        self.update_combs();
    }

    fn set_dry(&mut self, value: f32) {
        self.dry = value;
    }

    fn set_freeze(&mut self, value: bool) {
        self.frozen = value;
        self.update_combs();
    }

    fn set_room_size(&mut self, value: f32) {
        self.room_size = value * SCALE_ROOM + OFFSET_ROOM;
        self.update_combs();
    }

    fn set_wet(&mut self, value: f32) {
        self.wet = value * SCALE_WET;
        self.update_wet_gains();
    }

    fn set_width(&mut self, value: f32) {
        self.width = value;
        self.update_wet_gains();
    }

    fn update_combs(&mut self) {
        let (feedback, dampening) = if self.frozen {
            (1.0, 0.0)
        } else {
            (self.room_size, self.dampening)
        };

        for (comb_l, comb_r) in self.combs.iter_mut() {
            comb_l.set_feedback(feedback);
            comb_r.set_feedback(feedback);

            comb_l.set_dampening(dampening);
            comb_r.set_dampening(dampening);
        }
    }

    fn update_wet_gains(&mut self) {
        self.wet_gains = (
            self.wet * (self.width / 2.0 + 0.5),
            self.wet * ((1.0 - self.width) / 2.0),
        );
    }

    fn resize(&mut self, sample_rate: f64) {
        for (comb_l, comb_r) in self.combs.iter_mut() {
            comb_l.resize(sample_rate);
            comb_r.resize(sample_rate);
        }

        for (allpass_l, allpass_r) in self.allpasses.iter_mut() {
            allpass_l.resize(sample_rate);
            allpass_r.resize(sample_rate);
        }
    }

    fn tick(&mut self, input_l: &f32, input_r: &f32) -> [f32; 2] {
        let input_mixed = (input_l + input_r) * FIXED_GAIN * self.input_gain;

        let [mut out_l, mut out_r] = [0.0; 2];

        for (comb_l, comb_r) in self.combs.iter_mut() {
            out_l += comb_l.tick(input_mixed);
            out_r += comb_r.tick(input_mixed);
        }

        for (allpass_l, allpass_r) in self.allpasses.iter_mut() {
            out_l = allpass_l.tick(out_l);
            out_r = allpass_r.tick(out_r);
        }

        let (wet_gain_l, wet_gain_r) = self.wet_gains;

        [
            out_l * wet_gain_l + out_r * wet_gain_r + input_l * self.dry,
            out_r * wet_gain_l + out_l * wet_gain_r + input_r * self.dry
        ]
    }
}

fn process(
    StereoEffectData {
        inputs: [input_l, input_r],
        outputs: [output_l, output_r],
        events,
        state,
        ..
    }: StereoEffectData<Freeverb>,
    Dampening(dampening): Dampening,
    Dry(dry): Dry,
    Freeze(freeze): Freeze,
    RoomSize(room_size): RoomSize,
    Wet(wet): Wet,
    Width(width): Width,
) {
    for event in events {
        if let Event::ParamsChanged = event {
            state.set_dampening(dampening);
            state.set_dry(dry);
            state.set_freeze(freeze);
            state.set_room_size(room_size);
            state.set_wet(wet);
            state.set_width(width);
        }
    }

    for (((input_sample_l, output_sample_l), input_sample_r), output_sample_r) in input_l.iter().zip(output_l).zip(input_r).zip(output_r) {
        [*output_sample_l, *output_sample_r] = state.tick(input_sample_l, input_sample_r);
    }
}

#[cfg(target_arch = "wasm32")]
pure_audio_wasm::pure_audio_wasm_entry!("Freeverb", process);

#[cfg(not(target_arch = "wasm32"))]
pure_audio_clap::pure_audio_clap_entry!("pureaudio.Freeverb", "PureAudioFreeverb", process);