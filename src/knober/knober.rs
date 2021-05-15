#![feature(io_read_to_string)]
extern crate rustop;

extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;

#[derive(Parser)]
#[grammar = "knober/plantuml_class.pest"]
struct PlantUmlClassParser;

use rustop::opts;
use std::io::*;
use std::fs::*;
use std::path::*;
use std::boxed::*;
use std::collections::*;

struct Edge {
	from_node: String,
	from_attribute: String,
	to_attribute: String,
	to_node: String,
}

enum AttributeType {
	QuotedFileName,
	EnumEntry,
	Float,
}

struct Attribute {
	attribute_type: AttributeType,
	name: String,
	value: String,
}

struct Node {
	name: String,
	node_type: String,
	attributes: Vec<Attribute>,
}

fn main() {
    let (args, rest) = opts! {
		auto_shorts true;
        synopsis "This is the command line synthesizer generator for garlic_crust by Team210.";
		opt output:Option<String>, desc: "Synthesizer rust file to generate.";
		param input:Option<String>, desc: "Input plantuml synthesizer definition file.";
    }.parse_or_exit();

	let mut in_reader = BufReader::new(
		match &args.input {
			Some(ref x) => Box::new(File::open(&Path::new(x)).unwrap()) as Box<dyn std::io::Read>,
			None => Box::new(stdin()) as Box<dyn std::io::Read>,
		}
	);

	let mut out_writer = BufWriter::new(
		match &args.output {
        	Some(ref x) => Box::new(File::create(&Path::new(x)).unwrap()) as Box<dyn std::io::Write>,
        	None => Box::new(stdout()) as Box<dyn std::io::Write>,
    	}
	);

	let mut file_content: String = std::io::read_to_string(&mut in_reader)
		.expect("Unable to read input file.");

	let parsed_synth_file = PlantUmlClassParser::parse(Rule::file, &file_content)
		.expect("Error parsing input file.")
		.next()
		.unwrap();

	// Parse the synth definition file.
	let mut name: String = String::from("");
	let mut edges: Vec<Edge> = Vec::new();
	let mut nodes: Vec<Node> = Vec::new();

	for entry in parsed_synth_file.into_inner() {
		match entry.as_rule() {
			Rule::identifier => name = String::from(entry.as_str()),
			Rule::block => {
				for block_entry in entry.into_inner() {
					match block_entry.as_rule() {
						Rule::node => {
							let mut node_data_iterator = block_entry.into_inner();
							let block_name = String::from(node_data_iterator.next().unwrap().as_str());
							let block_type = String::from(node_data_iterator.next().unwrap().as_str());
							let mut attributes: Vec<Attribute> = Vec::new();
							
							for attribute_entry in node_data_iterator {
								let mut attribute_name: String;
								let attribute_type: AttributeType;
								let attribute_value: String;

								match attribute_entry.as_rule() {
									Rule::identifier => {
										attribute_name = String::from(attribute_entry.as_str());
										out_writer.write(&format!("Name: {}\n", attribute_name).as_bytes());
									}
									Rule::attribute_value => {

									}
									_ => (),
								}

								// attributes.push(Attribute{
								// 	name: attribute_name,
								// 	attribute_type: attribute_type,
								// 	value: attribute_value,
								// });
							}
						// 	match node_data_iterator.next() {
						// 		Some(x) => {
						// 			for attribute_entry in x.into_inner() {
						// 				let mut attribute_data_iterator = attribute_entry.into_inner();
						// 				let attribute_name = String::from(attribute_data_iterator.next().unwrap().as_str());

						// 				out_writer.write(&format!("Attribute name: {}\n", attribute_name).as_bytes());

						// // 				let mut attribute_type: AttributeType;
						// // 				let mut attribute_value: String;

						// // 				match attribute_data_iterator.next().unwrap().as_rule() {
						// // 					Rule::quoted_file_name => {
						// // 						attribute_type = AttributeType::QuotedFileName;
						// // 					}
						// // 					Rule::enum_entry => {
						// // 						attribute_type = AttributeType::EnumEntry;
						// // 					}
						// // 					Rule::float => {
						// // 						attribute_type = AttributeType::Float;
						// // 					}
						// // 					_ => (),
						// // 				}
						// 			}
						// 		},
						// 		None => (),
						// 	}

							nodes.push(Node {
								name: block_name,
								node_type: block_type,
								attributes: attributes,
							});
						},
						Rule::edge => {
							let mut edge_data_iterator = block_entry.into_inner();
							edges.push(Edge {
								from_node: String::from(edge_data_iterator.next().unwrap().as_str()),
								from_attribute: String::from(edge_data_iterator.next().unwrap().as_str()),
								to_attribute: String::from(edge_data_iterator.next().unwrap().as_str()),
								to_node: String::from(edge_data_iterator.next().unwrap().as_str()),
							});
						}
						_ => (),
					}
				}
			}
			_ => (),
		}
	}

	// Write the resulting rust file.
	out_writer.write(&format!("Name: {}", name).as_bytes());
}
