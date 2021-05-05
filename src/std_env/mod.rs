use log::{debug, error, info};
use std::rc::Rc;

use super::codegen::{
  function::{CallStack, Function, NativeFunction},
  runtime, InvokeResult, RuntimeError,
};
use super::Module;

mod string_builder;

fn java_lang_object_init(
  _params: Vec<runtime::Value>,
  _cs: Rc<CallStack>,
  _env: &Module,
) -> InvokeResult {
  InvokeResult::Ok(runtime::Value::Void)
}

fn java_io_print_stream_println(
  params: Vec<runtime::Value>,
  _cs: Rc<CallStack>,
  _env: &Module,
) -> InvokeResult {
  debug!("Println: {:?}", params);
  if params.len() == 2 {
    match &params[1] {
      runtime::Value::U32(v) => println!("{}", v),
      runtime::Value::Instance(v) => {
        let i = v.lock().unwrap();
        if i.get_class_type() == "java_lang_string" {
          let a = match i.get_field("data") {
            None => unimplemented!(),
            Some(v) => match v {
              runtime::Value::Array(arr) => arr,
              _ => unimplemented!(),
            },
          };

          let string = a
            .iter()
            .map(|x| match x {
              runtime::Value::Char(v) => *v,
              _ => unimplemented!(),
            })
            .collect::<String>();

          println!("{}", string);
        } else {
          println!("{:?}", i);
        }
      }
      _ => println!("{:?}", params[1]),
    }
  } else {
    error!(
      "Unsupported number of parameters in print stream println: {}",
      params.len()
    );
  }

  InvokeResult::Ok(runtime::Value::Void)
}

pub fn add_all(m: &mut Module) {
  info!("Initializing java environment");

  m.add_static("CLASS_java__lang__System__out".to_owned());

  m.add_function(
    "CLASS_java__lang__Object____init__".to_owned(),
    Function::Native(NativeFunction(java_lang_object_init)),
  );

  m.add_function(
    "CLASS_java__io__PrintStream__println".to_owned(),
    Function::Native(NativeFunction(java_io_print_stream_println)),
  );

  string_builder::add_functions(m);

  // "CLASS_java__lang__Object____init__" => Ok(runtime::Value::Void),
  // "CLASS_java__io__PrintStream__println" => {
  // 	println!("Printstream: {:?}", params);
  // 	Ok(runtime::Value::Void)
  // }
}
