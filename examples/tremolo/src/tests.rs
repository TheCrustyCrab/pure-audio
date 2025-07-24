#[cfg(test)]
mod tests {
    use pure_audio::{IsPlaying, MonoEffectData, OutEventDispatcher, OutEvents, State, Tempo};
    use crate::{process, Depth, Rate, Shape, LFO};

    struct MockOutEventDispatcher {}

    impl OutEventDispatcher for MockOutEventDispatcher {
        fn dispatch(&self, _event: pure_audio::OutEvent) {}
    }

    #[test]
    pub fn sine_peak_aligns_with_beat() {
        test_peak_trough_alignment_and_depth(Rate::One, Depth(1.0), Shape::Sine);
    }

    #[test]
    pub fn sine_trough_with_beat_is_limited_by_depth() {
        test_peak_trough_alignment_and_depth(Rate::One, Depth(0.75), Shape::Sine);
    }

    #[test]
    pub fn sine_peak_aligns_with_second_beat() {
        test_peak_trough_alignment_and_depth(Rate::Second, Depth(1.0), Shape::Sine);
    }

    #[test]
    pub fn sine_trough_with_second_beat_is_limited_by_depth() {
        test_peak_trough_alignment_and_depth(Rate::Second, Depth(0.75), Shape::Sine);
    }

    #[test]
    pub fn sine_peak_aligns_with_quarter_beat() {
        test_peak_trough_alignment_and_depth(Rate::Quarter, Depth(1.0), Shape::Sine);
    }

    #[test]
    pub fn sine_trough_with_quarter_beat_is_limited_by_depth() {
        test_peak_trough_alignment_and_depth(Rate::Quarter, Depth(0.75), Shape::Sine);
    }

    #[test]
    pub fn sine_peak_aligns_with_eighth_beat() {
        test_peak_trough_alignment_and_depth(Rate::Eighth, Depth(1.0), Shape::Sine);
    }

    #[test]
    pub fn sine_trough_with_eighth_beat_is_limited_by_depth() {
        test_peak_trough_alignment_and_depth(Rate::Eighth, Depth(0.75), Shape::Sine);
    }

    #[test]
    pub fn sine_peak_aligns_with_sixteenth_beat() {
        test_peak_trough_alignment_and_depth(Rate::Sixteenth, Depth(1.0), Shape::Sine);
    }

    #[test]
    pub fn sine_trough_with_sixteenth_beat_is_limited_by_depth() {
        test_peak_trough_alignment_and_depth(Rate::Sixteenth, Depth(0.75), Shape::Sine);
    }

    #[test]
    pub fn triangle_peak_aligns_with_beat() {
        test_peak_trough_alignment_and_depth(Rate::One, Depth(1.0), Shape::Triangle);
    }

    #[test]
    pub fn triangle_trough_with_beat_is_limited_by_depth() {
        test_peak_trough_alignment_and_depth(Rate::One, Depth(0.75), Shape::Triangle);
    }

    #[test]
    pub fn triangle_peak_aligns_with_second_beat() {
        test_peak_trough_alignment_and_depth(Rate::Second, Depth(1.0), Shape::Triangle);
    }

    #[test]
    pub fn triangle_trough_with_second_beat_is_limited_by_depth() {
        test_peak_trough_alignment_and_depth(Rate::Second, Depth(0.75), Shape::Triangle);
    }

    #[test]
    pub fn triangle_peak_aligns_with_quarter_beat() {
        test_peak_trough_alignment_and_depth(Rate::Quarter, Depth(1.0), Shape::Triangle);
    }

    #[test]
    pub fn triangle_trough_with_quarter_beat_is_limited_by_depth() {
        test_peak_trough_alignment_and_depth(Rate::Quarter, Depth(0.75), Shape::Triangle);
    }

    #[test]
    pub fn triangle_peak_aligns_with_eighth_beat() {
        test_peak_trough_alignment_and_depth(Rate::Eighth, Depth(1.0), Shape::Triangle);
    }

    #[test]
    pub fn triangle_trough_with_eighth_beat_is_limited_by_depth() {
        test_peak_trough_alignment_and_depth(Rate::Eighth, Depth(0.75), Shape::Triangle);
    }

    #[test]
    pub fn triangle_peak_aligns_with_sixteenth_beat() {
        test_peak_trough_alignment_and_depth(Rate::Sixteenth, Depth(1.0), Shape::Triangle);
    }

    #[test]
    pub fn triangle_trough_with_sixteenth_beat_is_limited_by_depth() {
        test_peak_trough_alignment_and_depth(Rate::Sixteenth, Depth(0.75), Shape::Triangle);
    }

    fn test_peak_trough_alignment_and_depth(rate: Rate, depth: Depth, shape: Shape) {
        const SAMPLE_RATE: usize = 48000;
        const FRAME_LENGTH: usize = 128;
        const FRAMES_PER_SECOND: usize = SAMPLE_RATE / FRAME_LENGTH;
        const TEMPO: f32 = 60.0;
        const SECONDS_TO_PLAY: usize = 500;

        let mut lfo = LFO::default();
        lfo.activate(SAMPLE_RATE as f64, FRAME_LENGTH, FRAME_LENGTH);
        lfo.set_rate(rate);
        lfo.set_depth(depth.0);

        let input = [1.0; FRAME_LENGTH]; // constant 1 to reveal the lfo wave
        let mut output = [0.0; FRAME_LENGTH];

        // a few process calls while not playing
        for _ in 0..3 {
            let data = MonoEffectData::<LFO> {
                events: &[],
                input: &input,
                output: &mut output,
                out_events: OutEvents::new(&MockOutEventDispatcher {}),
                state: &mut lfo
            };
            process(data, IsPlaying(false), Tempo(TEMPO), rate, depth, shape);
        }

        let mut signal_buffer = Vec::with_capacity(SECONDS_TO_PLAY * SAMPLE_RATE as usize);

        // start playing
        for _ in 0..SECONDS_TO_PLAY {
            for _ in 0..FRAMES_PER_SECOND {
                let data = MonoEffectData::<LFO> {
                    events: &[],
                    input: &input,
                    output: &mut output,
                    out_events: OutEvents::new(&MockOutEventDispatcher {}),
                    state: &mut lfo
                };
                process(data, IsPlaying(true), Tempo(TEMPO), rate, depth, shape);
                for sample in output {
                    signal_buffer.push(sample);
                }
            }
        }

        let lfo_frequency = match rate {
            Rate::One => 1,
            Rate::Second => 2,
            Rate::Quarter => 4,
            Rate::Eighth => 8,
            Rate::Sixteenth => 16,
        };

        // validate peak
        for &sample in signal_buffer.iter().step_by(SAMPLE_RATE / lfo_frequency) {
            assert!(sample >= 0.999 && sample <= 1.0);
        }
        // validate trough
        for &sample in signal_buffer.iter().skip(SAMPLE_RATE / lfo_frequency / 2).step_by(SAMPLE_RATE / lfo_frequency) {
            assert!(sample <= 1.0 - depth + 0.001 && sample >= 1.0 - depth);
        }
    }
}