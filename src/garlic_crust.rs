use crate::garlic_head::{BLOCK_SIZE, BlockArray, EMPTY_BLOCKARRAY};

pub mod oscillator;
pub mod envelope;
pub mod filter;

pub type TimeFloat = f32;
pub type AmpFloat = f32;

pub const SAMPLERATE: f32 = 44100.;

// this might be solved with enum / variants; but not for now.
// there could be another option: a function pointer. come back to that option when I know whether gud or notgud
#[derive(Copy, Clone)]
pub struct Edge {
    array: Option<BlockArray>,
    function: Option<fn(playhead: TimeFloat) -> AmpFloat>, // hm. is it good to have fn(globaltime, playhead) instead of just fn(playhead) ?
    constant: AmpFloat,
}

pub type PlayFunc = fn(TimeFloat) -> AmpFloat;

impl Edge {
    pub fn constant(value: f32) -> Edge {
        Edge {
            array: None,
            function: None,
            constant: value,
        }
    }

    pub fn function(function: PlayFunc) -> Edge { // HAVE NO IDEA ABOUT THIS YET..!!
        Edge {
            array: None,
            function: Some(function),
            constant: 0.,
        }
    }

    pub fn array(block: BlockArray) -> Edge {
        Edge {
            array: Some(block),
            function: None,
            constant: 0.,
        }
    }

    pub fn zero() -> Edge {
        Edge::constant(0.)
    }

    // this is, of course, pure decadence.
    pub fn one() -> Edge {
        Edge::constant(1.)
    }

    pub fn is_const(&self) -> bool {
        self.array.is_none() && self.function.is_none()
    }

    pub fn evaluate(&self, pos: usize) -> AmpFloat {
        if let Some(array) = self.array {
            return array[pos % BLOCK_SIZE];
        }
        if let Some(func) = self.function {
            // no idea whether this somehow works or rather is garbage
            return func(pos as TimeFloat / SAMPLERATE);
        }
        return self.constant;
    }

    pub fn scale(&mut self, factor: f32) -> Edge {
        if let Some(mut array) = self.array {
            for pos in 0 .. BLOCK_SIZE {
                array[pos] = factor * array[pos];
            }
            self.array = Some(array);
        }
        if let Some(func) = self.function {
            // ..?? maybe we somehow multiply this by self.constant. think about that later.
        }
        self.constant *= factor;

        *self
    }

    pub fn times(&mut self, other: &Edge) -> Edge {
        if other.is_const() {
            return self.scale(other.constant);
        }
        let mut array = EMPTY_BLOCKARRAY.clone();
        for pos in 0 .. BLOCK_SIZE {
            array[pos] = other.evaluate(pos) * self.evaluate(pos);
        }
        Edge::array(array)
    }

    pub fn mad(&self, multiply: &Edge, add: &Edge) -> Edge {
        let mut array = EMPTY_BLOCKARRAY;
        for pos in 0 .. BLOCK_SIZE {
            array[pos] = multiply.evaluate(pos) * self.evaluate(pos) + add.evaluate(pos);
        }
        Edge::array(array)
    }
}

pub trait Operator {
    fn handle_message(&mut self, message: &SeqMsg);
    fn evaluate(&mut self, sample: usize, total_time: TimeFloat) -> AmpFloat;
    fn advance(&mut self, sample: usize);
    fn get_cursor(&mut self) -> usize;
    fn inc_cursor(&mut self);
}

pub fn next_event_option(sequence: &[SeqEvent], cursor: usize) -> Option<SeqNormalizedEvent> {
    match cursor == sequence.len() {
        true => None,
        false => Some(SeqNormalizedEvent::from(&sequence[cursor]))
    }
}

pub fn process_operator_seq<O: Operator>(op: &mut O, sequence: &[SeqEvent], block_offset: usize) -> Edge {
    let mut output = EMPTY_BLOCKARRAY; // .clone();

    let mut next_event = next_event_option(&sequence, op.get_cursor());

    for sample in 0 .. BLOCK_SIZE {
        let time: TimeFloat = (sample + block_offset) as TimeFloat / SAMPLERATE;

        while let Some(event) = next_event {
            if event.time > time {
                break;
            }
            op.handle_message(&event.message);
            op.inc_cursor();
            next_event = next_event_option(&sequence, op.get_cursor());
        }

        output[sample] = op.evaluate(sample + block_offset, time);

        op.advance(sample);
    }

    Edge::array(output)
}

pub fn process_operator<O: Operator>(op: &mut O, block_offset: usize) -> Edge {
    let mut output = EMPTY_BLOCKARRAY; // .clone();

    for sample in 0 .. BLOCK_SIZE {
        let time: TimeFloat = (sample + block_offset) as TimeFloat / SAMPLERATE;

        output[sample] = op.evaluate(sample, time);

        op.advance(sample);
    }

    Edge::array(output)
}

pub type SeqParameter = usize; // check whether we have enough withi half::f16

// design decision for now: garlic_extract will take BPM information and give you a sequence over _time_
#[derive(Clone, Copy, Debug)]
pub struct SeqEvent {
    pub time: u32, // in milliseconds, this should be precise enough
    pub message: SeqMsg,
}

#[derive(Clone, Copy, Debug)]
pub struct SeqNormalizedEvent {
    pub time: TimeFloat,
    pub message: SeqMsg,
}

impl SeqNormalizedEvent {
    pub fn from(seq_event: &SeqEvent) -> SeqNormalizedEvent {
        SeqNormalizedEvent {
            time: 0.0001 * seq_event.time as TimeFloat,
            message: seq_event.message,
        }
    }
}

// can I do this polymorphically in no_std Rust?
#[derive(Clone, Copy, Debug)]
pub enum SeqMsg {
    NoteOn(SeqParameter, SeqParameter),
    NoteOff,
    SetVel,
    SetSlide,
    SetPan,
    // ...?
}

pub fn note_frequency(note_number: SeqParameter) -> f32 {
    440. * libm::powf(2., (note_number as f32 - 69.)/12.)
}

// LIST OF INVESTIGATIONS, watch for Size / Performance:
// ... probably after first track exists, to see REAL difference
//
// loop vs for loop -- no difference at all (sizewise)
// unsafe get_unchecked_mut vs. get_mut & unwrap
// math::sin vs other sin?
