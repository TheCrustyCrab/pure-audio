use std::f32::consts::PI;
use crate::{AEGMode, AmplitudeAttack, AmplitudeEnvelopeIsGate, AmplitudeRelease, Cutoff, FilterMode, OscillatorDetune, PreFilterVCA, Resonance, UnisonCount, UnisonSpread, MAX_UNISON};

#[derive(Clone, Copy)]
pub struct Voice {
    pub(crate) port_id: i32,
    pub(crate) channel: i32,
    pub(crate) key: i32,
    pub(crate) note_id: i32,
    unison_count: UnisonCount,
    unison_spread: UnisonSpread,
    unison_spread_mod: f32,
    oscillator_detune: OscillatorDetune,
    oscillator_detune_mod: f32,
    filter_mode: FilterMode,
    cutoff: Cutoff,
    cutoff_mod: f32,
    resonance: Resonance,
    resonance_mod: f32,
    amplitude_envelope_is_gate: AmplitudeEnvelopeIsGate,
    amplitude_attack_seconds: f32,
    amplitude_release_seconds: f32,
    prefilter_vca: PreFilterVCA,
    prefilter_vca_mod: f32,
    volume_note_expression_value: f32,
    pitch_note_expression_value: f32,
    pitch_bend_wheel: f32,
    pub(crate) sample_rate: f32,
    pub(crate) aeg_mode: AEGMode,
    pub(crate) out_l: f32,
    pub(crate) out_r: f32,
    filter: StereoSimperSVF,
    base_freq: f32,
    sr_inv: f32,
    time: f32,
    release_from: f32,
    pan_l: [f32; MAX_UNISON],
    pan_r: [f32; MAX_UNISON],
    unit_shift: [f32; MAX_UNISON],
    norm: [f32; MAX_UNISON],
    phase: [f64; MAX_UNISON],
    d_phase: [f64; MAX_UNISON], // phase increment
    d_phase_inv: [f64; MAX_UNISON],
}

impl Default for Voice {
    fn default() -> Self {
        Self {
            aeg_mode: AEGMode::Off,
            amplitude_attack_seconds: 0.01,
            amplitude_envelope_is_gate: AmplitudeEnvelopeIsGate(false),
            amplitude_release_seconds: 0.2,
            channel: 0,
            cutoff: Cutoff(69.0),
            cutoff_mod: 0.0,
            filter_mode: FilterMode::LowPass,
            key: 0,
            note_id: 0,
            oscillator_detune: OscillatorDetune(0.0),
            oscillator_detune_mod: 0.0,
            out_l: 0.0,
            out_r: 0.0,
            filter: Default::default(),
            pitch_bend_wheel: 0.0,
            pitch_note_expression_value: 0.0,
            port_id: 0,
            prefilter_vca: PreFilterVCA(1.0),
            prefilter_vca_mod: 0.0,
            resonance: Resonance(0.7),
            resonance_mod: 0.0,
            sample_rate: 0.0,
            unison_count: UnisonCount(3),
            unison_spread: UnisonSpread(10.0),
            unison_spread_mod: 0.0,
            volume_note_expression_value: 0.0,
            base_freq: 440.0,
            sr_inv: 0.0,
            time: 0.0,
            release_from: 1.0,
            pan_l: Default::default(),
            pan_r: Default::default(),
            unit_shift: Default::default(),
            norm: Default::default(),
            phase: Default::default(),
            d_phase: Default::default(),
            d_phase_inv: Default::default(),
        }
    }
}

impl Voice {
    pub fn activate(
        &mut self,
        port_index: i32,
        channel: i32,
        key: i32,
        note_id: i32,
        filter_mode: FilterMode,
        unison_count: UnisonCount,
        unison_spread: UnisonSpread,
        oscillator_detune: OscillatorDetune,
        cutoff: Cutoff,
        resonance: Resonance,
        prefilter_vca: PreFilterVCA,
        amplitude_attack: AmplitudeAttack,
        amplitude_release: AmplitudeRelease,
        amplitude_envelope_is_gate: AmplitudeEnvelopeIsGate,
    ) {
        self.unison_count = unison_count;
        self.filter_mode = filter_mode;
        self.port_id = port_index;
        self.channel = channel;
        self.note_id = note_id;

        self.unison_spread = unison_spread;
        self.oscillator_detune = oscillator_detune;
        self.cutoff = cutoff;
        self.resonance = resonance;
        self.prefilter_vca = prefilter_vca;
        self.amplitude_release_seconds = scale_time_param_to_seconds(amplitude_release.0);
        self.amplitude_attack_seconds = scale_time_param_to_seconds(amplitude_attack.0);
        self.amplitude_envelope_is_gate = amplitude_envelope_is_gate;

        self.cutoff_mod = 0.0;
        self.oscillator_detune_mod = 0.0;
        self.resonance_mod = 0.0;
        self.prefilter_vca_mod = 0.0;
        self.unison_spread_mod = 0.0;
        self.volume_note_expression_value = 0.0;
        self.pitch_note_expression_value = 0.0;

        self.start(key);
    }

