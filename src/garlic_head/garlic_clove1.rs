use crate::garlic_crust::*;
use super::*;

// A garlic clove is basically a garlic crust "preset", i.e. its internal wiring

// the member fields
pub struct Clove1State {
    osc_osc1: oscillator::Oscillator,
    osc_osc1_output: Edge,

    env_osc1: envelope::Envelope,
    env_osc1_output: Edge,

    osc_osc2: oscillator::Oscillator,
    osc_osc2_output: Edge,

    env_osc2: envelope::Envelope,
    env_osc2_output: Edge,

    osc_lfo1: oscillator::Oscillator,
    osc_lfo1_output: Edge,

    lp1: filter::Filter,
    lp1_output: Edge,

    math_lfofiltertransform: Edge,
}

pub struct Config1 {
    pub env_attack: Edge,
    pub env_decay: Edge,
    pub env_shape: envelope::BaseEnv,
    pub osc1_shape: oscillator::BaseWave,
}

pub struct Config2 {
    pub osc2_shape: oscillator::BaseWave,
}

pub fn create_config1(preset: &str) -> Config1 {
    match preset {
        _ => Config1 {
            env_attack: Edge::zero(),
            env_decay: Edge::constant(0.3),
            env_shape: envelope::BaseEnv::ExpDecay,
            osc1_shape: oscillator::BaseWave::Triangle,
        }
    }
}

pub fn create_config2(preset: &str) -> Config2 {
    match preset {
        _ => Config2 {
            osc2_shape: oscillator::BaseWave::Square,
        }
    }
}

pub fn create_state(config1: &Config1, config2: &Config2) -> Clove1State {
    Clove1State {
        osc_osc1: oscillator::Oscillator {
            shape: config1.osc1_shape,
            volume: Edge::constant(1.),
            ..Default::default()
        },
        osc_osc1_output: Edge::zero(),

        env_osc1: envelope::Envelope {
            attack: config1.env_attack,
            decay: config1.env_decay,
            shape: envelope::BaseEnv::ExpDecay,
            ..Default::default()
        },
        env_osc1_output: Edge::zero(),

        osc_osc2: oscillator::Oscillator {
            shape: config2.osc2_shape,
            volume: Edge::constant(1.),
            ..Default::default()
        },
        osc_osc2_output: Edge::zero(),

        env_osc2: envelope::Envelope {
            attack: config1.env_attack,
            decay: config1.env_decay,
            shape: envelope::BaseEnv::ExpDecay,
            ..Default::default()
        },
        env_osc2_output: Edge::zero(),

        osc_lfo1: oscillator::Oscillator {
            shape: oscillator::BaseWave::Triangle,
            volume: Edge::constant(1.),
            frequency: Edge::constant(0.2),
            phase: 0.5,
            ..Default::default()
        },
        osc_lfo1_output: Edge::zero(),

        lp1: filter::Filter {
            shape: filter::FilterType::LowPass,
            cutoff: Edge::zero(),
            state: filter::FilterState::new(),
            input: Edge::zero(),
        },
        lp1_output: Edge::zero(),

        math_lfofiltertransform: Edge::zero(),
    }
}

#[inline]
pub fn process(sequence: &[SeqEvent], block_offset: usize, state: &mut Clove1State) -> Edge {
    // cloves are monophonic, there is only one time since the last noteon

    // unclear: management of seq_cursor, output could also be in the GarlicClove1State. think about.
    // sequence would then have to be split into the blocks itself, but this could be done by garlic_extract

    // THESE CHAINS WILL BE GIVEN BY knober

    generate_from_func(func_osc_phasemod, block_offset, &mut state.osc_osc1.phasemod);
    generate_from_func(func_osc_factor, block_offset, &mut state.osc_osc1.freq_factor);

    // first branch
    process_operator_seq(&mut state.env_osc1, &sequence, block_offset, &mut state.env_osc1_output);
    state.osc_osc1.volume = state.env_osc1_output;
    process_operator_seq(&mut state.osc_osc1, &sequence, block_offset, &mut state.osc_osc1_output);

    // second branch
    process_operator_seq(&mut state.env_osc2, &sequence, block_offset, &mut state.env_osc2_output);
    state.osc_osc2.volume = state.env_osc2_output;
    process_operator_seq(&mut state.osc_osc2, &sequence, block_offset, &mut state.osc_osc2_output);

    // third branch
    process_operator(&mut state.osc_lfo1, block_offset, &mut state.osc_lfo1_output);
    generate_math_lfofiltertransform(&state.osc_lfo1_output, &mut state.math_lfofiltertransform);

    // filter junction
    //state.lp1.input = math_mixer(&osc_osc1_output, &osc_osc2_output, &Edge::constant(0.5)); // more advanced blocks will have to be converted to Rust code, but I can help with that
    math_mixer(&state.osc_osc1_output, &Edge::constant(1.), &state.osc_osc2_output, &mut state.lp1.input); // more advanced blocks will have to be converted to Rust code, but I can help with that
    state.lp1.cutoff = state.math_lfofiltertransform;
    process_operator(&mut state.lp1, block_offset, &mut state.lp1_output);

    state.lp1_output
}

// inline or not inline?
#[inline]
// individual math operators (more complex than Edge::mad()) might be created directly in the clove
fn math_mixer(input1: &Edge, input2: &Edge, cv: &Edge, output: &mut Edge) {
    for sample in 0 .. BLOCK_SIZE {
        output.put_at(sample,
            cv.evaluate(sample) * (input1.evaluate(sample) + input2.evaluate(sample))
        );
    }
}

#[inline]
fn generate_math_lfofiltertransform(input: &Edge, output: &mut Edge) {
    for sample in 0 .. BLOCK_SIZE {
        output.put_at(sample, 2000. + input.evaluate(sample) * 1800.);
    }
}

fn func_osc_phasemod(t: TimeFloat) -> AmpFloat {
    0.02 * libm::sinf(4.*t)
}

fn func_osc_factor(t: TimeFloat) -> AmpFloat {
    0.5005
}

// with this commit: 71.3 seconds for 16 second track (outputs not stored in Op, block_size 1024)
// same with block_size 256: 10 seconds?? wtf?
// block_size 512: 55 seconds;

// THINGS TO TEST:
// put "env_osc1_output" again as a field of "env_osc1.output", if that helps the compiler?
// Split Sequence into Chunks, one for each 512-sample-block
// Put Sequence into Byte Array
// use get_unchecked()
// multithreading?? -- each Clove can be processed simultaneously
// should every Edge always hold its array??