//TODO Rewrite this to allow for variables etc and become more of a cli-tool
// aar input.dex --main MyClass.main -- argument1 argument 2

use log::{info, warn};
use std::fs::{self, File};
use std::io;
use std::io::prelude::*;

use aar::codegen::runtime::{InvokeResult, Value};
use aar::process;

fn main() -> io::Result<()> {
  env_logger::init();

  let path = "resources/MyCode/classes.dex";

  info!("Reading dex file: {}", &path);
  let mut file = File::open(path).unwrap();
  let mut bytes = Vec::new();
  file.read_to_end(&mut bytes)?;

  //TODO Warn about overwriting a directory?
  match fs::create_dir_all("out/analysis") {
    Ok(_) => info!("Created directory 'out'"),
    Err(e) => warn!("Could not create an output directory: {:?}", e),
  }

  match dexparser::parse(&bytes) {
    Ok(res) => {
      let mut module = process(&res);

      let out = module.build_ir();
      let mut file = File::create("out/out")?;
      file.write_all(out.as_bytes())?;
      info!("IR Output saved");

      let res = module.run("CLASS_MyCode__main", vec![Value::Array(Vec::new())]);
      match res {
        InvokeResult::Ok(v) => info!("Return value: {:?}", v),
        InvokeResult::Exception(e, cs) => println!("Exception: {:?}\n{}", e, cs.finalize()),
        InvokeResult::RuntimeError(e) => {
          let f = e.finalize();
          println!("{}", f);
        }
      }
    }

    Err(e) => panic!("Failed to parse dex file: {}", e),
  }

  Ok(())
}
