use dex::DexReader;
use log::info;
use std::fs::File;
use std::io;
use std::io::prelude::*;

use aar::parse_and_generate;

fn main() -> io::Result<()> {
  env_logger::init();

  let p = "resources/MyCode/classes.dex";

  info!("Reading dex file: {}", &p);
  let input = DexReader::from_file(p).unwrap();

  let (c_file, h_file) = parse_and_generate(&input);

  let mut file = File::create("out/out.c")?;
  file.write_all(c_file.as_bytes())?;
  let mut file = File::create("out/out.h")?;
  file.write_all(h_file.as_bytes())?;

  info!("Output saved");

  Ok(())
}
