#[cfg(test)]
mod tests {
    use crate::{
        AEGMode, AmplitudeAttack, AmplitudeEnvelopeIsGate, AmplitudeRelease, Cutoff, FilterMode, OscillatorDetune, PreFilterVCA, Resonance, SawState, UnisonCount, UnisonSpread,
        process,
    };
    use pure_audio::{Event, OutEventDispatcher, OutEvents, State, StereoSynthData};

    struct MockOutEventDispatcher {}

    impl OutEventDispatcher for MockOutEventDispatcher {
        fn dispatch(&self, _event: pure_audio::OutEvent) {}
    }

    #[test]
    fn note_on_off_in_same_frame_stops_playing_note_after_release_time() {
        let mut state = SawState::default();
        state.activate(48000.0, 128, 128);
        let mut output_l = [0.0; 128];
        let mut output_r = [0.0; 128];
        
        let unison_count = UnisonCount(7);
        let unison_spread = UnisonSpread(10.0);
        let oscillator_detune = OscillatorDetune(0.0);
        let amplitude_attack = AmplitudeAttack(0.0); // maps to 0.0625s = 3000 samples
        let amplitude_release = AmplitudeRelease(0.2); // maps to 0.1435 = 6888 samples, 53.81 * 128
        let amplitude_envelope_is_gate = AmplitudeEnvelopeIsGate(false);
        let prefilter_vca = PreFilterVCA(1.0);
        let cutoff = Cutoff(69.0);
        let resonance = Resonance(0.7);
        let filter_mode = FilterMode::HighPass;

        // 1st call, note on and off
        let data = StereoSynthData::<SawState> {
            events: &vec![
                Event::NoteOn {
                    port_index: 0,
                    channel: 0,
                    key: 64,
                    note_id: 0,
                    velocity: 127,
                },
                Event::NoteOff {
                    port_index: 0,
                    channel: 0,
                    key: 64,
                    note_id: 0,
                    velocity: 127,
                },
            ],
            out_events: OutEvents::new(&MockOutEventDispatcher {}),
            outputs: [&mut output_l, &mut output_r],
            state: &mut state,
        };
        process(
            data,
            unison_count,
            unison_spread,
            oscillator_detune,
            amplitude_attack,
            amplitude_release,
            amplitude_envelope_is_gate,
            prefilter_vca,
            cutoff,
            resonance,
            filter_mode
        );

        for _ in 0..52 {
            let data = StereoSynthData::<SawState> {
                events: &vec![],
                out_events: OutEvents::new(&MockOutEventDispatcher {}),
                outputs: [&mut output_l, &mut output_r],
                state: &mut state,
            };
            process(
                data,
                unison_count,
                unison_spread,
                oscillator_detune,
                amplitude_attack,
                amplitude_release,
                amplitude_envelope_is_gate,
                prefilter_vca,
                cutoff,
                resonance,
                filter_mode
            );
            let voice = state
                .voices
                .iter()
                .find(|voice| voice.is_playing())
                .unwrap();
            assert_eq!(voice.aeg_mode, AEGMode::Releasing);
        }

        let data = StereoSynthData::<SawState> {
            events: &vec![],
            out_events: OutEvents::new(&MockOutEventDispatcher {}),
            outputs: [&mut output_l, &mut output_r],
            state: &mut state,
        };
        process(
            data,
            unison_count,
            unison_spread,
            oscillator_detune,
            amplitude_attack,
            amplitude_release,
            amplitude_envelope_is_gate,
            prefilter_vca,
            cutoff,
            resonance,
            filter_mode
        );

        // there should be no playing voices
        let voice = state.voices.iter().find(|voice| voice.is_playing());
        assert!(voice.is_none());
    }

