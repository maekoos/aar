use log::debug;
use std::rc::Rc;

use super::{runtime, CallStack, Function, InvokeResult, Module, NativeFunction, RuntimeError};

pub fn add_functions(m: &mut Module) {
  m.add_function(
    "CLASS_java__lang__StringBuilder____init__".to_owned(),
    Function::Native(NativeFunction(java_lang_string_builder_init_)),
  );

  m.add_function(
    "CLASS_java__lang__StringBuilder__append".to_owned(),
    Function::Native(NativeFunction(java_lang_string_builder_append)),
  );

  m.add_function(
    "CLASS_java__lang__StringBuilder__toString".to_owned(),
    Function::Native(NativeFunction(java_lang_string_builder_to_string)),
  );
}

fn java_lang_string_builder_init_(
  params: Vec<runtime::Value>,
  cs: Rc<CallStack>,
  _env: &Module,
) -> InvokeResult {
  if params.len() != 1 {
    return InvokeResult::runtime(
      RuntimeError::Unimplemented(format!(
        "multiple init arguments ({}): {:?}",
        params.len(),
        params
      )),
      cs,
    );
  }

  let mut string_instance = match params[0].instance() {
    Err(e) => return InvokeResult::runtime(e, cs),
    Ok(i) => i.lock().unwrap(),
  };

  string_instance.set_class_type("java_lang_StringBuilder".to_owned());
  string_instance.set_field("value".to_owned(), runtime::Value::Array(Vec::new()));

  InvokeResult::Ok(runtime::Value::Void)
}

fn java_lang_string_builder_append(
  params: Vec<runtime::Value>,
  cs: Rc<CallStack>,
  _env: &Module,
) -> InvokeResult {
  //TODO if params.len() != 2

  let string_instance = match params[0].instance() {
    Err(e) => return InvokeResult::runtime(e, cs),
    Ok(i) => i.lock().unwrap(),
  };

  let to_append: Vec<char> = match params.get(1) {
    None => {
      return InvokeResult::runtime(
        RuntimeError::Unimplemented("no params to append function".to_owned()),
        cs,
      )
    }
    Some(v) => match v {
      runtime::Value::Char(v) => vec![*v],
      runtime::Value::U32(v) => format!("{}", v).chars().collect(),
      runtime::Value::Instance(i) => format!("{:?}", i.lock().unwrap()).chars().collect(),
      _ => {
        return InvokeResult::runtime(
          RuntimeError::Unimplemented(format!("Unimplemented append type: {:?}", v)),
          cs,
        );
      }
    },
  };

  debug!("STRINGBUILDER APPEND: {:?}", to_append);
  debug!(
    "STRINGBUILDER APPEND: string instance: {:?}",
    string_instance
  );

  InvokeResult::runtime(
    RuntimeError::Unimplemented("Java.lang.stringbuilder.append".to_owned()),
    cs,
  )
}

fn java_lang_string_builder_to_string(
  params: Vec<runtime::Value>,
  cs: Rc<CallStack>,
  _env: &Module,
) -> InvokeResult {
  debug!("STRINGBUILDER TO STRING: {:?}", params);

  InvokeResult::runtime(
    RuntimeError::Unimplemented("Java.lang.stringbuilder.to_string".to_owned()),
    cs,
  )
}
