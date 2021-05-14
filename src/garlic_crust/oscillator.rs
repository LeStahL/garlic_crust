use crate::math::{sin, TAU};
use super::*;

#[derive(Debug)]
pub enum BaseWave {
    Sine,
    Saw,
    Square,
    Triangle,
    Zero,
}

pub struct Oscillator {
    pub shape: BaseWave,
    pub volume: Edge,
    pub frequency: Edge,
    pub phasemod: Edge,
    pub detune: Edge,
    pub phase: TimeFloat, // what would be _phase convention?
    // makes sense to define some BaseOperator which holds seq_cursor and output?
    pub seq_cursor: usize,
    pub output: Edge,
}

impl Operator for Oscillator {
    fn handle_message(&mut self, message: &SeqMsg) {
        match &message {
            SeqMsg::NoteOn(note_key, _) => {
                self.phase = 0.;
                self.frequency = Edge::constant(note_frequency(*note_key));
            },
            // could react to Volume or whatevs here.
            _ => ()
        }
    }

    fn evaluate(&mut self, sample: usize, _: TimeFloat) -> AmpFloat {
        let phase = self.phase + self.phasemod.evaluate(sample);

        let result_in_tune = self.evaluate_at(phase);
        let result_detuned = self.evaluate_at(phase * (1. + self.detune.evaluate(sample)));

        0.5 * (result_in_tune + result_detuned) * self.volume.evaluate(sample)
    }

    fn advance(&mut self, sample: usize) {
        self.phase += self.frequency.evaluate(sample) / SAMPLERATE;
        if self.phase >= 1. {
            self.phase -= 1.;
        }
    }

    fn get_cursor(&mut self) -> usize {
        self.seq_cursor
    }

    fn inc_cursor(&mut self) {
        self.seq_cursor += 1;
    }
}

impl Oscillator {
    fn evaluate_at(&self, phase: TimeFloat) -> AmpFloat {
        let basewave_value: AmpFloat = match self.shape {
            BaseWave::Sine => sin(TAU * phase),
            BaseWave::Square => (37. * sin(TAU * phase)).clamp(-1., 1.),
            BaseWave::Saw => 2. * libm::fmodf(phase, 1.) - 1.,
            BaseWave::Triangle => 4. * libm::fabsf(libm::fmodf(phase, 1.) - 0.5) - 1.0,
            _ => 0.,
        };

        basewave_value.clamp(-1., 1.)
    }
}