    #[test]
    fn three_different_notes_on_off_in_same_frame_stops_playing_notes_after_release_time() {
        let mut state = SawState::default();
        state.activate(48000.0, 128, 128);
        let mut output_l = [0.0; 128];
        let mut output_r = [0.0; 128];
        
        let unison_count = UnisonCount(7);
        let unison_spread = UnisonSpread(10.0);
        let oscillator_detune = OscillatorDetune(0.0);
        let amplitude_attack = AmplitudeAttack(0.0); // maps to 0.0625s = 3000 samples
        let amplitude_release = AmplitudeRelease(0.2); // maps to 0.1435 = 6888 samples, 53.81 * 128
        let amplitude_envelope_is_gate = AmplitudeEnvelopeIsGate(false);
        let prefilter_vca = PreFilterVCA(1.0);
        let cutoff = Cutoff(69.0);
        let resonance = Resonance(0.7);
        let filter_mode = FilterMode::HighPass;

        // 1st call, notes on
        let data = StereoSynthData::<SawState> {
            events: &vec![
                Event::NoteOn {
                    port_index: 0,
                    channel: 0,
                    key: 64,
                    note_id: 0,
                    velocity: 127,
                },
                Event::NoteOn {
                    port_index: 0,
                    channel: 0,
                    key: 65,
                    note_id: 0,
                    velocity: 127,
                },
                Event::NoteOn {
                    port_index: 0,
                    channel: 0,
                    key: 66,
                    note_id: 0,
                    velocity: 127,
                },
                Event::NoteOff {
                    port_index: 0,
                    channel: 0,
                    key: 64,
                    note_id: 0,
                    velocity: 127,
                },
                Event::NoteOff {
                    port_index: 0,
                    channel: 0,
                    key: 65,
                    note_id: 0,
                    velocity: 127,
                },
                Event::NoteOff {
                    port_index: 0,
                    channel: 0,
                    key: 66,
                    note_id: 0,
                    velocity: 127,
                },
            ],
            out_events: OutEvents::new(&MockOutEventDispatcher {}),
            outputs: [&mut output_l, &mut output_r],
            state: &mut state,
        };
        process(
            data,
            unison_count,
            unison_spread,
            oscillator_detune,
            amplitude_attack,
            amplitude_release,
            amplitude_envelope_is_gate,
            prefilter_vca,
            cutoff,
            resonance,
            filter_mode
        );

        for _ in 0..52 {
            let data = StereoSynthData::<SawState> {
                events: &vec![],
                out_events: OutEvents::new(&MockOutEventDispatcher {}),
                outputs: [&mut output_l, &mut output_r],
                state: &mut state,
            };
            process(
                data,
                unison_count,
                unison_spread,
                oscillator_detune,
                amplitude_attack,
                amplitude_release,
                amplitude_envelope_is_gate,
                prefilter_vca,
                cutoff,
                resonance,
                filter_mode
            );
            
            let voices = state
                .voices
                .iter()
                .filter(|voice| voice.is_playing())
                .collect::<Vec<_>>();

            assert_eq!(3, voices.len());
            for voice in voices {
                assert_eq!(voice.aeg_mode, AEGMode::Releasing);
            }
        }

        let data = StereoSynthData::<SawState> {
            events: &vec![],
            out_events: OutEvents::new(&MockOutEventDispatcher {}),
            outputs: [&mut output_l, &mut output_r],
            state: &mut state,
        };
        process(
            data,
            unison_count,
            unison_spread,
            oscillator_detune,
            amplitude_attack,
            amplitude_release,
            amplitude_envelope_is_gate,
            prefilter_vca,
            cutoff,
            resonance,
            filter_mode
        );

        // there should be no playing voices
        let voice = state.voices.iter().find(|voice| voice.is_playing());
        assert!(voice.is_none());
    }

