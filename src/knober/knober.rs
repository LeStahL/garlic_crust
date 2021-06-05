#![feature(io_read_to_string)]
extern crate rustop;

extern crate pest;
#[macro_use]
extern crate pest_derive;

extern crate petgraph;

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
use petgraph::Graph;

struct Edge {
	from_node: String,
	from_attribute: String,
	to_attribute: String,
	to_node: String,
}

enum AttributeType {
	String,
	EnumEntry,
	Float,
	None,
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
	let mut name: String = String::new();

	let mut edges: Vec<Edge> = Vec::new();
	let mut nodes: Vec<Node> = Vec::new();

	for file_entry in parsed_synth_file.into_inner() {
		match file_entry.as_rule() {
			Rule::identifier => name = String::from(file_entry.as_str()),
			Rule::block => {
				// out_writer.write(String::from("Block:\n").as_bytes());
				for block_entry in file_entry.into_inner() {
					match block_entry.as_rule() {
						Rule::node => {
							let mut node_data_iterator = block_entry.into_inner();
							let block_name = String::from(node_data_iterator.next().unwrap().as_str());
							let block_type = String::from(node_data_iterator.next().unwrap().as_str());
							let mut attributes: Vec<Attribute> = Vec::new();
							
							// out_writer.write(&format!("Name: {}\n", &block_name).as_bytes());

							for attribute_entry in node_data_iterator {
								let mut attribute_data_iterator = attribute_entry.into_inner();
								let attribute_name = String::from(attribute_data_iterator.next().unwrap().as_str());

								let mut attribute_type = AttributeType::None;
								let mut attribute_value = String::new();

								let attribute_value_data = attribute_data_iterator.next().unwrap();
								match attribute_value_data.as_rule() {
									Rule::string => {
										attribute_type = AttributeType::String;
									},
									Rule::enum_entry => {
										attribute_type = AttributeType::EnumEntry;
									},
									Rule::float => {
										attribute_type = AttributeType::Float;
									},
									_ => ()
								}
								attribute_value = String::from(attribute_value_data.as_str());
								
								// out_writer.write(&format!("Attribute: {} - {}\n", &attribute_name, &attribute_value).as_bytes());

								attributes.push(Attribute{
									name: attribute_name,
									attribute_type: attribute_type,
										value: attribute_value,
								});
							}

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

	// Topological sorting of the nodes.
	let mut graph = Graph::<&Node, &Edge>::new();
	for node in Nodes {
		
	}

	// Write the resulting rust file.
	out_writer.write(&format!("Name: {}", name).as_bytes());
}
