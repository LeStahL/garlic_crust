use rustop::opts;
use std::io::*;
use std::fs::*;
use std::fmt::Write;
use std::path::*;

fn main() {
    let (args, rest) = opts! {
        synopsis "This is the command line synthesizer generator for garlic_crust by Team210.";
		param output:Option<String>, desc:"Synthesizer rust file to generate.";
    }.parse_or_exit();

	let mut out_writer: Box<dyn Write> = BufWriter::new(match args.output {
        Some(ref x) => Box::new(File::create(&Path::new(x)).unwrap()),
        None => Box::new(stdout()),
    });
    out_writer.write(b"Test output\n").unwrap();
}