    #[test]
    fn two_same_notes_on_off_in_same_frame_stops_playing_both_notes_after_release_time() {
        let mut state = SawState::default();
        state.activate(48000.0, 128, 128);
        let mut output_l = [0.0; 128];
        let mut output_r = [0.0; 128];
        
        let unison_count = UnisonCount(7);
        let unison_spread = UnisonSpread(10.0);
        let oscillator_detune = OscillatorDetune(0.0);
        let amplitude_attack = AmplitudeAttack(0.0); // maps to 0.0625s = 3000 samples
        let amplitude_release = AmplitudeRelease(0.2); // maps to 0.1435 = 6888 samples, 53.81 * 128
        let amplitude_envelope_is_gate = AmplitudeEnvelopeIsGate(false);
        let prefilter_vca = PreFilterVCA(1.0);
        let cutoff = Cutoff(69.0);
        let resonance = Resonance(0.7);
        let filter_mode = FilterMode::HighPass;

        // 1st call, note on
        let data = StereoSynthData::<SawState> {
            events: &vec![
                Event::NoteOn {
                    port_index: 0,
                    channel: 0,
                    key: 52,
                    note_id: 0,
                    velocity: 127,
                }
            ],
            out_events: OutEvents::new(&MockOutEventDispatcher {}),
            outputs: [&mut output_l, &mut output_r],
            state: &mut state,
        };
        process(
            data,
            unison_count,
            unison_spread,
            oscillator_detune,
            amplitude_attack,
            amplitude_release,
            amplitude_envelope_is_gate,
            prefilter_vca,
            cutoff,
            resonance,
            filter_mode
        );

        // 7 calls without events
        for _ in 0..7 {
            let data = StereoSynthData::<SawState> {
                events: &vec![],
                out_events: OutEvents::new(&MockOutEventDispatcher {}),
                outputs: [&mut output_l, &mut output_r],
                state: &mut state,
            };
            process(
                data,
                unison_count,
                unison_spread,
                oscillator_detune,
                amplitude_attack,
                amplitude_release,
                amplitude_envelope_is_gate,
                prefilter_vca,
                cutoff,
                resonance,
                filter_mode
            );
        }

        // 9th call, note off + on + off
        let data = StereoSynthData::<SawState> {
            events: &vec![
                Event::NoteOff {
                    port_index: 0,
                    channel: 0,
                    key: 52,
                    note_id: 0,
                    velocity: 127,
                },
                Event::NoteOn {
                    port_index: 0,
                    channel: 0,
                    key: 52,
                    note_id: 0,
                    velocity: 127,
                },
                Event::NoteOff {
                    port_index: 0,
                    channel: 0,
                    key: 52,
                    note_id: 0,
                    velocity: 127,
                },
            ],
            out_events: OutEvents::new(&MockOutEventDispatcher {}),
            outputs: [&mut output_l, &mut output_r],
            state: &mut state,
        };
        process(
            data,
            unison_count,
            unison_spread,
            oscillator_detune,
            amplitude_attack,
            amplitude_release,
            amplitude_envelope_is_gate,
            prefilter_vca,
            cutoff,
            resonance,
            filter_mode
        );

        // wait for release
        for _ in 0..52 {
            let data = StereoSynthData::<SawState> {
                events: &vec![],
                out_events: OutEvents::new(&MockOutEventDispatcher {}),
                outputs: [&mut output_l, &mut output_r],
                state: &mut state,
            };
            process(
                data,
                unison_count,
                unison_spread,
                oscillator_detune,
                amplitude_attack,
                amplitude_release,
                amplitude_envelope_is_gate,
                prefilter_vca,
                cutoff,
                resonance,
                filter_mode
            );

            let voice = state.voices.iter().find(|voice| voice.is_playing()).unwrap();
            assert_eq!(voice.aeg_mode, AEGMode::Releasing);
        }

        let data = StereoSynthData::<SawState> {
            events: &vec![],
            out_events: OutEvents::new(&MockOutEventDispatcher {}),
            outputs: [&mut output_l, &mut output_r],
            state: &mut state,
        };
        process(
            data,
            unison_count,
            unison_spread,
            oscillator_detune,
            amplitude_attack,
            amplitude_release,
            amplitude_envelope_is_gate,
            prefilter_vca,
            cutoff,
            resonance,
            filter_mode
        );
        
        // there should be no playing voices
        let voice = state.voices.iter().find(|voice| voice.is_playing());
        assert!(voice.is_none());
    }
}