    pub fn start(&mut self, key: i32) {
        self.sr_inv = 1.0 / self.sample_rate;

        self.filter.init();
        self.key = key;
        self.aeg_mode = if self.amplitude_attack_seconds > 0.0 {
            AEGMode::Attack
        } else {
            AEGMode::Hold
        };
        self.time = 0.0;

        if self.unison_count.0 == 1 {
            self.unit_shift[0] = 0.0;
            self.pan_l[0] = 1.0;
            self.pan_r[0] = 1.0;
            self.phase[0] = 0.0;
            self.norm[0] = 1.0;
        } else {
            for i in 0..self.unison_count.0 as usize {
                let di = 1.0 * i as f32 / (self.unison_count.0 as f32 - 1.0);
                self.unit_shift[i] = 2.0 * di - 1.0;
                self.phase[i] = di as f64;
                self.pan_l[i] = (0.5 * PI * di).cos();
                self.pan_r[i] = (0.5 * PI * di).sin();

                self.norm[i] = 1.0 / (self.unison_count.0 as f32).sqrt();
            }
        }

        self.recalc_pitch();
        self.recalc_filter();
    }

    pub fn step(&mut self) {
        let mut ar = 1.0;

        match self.aeg_mode {
            AEGMode::Attack => {
                ar = self.time / self.amplitude_attack_seconds;
                self.release_from = ar;
                self.time += self.sr_inv;
                if self.time >= self.amplitude_attack_seconds {
                    self.aeg_mode = AEGMode::Hold;
                }

                if let AmplitudeEnvelopeIsGate(true) = self.amplitude_envelope_is_gate {
                    ar = 1.0;
                }
            }
            AEGMode::Releasing => {
                let tn = self.time / self.amplitude_release_seconds;
                let tf = 1.0 - tn;
                ar = self.release_from * tf;
                self.time += self.sr_inv;
                if self.time >= self.amplitude_release_seconds {
                    self.aeg_mode = AEGMode::NewlyOff;
                }

                if let AmplitudeEnvelopeIsGate(true) = self.amplitude_envelope_is_gate {
                    ar = 1.0;
                    const LAST_SEG: f32 = 0.02;
                    if tn > 1.0 - LAST_SEG {
                        // avoid a click with a last 2% fade
                        ar = 1.0 - (tn - (1.0 - LAST_SEG)) / LAST_SEG;
                    }
                }
            }
            AEGMode::Hold => {
                ar = 1.0;
                self.time = 0.0;
                self.release_from = 1.0;
            }
            _ => {}
        }

        ar *= self.prefilter_vca + self.prefilter_vca_mod + self.volume_note_expression_value;
        self.out_l = 0.0;
        self.out_r = 0.0;

        for i in 0..self.unison_count.0 as usize {
            let mut phase_steps = [0f64; 3];
            for q in 0..3 {
                let mut ph = self.phase[i] + (q as f64 - 2.0) * self.d_phase[i];

                ph = ph - ph.floor(); // bind to 0-1

                ph = ph * 2.0 - 1.0; // assume -1,1
                phase_steps[q] = (ph * ph - 1.0) * ph / 6.0;
            }

            let saw = ((phase_steps[0] + phase_steps[2] - 2.0 * phase_steps[1])
                * 0.25
                * self.d_phase_inv[i]
                * self.d_phase_inv[i]) as f32;

            self.out_l += 0.2 * self.norm[i] * ar * self.pan_l[i] * saw;
            self.out_r += 0.2 * self.norm[i] * ar * self.pan_r[i] * saw;

            self.phase[i] += self.d_phase[i];
            if self.phase[i] > 1.0 {
                self.phase[i] -= 1.0;
            }

            self.filter.step(&mut self.out_l, &mut self.out_r);
        }
    }

    pub fn release(&mut self) {
        self.aeg_mode = AEGMode::Releasing;
        self.time = 0.0;

        if self.time > self.amplitude_release_seconds {
            self.aeg_mode = AEGMode::NewlyOff;
        }
    }

