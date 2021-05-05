use dexparser::DexFile;
use log::{debug, info};

pub mod codegen;
mod parser;
mod std_env;

use codegen::Module;

/// Takes a reference to a DexFile as input and spits out a module, ready to be used.
pub fn process(input: &DexFile) -> Module {
	info!("Parsing dex input");

	let mut module = Module::new("undexed".to_owned());

	std_env::add_all(&mut module);

	for class in &input.classes {
		debug!("Generating class: {}", class.class_type);
		parser::parse_class(&class, &input, &mut module).unwrap();
	}

	module
}

//TODO `process` but append to an already existing module (useful for parsing multiple dex files)
pub fn process_and_append(_input: &DexFile, _module: &mut Module) {
	unimplemented!();
}
