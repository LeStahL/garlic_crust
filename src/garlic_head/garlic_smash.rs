use crate::garlic_crust::*;
use super::*;

// Garlic Smashsare Drum Synth Cloves
// they are monosynths by design
// and use pattern functions for triggering instead of NoteOn Sequences

// the member fields
pub struct Smash1State {
    pub output: BlockArray,

    osc: oscillator::Oscillator,
    osc_output: Edge,

    env_vca: envelope::Envelope,
    env_vca_output: Edge,

    env_freq: envelope::Envelope,
    env_freq_output: Edge,

    // waveshapes are just math blocks (i.e. some function)
    dist: Edge,
}

pub fn create_state() -> Smash1State {
    Smash1State {
        output: EMPTY_BLOCKARRAY,

        osc: oscillator::Oscillator {
            frequency: Edge::constant(46.25), // F#1
            detune: Edge::constant_stereo([0., 0.02]),
            phasemod: Edge::constant_stereo([0.3, 0.]),
            shape: oscillator::BaseWave::Triangle,
            ..Default::default()
        },
        osc_output: Edge::zero(),

        env_vca: envelope::Envelope {
            attack: Edge::zero(),
            decay: Edge::constant(0.1),
            min: Edge::zero(),
            max: Edge::constant(1.),
            shape: envelope::BaseEnv::ExpDecay,
            ..Default::default()
        },
        env_vca_output: Edge::zero(),

        env_freq: envelope::Envelope {
            shape: envelope::BaseEnv::Generic {
                func: kick_freq_env
            },
            ..Default::default()
        },
        env_freq_output: Edge::zero(),

        dist: Edge::zero(),
    }
}

#[inline]
fn slope(t: TimeFloat, t0: TimeFloat, t1: TimeFloat, y0: MonoSample, y1: MonoSample) -> MonoSample {
    y0 + (t - t0) / (t1 - t0) * (y1 - y0)
}

fn powerslope(t: TimeFloat, t0: TimeFloat, t1: TimeFloat, y0: MonoSample, y1: MonoSample, power: f32) -> MonoSample {
    let result = slope(t, t0, t1, y0, y1);
    libm::powf(result, power)
}


fn kick_freq_env(t: TimeFloat) -> MonoSample {
    let a = 5e-3;
    let ah = a + 17e-3;
    let ahd = ah + 54e-3;
    match t {
        x if x < a => 1., // slope(t, 0., a, 0., 1.),
        x if x < ah => 1.,
        x if x < ahd => 0., // powerslope(t, ah, ahd, 1., 0., 3.),
        _ => 0.
    }
}

/* process() is the heart of the Garlic Smash and will be generated by knober
 *
 * unclear: management of seq_cursor, output could also be in the GarlicSmash1State. think about.
 * sequence would then have to be split into the blocks itself, but this could be done by garlic_extract. meh
 *
 */
#[inline]
pub fn process(block_offset: usize, state: &mut Smash1State) {

    process_operator_dyn(&mut state.env_vca, &trigger, block_offset, &mut state.env_vca_output);
    //state.osc.volume = state.env_vca_output;
    //process_operator_dyn(&mut state.env_freq, &trigger, block_offset, &mut state.env_freq_output);
    //state.osc.frequency = state.env_freq_output;

    //process_operator_dyn(&mut state.osc, &trigger, block_offset, &mut state.osc_output);
    process_operator(&mut state.osc, &mut state.osc_output);

    //math_distort(&mut state.osc_output);

    state.osc_output.write_to(&mut state.output);
}

/* trigger() holds, as a mathematical function, the repetition pattern of the kick.
 * it will be produced by dynamo210 soon.
*/
#[inline]
fn trigger(total_sample: usize) -> bool {
    let total_beat = DYNAMO.beat(total_sample);
    //if total_beat >= pattern_start_beat && total_beat < pattern_end_beat //inside beat condition
    let pattern_start_beat = 0.;
    let pattern_end_beat = 4.;
    let beat_length = pattern_end_beat - pattern_start_beat;
    let beat_inside_pattern = libm::fmodf(total_beat - pattern_start_beat, beat_length);
    // two options: something regular (-> fmodf) or one-shots
    let beat_trigger = libm::fmodf(beat_inside_pattern, 1.);

    return beat_trigger >= 0. && beat_trigger <= INV_SAMPLERATE;
}

// inline or not inline?
#[inline]
// individual math operators (more complex than Edge::mad()) might be created directly in the smash
fn math_mixer(input1: &Edge, input2: &Edge, cv: &Edge, output: &mut Edge) {
    for sample in 0 .. BLOCK_SIZE {
        for ch in 0 .. 2 { // the looping could be hidden by generalizing 2000 + and * 1800 to
            output.put_at_mono(sample, ch,
                cv.evaluate_mono(sample, ch) * (input1.evaluate_mono(sample, ch) + input2.evaluate_mono(sample, ch))
            );
        }
    }
}

#[inline]
fn math_waveshape(output: &mut Edge, waveshape: fn(MonoSample) -> MonoSample) {
    for sample in 0 .. BLOCK_SIZE {
        for ch in 0 .. 2 {
            let input = output.evaluate_mono(sample, ch);
            output.put_at_mono(sample, ch, libm::copysignf(input, waveshape(libm::fabsf(input))));
        }
    }
}

#[inline]
fn math_distort(output: &mut Edge) {
    math_waveshape(output, |x| if x >= 0.1 && x < 0.13 { 0.5 } else { x });
}

// with this commit: 71.3 seconds for 16 second track (outputs not stored in Op, block_size 1024)
// same with block_size 256: 10 seconds?? wtf?
// block_size 512: 55 seconds;

// THINGS TO TEST:
// put "env_osc1_output" again as a field of "env_osc1.output", if that helps the compiler?
// Split Sequence into Chunks, one for each 512-sample-block
// Put Sequence into Byte Array
// use get_unchecked()
// multithreading?? -- each Smash can be processed simultaneously
// should every Edge always hold its array??