    pub fn update(
        &mut self,
        unison_spread: UnisonSpread,
        oscillator_detune: OscillatorDetune,
        cutoff: Cutoff,
        resonance: Resonance,
        prefilter_vca: PreFilterVCA,
        AmplitudeAttack(amplitude_attack): AmplitudeAttack,
        AmplitudeRelease(amplitude_release): AmplitudeRelease,
        amplitude_envelope_is_gate: AmplitudeEnvelopeIsGate,
        filter_mode: FilterMode,
    ) {
        self.unison_spread = unison_spread;
        self.oscillator_detune = oscillator_detune;
        self.cutoff = cutoff;
        self.resonance = resonance;
        self.prefilter_vca = prefilter_vca;
        self.amplitude_attack_seconds = scale_time_param_to_seconds(amplitude_attack);
        self.amplitude_release_seconds = scale_time_param_to_seconds(amplitude_release);
        self.amplitude_envelope_is_gate = amplitude_envelope_is_gate;
        self.filter_mode = filter_mode;

        self.recalc_pitch();
        self.recalc_filter();
    }

    pub fn recalc_pitch(&mut self) {
        self.base_freq = 440.0
            * 2f32.powf(
                ((self.key as f32
                    + self.pitch_note_expression_value
                    + self.pitch_bend_wheel
                    + (self.oscillator_detune + self.oscillator_detune_mod) / 100.0)
                    - 69.0)
                    / 12.0,
            );

        for i in 0..self.unison_count.0 as usize {
            self.d_phase[i] = ((self.base_freq
                * 2f32.powf(
                    (self.unison_spread + self.unison_spread_mod) * self.unit_shift[i]
                        / 100.0
                        / 12.0,
                ))
                / self.sample_rate) as f64;
            self.d_phase_inv[i] = 1.0 / self.d_phase[i];
        }
    }

    pub fn recalc_filter(&mut self) {
        let co = self.cutoff + self.cutoff_mod;
        let rm = self.resonance + self.resonance_mod;

        let new_fm = self.filter_mode;

        if new_fm != self.filter.mode {
            self.filter.init();
        }
        self.filter.mode = new_fm;
        self.filter.set_coeff(co, rm, self.sr_inv);
    }

    #[inline]
    pub fn is_playing(&self) -> bool {
        match self.aeg_mode {
            AEGMode::Attack | AEGMode::Hold | AEGMode::Releasing => true,
            AEGMode::NewlyOff | AEGMode::Off => false,
        }
    }
}

#[inline]
fn scale_time_param_to_seconds(value: f32) -> f32 {
    let scale_time = ((value - 2.0 / 3.0) * 6.0).clamp(-100.0, 2.0);
    2f32.powf(scale_time)
}

#[derive(Clone, Copy, Default)]
pub struct StereoSimperSVF {
    ic_1_eq: [f32; 2],
    ic_2_eq: [f32; 2],
    g: f32,
    k: f32,
    gk: f32,
    a1: f32,
    a2: f32,
    a3: f32,
    ak: f32,
    mode: FilterMode,
}

impl StereoSimperSVF {
    pub fn set_coeff(&mut self, key: f32, res: f32, sr_inv: f32) {
        let co = (440.0 * 2f32.powf((key - 69.0) / 12.0)).clamp(10.0, 15000.0);
        let res = res.clamp(0.01, 0.99);
        self.g = (PI * co * sr_inv).tan();
        self.k = 2.0 - 2.0 * res;
        self.gk = self.g + self.k;
        self.a1 = 1.0 / (1.0 + self.g * self.gk);
        self.a2 = self.g * self.a1;
        self.a3 = self.g * self.a2;
        self.ak = self.gk * self.a1;
    }

    pub fn step(&mut self, l: &mut f32, r: &mut f32) {
        let vin = [*l, *r];
        let mut res = [0f32; 2];
        for i in 0..2 {
            let v3 = vin[i] - self.ic_2_eq[i];
            let v0 = self.a1 * v3 - self.ak * self.ic_1_eq[i];
            let v1 = self.a2 * v3 + self.a1 * self.ic_1_eq[i];
            let v2 = self.a3 * v3 + self.a2 * self.ic_1_eq[i] + self.ic_2_eq[i];

            self.ic_1_eq[i] = 2.0 * v1 - self.ic_1_eq[i];
            self.ic_2_eq[i] = 2.0 * v2 - self.ic_2_eq[i];

            res[i] = match self.mode {
                FilterMode::LowPass => v2,
                FilterMode::HighPass => v0,
                FilterMode::BandPass => v1,
                FilterMode::Notch => v2 + v0,             // low + high
                FilterMode::Peak => v2 - v0,              // low - high
                FilterMode::All => v2 + v0 - self.k * v1, // low + high * k * band
            }
        }

        (*l, *r) = (res[0], res[1]);
    }

    pub fn init(&mut self) {
        self.ic_1_eq = Default::default();
        self.ic_2_eq = Default::default();
    }
}