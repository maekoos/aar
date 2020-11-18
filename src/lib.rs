use dex::Dex;
use log::{debug, info};
use memmap::Mmap;

pub mod generate;
pub mod parser;

pub fn parse_and_generate(input: &Dex<Mmap>) -> (String, String) {
  info!("Parsing dex input");
  let mut classes = vec![];
  for class in input.classes() {
    let class = class.expect("Class failed");
    debug!(
      "Parsing class: {}",
      class.jtype().type_descriptor().to_string(),
    );

    let c = parser::ASTClass::parse(&class, &input).unwrap();
    classes.push(c);
  }

  info!("Generating c from dex input");

  let mut g = generate::Generator::new();
  generate::generate_boilerplate(&mut g);
  generate::generate_definitions(&mut g, &input);

  for mut c in classes {
    generate::generate_class(&mut g, &mut c, &input).unwrap();
  }

  g.generate()
}
