#![feature(io_read_to_string)]

use ::rustop::opts;
use ::std::io::*;
use ::std::fs::*;
use ::std::path::*;
use ::std::boxed::*;

fn main() {
    let (args, rest) = opts! {
		auto_shorts true;
        synopsis "This is the command line synthesizer generator for garlic_crust by Team210.";
		opt output:Option<String>, desc: "Synthesizer rust file to generate.";
		param input:Option<String>, desc: "Input plantuml synthesizer definition file.";
    }.parse_or_exit();

	let mut in_reader = BufReader::new(
		match &args.input {
			Some(ref x) => Box::new(File::open(&Path::new(x)).unwrap()) as Box<dyn Read>,
			None => Box::new(stdin()) as Box<dyn Read>,
		}
	);

	let mut fileContent: String = std::io::read_to_string(&mut in_reader)
		.expect("Unable to read input file.");


	let mut out_writer = BufWriter::new(
		match &args.output {
        	Some(ref x) => Box::new(File::create(&Path::new(x)).unwrap()) as Box<dyn Write>,
        	None => Box::new(stdout()) as Box<dyn Write>,
    	}
	);
    out_writer.write(fileContent.as_bytes()).unwrap();
}
