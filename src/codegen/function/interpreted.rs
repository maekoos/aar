use std::collections::HashMap;
use std::rc::Rc;

use log::{debug, error, warn};

use super::instructions;
use super::instructions::Instruction;
use super::runtime;
use super::{CallStack, InvokeResult, RuntimeError};

pub type HandlerIdx = usize;

macro_rules! throw {
  ($e:expr, $instr:ident, $cur_exception:ident, $idx:ident, $labels:ident, $handlers:expr, $call_stack:ident) => {
    if let Some(handler_idx) = $instr.1 {
      debug!("Throw exception to handler {}", handler_idx);

      match $handlers.get(handler_idx) {
        None => unreachable!("handler idx not valid"),
        Some(handler) => {
          debug!("Handler: {:?}", handler);

          //TODO Check super classes
          match handler.get($e.get_class_type()) {
            None => unimplemented!(),
            Some(target) => {
              debug!("Found a suitable handler: {}", target);

              $cur_exception = Some($e);
              $idx = match $labels.get(target) {
                None => {
                  unreachable!("The handler's jump target has no label...");
                  // return InvokeResult::runtime(
                  //   RuntimeError::BadJumpTarget,
                  //   call_stack,
                  // );
                }
                Some(index) => *index,
              }
            }
          };
        }
      }
    } else {
      return InvokeResult::Exception($e, $call_stack);
    }
  };
}

#[derive(Debug)]
pub struct InterpretedFunction {
  n_regs: usize,
  n_params: usize,
  return_: bool,
  instructions: Vec<(Instruction, Option<HandlerIdx>)>,
  handlers: Vec<HashMap<String, usize>>,
}

impl InterpretedFunction {
  pub fn new(
    n_regs: usize,
    n_params: usize,
    return_: bool,
    instructions: Vec<(Instruction, Option<HandlerIdx>)>,
    handlers: Vec<HashMap<String, usize>>,
  ) -> Self {
    Self {
      n_regs,
      n_params,
      return_,
      instructions,
      handlers,
    }
  }

  pub fn n_params(&self) -> usize {
    self.n_params
  }

  pub fn n_regs(&self) -> usize {
    self.n_regs
  }

  pub fn return_(&self) -> bool {
    self.return_
  }

  pub fn instructions(&self) -> &Vec<(Instruction, Option<HandlerIdx>)> {
    &self.instructions
  }

  pub fn handlers(&self) -> &Vec<HashMap<String, usize>> {
    &self.handlers
  }

  /// Run this function using the interpreter
  pub fn run_interpreted(
    &self,
    parameters: Vec<runtime::Value>,
    call_stack: Rc<CallStack>,
    env: &super::Module,
  ) -> InvokeResult {
    let mut registers: Vec<runtime::Value> = vec![runtime::Value::Void; self.n_regs];

    if parameters.len() != self.n_params {
      return InvokeResult::runtime(
        RuntimeError::WrongNumberOfParameters(self.n_params, parameters.len()),
        call_stack,
      );
    }
    for i in (self.n_regs - self.n_params)..self.n_regs {
      registers[i] = parameters[i - (self.n_regs - self.n_params)].to_owned();
    }
    // let parameters = &registers[self.n_regs - self.n_params..];

    let mut labels: HashMap<usize, usize> = HashMap::new();
    for (i, instr) in self.instructions.iter().enumerate() {
      match &instr.0 {
        Instruction::Label(id) => {
          labels.insert(*id, i);
        }
        _ => {}
      }
    }

    let mut return_value: runtime::Value = runtime::Value::Void;
    let mut cur_exception: Option<runtime::Instance> = None;
    let mut i = 0;
    while i < self.instructions.len() {
      let instr = &self.instructions[i];
      i += 1;

      match &instr.0 {
        Instruction::Label(_) => {}
        Instruction::Nop => {}
        // Instruction::Move
        Instruction::MoveResult(kind, v) => {
          let dest = match registers.get_mut(*v as usize) {
            None => {
              return InvokeResult::runtime(RuntimeError::RegisterOutOfBounds, call_stack);
            }
            Some(v) => v,
          };

          match kind {
            instructions::MoveKind::Single => match return_value {
              runtime::Value::U32(_) => {
                *dest = return_value;
                return_value = runtime::Value::Void;
              }
              _ => todo!("Cast error"),
            },
            instructions::MoveKind::Wide => unimplemented!(),
            instructions::MoveKind::Object => match return_value {
              runtime::Value::Instance(_) => {
                *dest = return_value;
                return_value = runtime::Value::Void;
              }
              rv => {
                error!("Error originated in move-result");
                return InvokeResult::runtime(
                  RuntimeError::CastError(format!("{:?} as Instance", rv)),
                  call_stack,
                );
              }
            },
          };
        }
        Instruction::MoveException(v_dest) => {
          if let Some(e) = cur_exception {
            let dest = match registers.get_mut(*v_dest as usize) {
              None => {
                return InvokeResult::runtime(RuntimeError::RegisterOutOfBounds, call_stack);
              }
              Some(v) => v,
            };

            *dest = runtime::Value::from_instance(e);
            cur_exception = None;
          } else {
            todo!("No exception to move");
          }
        }
        Instruction::Return(ty) => match ty {
          instructions::ReturnType::Void => return InvokeResult::Ok(runtime::Value::Void),
          instructions::ReturnType::Single(v) => {
            return InvokeResult::Ok(runtime::Value::U32(match registers.get(*v) {
              None => {
                return InvokeResult::runtime(RuntimeError::RegisterOutOfBounds, call_stack);
              }
              Some(v) => v.to_single(),
            }))
          }
          instructions::ReturnType::Wide(_) => unimplemented!(),
          instructions::ReturnType::Object(v_o) => match registers.get(*v_o) {
            None => {
              return InvokeResult::runtime(RuntimeError::RegisterOutOfBounds, call_stack);
            }
            Some(o) => return InvokeResult::Ok(o.clone()),
          },
        },
        Instruction::ConstSet(v, lit) => {
          let dest = match registers.get_mut(*v) {
            None => {
              return InvokeResult::runtime(RuntimeError::RegisterOutOfBounds, call_stack);
            }
            Some(v) => v,
          };

          *dest = runtime::Value::from(lit);
        }
        Instruction::GoTo(label) => {
          i = match labels.get(label) {
            None => {
              return InvokeResult::runtime(RuntimeError::BadJumpTarget, call_stack);
            }
            Some(index) => *index,
          }
        }
        //? Do we actually need the type_idx?
        Instruction::NewInstance(v, _ty) => match registers.get_mut(*v as usize) {
          None => {
            return InvokeResult::runtime(RuntimeError::RegisterOutOfBounds, call_stack);
          }
          Some(v) => *v = runtime::Value::new_instance(),
        },
        //? Do we actually need the type_idx?
        Instruction::NewArray(v, size, _ty) => match registers.get_mut(*v as usize) {
          None => {
            return InvokeResult::runtime(RuntimeError::RegisterOutOfBounds, call_stack);
          }
          Some(v) => {
            *v = runtime::Value::Array(Vec::with_capacity(*size));
          }
        },
        Instruction::FillArrayData(v, data) => match registers.get_mut(*v as usize) {
          None => return InvokeResult::runtime(RuntimeError::RegisterOutOfBounds, call_stack),
          Some(r) => *r = runtime::Value::Array(data.clone()),
        },
        Instruction::If(kind, v1, v2, label_id) => {
          let v1 = match registers.get(*v1 as usize) {
            None => return InvokeResult::runtime(RuntimeError::RegisterOutOfBounds, call_stack),
            Some(v) => v,
          };
          let v2 = match registers.get(*v2 as usize) {
            None => return InvokeResult::runtime(RuntimeError::RegisterOutOfBounds, call_stack),
            Some(v) => v,
          };

          let cond = match kind {
            instructions::IfKind::Eq => unimplemented!(),
            instructions::IfKind::Ne => match v1 {
              runtime::Value::U32(a) => match v2 {
                runtime::Value::U32(b) => a != b,
                _ => unimplemented!(),
              },
              _ => unimplemented!(),
            },
            instructions::IfKind::Lt => unimplemented!(),
            instructions::IfKind::Ge => match v1 {
              runtime::Value::U32(a) => match v2 {
                runtime::Value::U32(b) => a >= b,
                _ => unimplemented!(),
              },
              _ => unimplemented!(),
            },
            instructions::IfKind::Gt => unimplemented!(),
            instructions::IfKind::Le => unimplemented!(),
          };

          if cond {
            i = match labels.get(label_id) {
              None => {
                return InvokeResult::runtime(RuntimeError::BadJumpTarget, call_stack);
              }
              Some(index) => *index,
            }
          }
        }
        Instruction::ArrayGet(kind, v_dest, v_arr, v_idx) => {
          let idx = match registers.get(*v_idx as usize) {
            None => return InvokeResult::runtime(RuntimeError::RegisterOutOfBounds, call_stack),
            Some(idx) => {
              let i = match idx {
                //TODO If v is negative?
                runtime::Value::U32(v) => *v as u32 as usize,
                v => {
                  return InvokeResult::runtime(
                    RuntimeError::CastError(format!("{:?} as index", v)),
                    call_stack,
                  )
                }
              };

              i
            }
          };
          let array = match registers.get(*v_arr as usize) {
            None => return InvokeResult::runtime(RuntimeError::RegisterOutOfBounds, call_stack),
            Some(arr) => match arr {
              runtime::Value::Array(a) => a,
              v => {
                return InvokeResult::runtime(
                  RuntimeError::CastError(format!("{:?} as array", v)),
                  call_stack,
                )
              }
            },
          };

          match kind {
            instructions::GetPutKind::Single | instructions::GetPutKind::Boolean => {
              let value = match array.get(idx) {
                None => todo!("Exception..."),
                Some(v) => v.to_owned(),
              };

              match registers.get_mut(*v_dest as usize) {
                None => {
                  return InvokeResult::runtime(RuntimeError::RegisterOutOfBounds, call_stack)
                }
                Some(v) => {
                  *v = value;
                }
              }
            }
            _ => unimplemented!(),
          }
        }
        Instruction::InstanceGet(kind, v_dest, v_obj, field) => {
          let obj = match registers.get(*v_obj as usize) {
            None => {
              return InvokeResult::runtime(RuntimeError::RegisterOutOfBounds, call_stack);
            }
            Some(o) => o,
          };

          let oi = match obj.instance() {
            Ok(v) => v,
            Err(e) => {
              error!("Error originated in instance-get");
              return InvokeResult::runtime(e, call_stack);
            }
          };

          let field_value = match kind {
            instructions::GetPutKind::Single => {
              let field_value = match oi.lock().unwrap().get_field(field) {
                None => {
                  warn!("Access to un-set field");
                  runtime::Value::Void
                }
                Some(v) => v.clone(),
              };

              if !field_value.is_u32() {
                warn!("Ignoring cast-error while running instance get - single");
              }

              field_value
            }
            instructions::GetPutKind::Wide => unimplemented!(),
            instructions::GetPutKind::Object => {
              let field_value = match oi.lock().unwrap().get_field(field) {
                None => {
                  warn!("Access to un-set field");
                  runtime::Value::Void
                }
                Some(v) => v.clone(),
              };

              if !field_value.is_instance() {
                warn!("Ignoring cast-error while running instance get - object");
              }

              field_value
            }
            instructions::GetPutKind::Boolean => unimplemented!(),
            instructions::GetPutKind::Byte => unimplemented!(),
            instructions::GetPutKind::Char => unimplemented!(),
            instructions::GetPutKind::Short => unimplemented!(),
          };

          let dest = match registers.get_mut(*v_dest as usize) {
            None => {
              return InvokeResult::runtime(RuntimeError::RegisterOutOfBounds, call_stack);
            }
            Some(o) => o,
          };

          *dest = field_value;
        }
        Instruction::InstancePut(kind, v_src, v_obj, field) => {
          let ob = match registers.get(*v_obj as usize) {
            None => return InvokeResult::runtime(RuntimeError::RegisterOutOfBounds, call_stack),
            Some(o) => {
              if !o.is_instance() {
                error!("Error originated in instance-put");
                return InvokeResult::runtime(
                  RuntimeError::CastError(format!("{:?} as instance", o)),
                  call_stack,
                );
              }

              match o.instance() {
                Ok(v) => v,
                Err(e) => {
                  error!("Error originated in instance-put");
                  return InvokeResult::runtime(e, call_stack);
                }
              }
            }
          };
          let src = match registers.get(*v_src as usize) {
            None => return InvokeResult::runtime(RuntimeError::RegisterOutOfBounds, call_stack),
            Some(o) => o,
          };

          match kind {
            instructions::GetPutKind::Single => {
              if !src.is_u32() {
                warn!("Ignoring cast error! (-> single)");
              }

              let cloned = src.clone();

              {
                let mut ob = ob.lock().unwrap();
                ob.set_field(field.to_owned(), cloned);
              }
            }
            instructions::GetPutKind::Wide => unimplemented!(),
            instructions::GetPutKind::Object => {
              if !src.is_instance() {
                warn!("Ignoring cast error! (-> instance)");
                // return Err(RuntimeError::CastError(format!("{:?} as instance", o)));
              }

              // Clone the Arc
              let cloned = src.clone();

              {
                let mut ob = ob.lock().unwrap();
                ob.set_field(field.to_owned(), cloned);
              }
            }
            instructions::GetPutKind::Boolean => unimplemented!(),
            instructions::GetPutKind::Byte => unimplemented!(),
            instructions::GetPutKind::Char => unimplemented!(),
            instructions::GetPutKind::Short => unimplemented!(),
          };
        }
        Instruction::StaticGet(kind, v_dest, name) => {
          let s_value = match env.get_static(name) {
            None => {
              warn!("Could not find the specified static variable: {}", name);
              runtime::Value::Void
            }
            Some(v) => v,
          };
          let dest = match registers.get_mut(*v_dest as usize) {
            None => return InvokeResult::runtime(RuntimeError::RegisterOutOfBounds, call_stack),
            Some(v) => v,
          };

          match kind {
            instructions::GetPutKind::Single => *dest = s_value,
            instructions::GetPutKind::Object => *dest = s_value,
            a => unimplemented!("{:?}", a),
          }
        }

        Instruction::StaticPut(kind, v_src, name) => {
          let src = match registers.get(*v_src as usize) {
            None => return InvokeResult::runtime(RuntimeError::RegisterOutOfBounds, call_stack),
            Some(v) => v.clone(),
          };

          match kind {
            instructions::GetPutKind::Single => env.set_static(name.to_owned(), src),
            a => unimplemented!("{:?}", a),
          }
        }
        Instruction::Invoke(kind, name, argc, args) => match kind {
          instructions::InvokeKind::Direct
          | instructions::InvokeKind::Static
          | instructions::InvokeKind::Virtual => {
            let mut a = Vec::new();
            for i in 0..*argc as usize {
              a.push(match registers.get(args[i] as usize) {
                None => {
                  return InvokeResult::runtime(RuntimeError::RegisterOutOfBounds, call_stack)
                }
                Some(v) => v.clone(),
              })
            }

            let cs = Rc::new(CallStack::extend(name.to_owned(), call_stack.clone()));
            let rv = env.invoke(name, cs, a);
            match rv {
              InvokeResult::Ok(v) => {
                return_value = v;
              }
              InvokeResult::Exception(e, cs) => {
                throw!(e, instr, cur_exception, i, labels, &self.handlers, cs);
              }
              // RuntimeError:
              e => return e,
            }
          }
          _ => unimplemented!(),
        },
        Instruction::BinOp2Addr(kind, v_dest, v_src) => match kind {
          instructions::BinOpKind::AddInt => {
            let dest_val = match registers.get(*v_dest as usize) {
              None => return InvokeResult::runtime(RuntimeError::RegisterOutOfBounds, call_stack),
              Some(d) => d.to_single(),
            };
            let src_val = match registers.get(*v_src as usize) {
              None => return InvokeResult::runtime(RuntimeError::RegisterOutOfBounds, call_stack),
              Some(s) => s.to_single(),
            };

            match registers.get_mut(*v_dest as usize) {
              None => return InvokeResult::runtime(RuntimeError::RegisterOutOfBounds, call_stack),
              Some(dest) => *dest = runtime::Value::U32(dest_val + src_val),
            }
          }
          instructions::BinOpKind::SubInt => unimplemented!(),
          instructions::BinOpKind::MulInt => unimplemented!(),
          instructions::BinOpKind::DivInt => unimplemented!(),
          instructions::BinOpKind::RemInt => unimplemented!(),
          instructions::BinOpKind::AndInt => unimplemented!(),
          instructions::BinOpKind::OrInt => unimplemented!(),
          instructions::BinOpKind::XorInt => unimplemented!(),
          instructions::BinOpKind::ShlInt => unimplemented!(),
          instructions::BinOpKind::ShrInt => unimplemented!(),
          instructions::BinOpKind::UshrInt => unimplemented!(),
          instructions::BinOpKind::AddLong => unimplemented!(),
          instructions::BinOpKind::SubLong => unimplemented!(),
          instructions::BinOpKind::MulLong => unimplemented!(),
          instructions::BinOpKind::DivLong => unimplemented!(),
          instructions::BinOpKind::RemLong => unimplemented!(),
          instructions::BinOpKind::AndLong => unimplemented!(),
          instructions::BinOpKind::OrLong => unimplemented!(),
          instructions::BinOpKind::XorLong => unimplemented!(),
          instructions::BinOpKind::ShlLong => unimplemented!(),
          instructions::BinOpKind::ShrLong => unimplemented!(),
          instructions::BinOpKind::UshrLong => unimplemented!(),
          instructions::BinOpKind::AddFloat => unimplemented!(),
          instructions::BinOpKind::SubFloat => unimplemented!(),
          instructions::BinOpKind::MulFloat => unimplemented!(),
          instructions::BinOpKind::DivFloat => unimplemented!(),
          instructions::BinOpKind::RemFloat => unimplemented!(),
          instructions::BinOpKind::AddDouble => unimplemented!(),
          instructions::BinOpKind::SubDouble => unimplemented!(),
          instructions::BinOpKind::MulDouble => unimplemented!(),
          instructions::BinOpKind::DivDouble => unimplemented!(),
          instructions::BinOpKind::RemDouble => unimplemented!(),
        },
        Instruction::BinOpLit(kind, v_dest, v_src, lit) => {
          match kind {
            instructions::BinOpLitKind::AddInt => {
              let src = match registers.get(*v_src as usize) {
                None => {
                  return InvokeResult::runtime(RuntimeError::RegisterOutOfBounds, call_stack)
                }
                Some(v) => v.to_single(),
              };

              let dest = match registers.get_mut(*v_dest as usize) {
                None => {
                  return InvokeResult::runtime(RuntimeError::RegisterOutOfBounds, call_stack)
                }
                Some(v) => v,
              };

              *dest = runtime::Value::U32(src + *lit as i32);
            }
            instructions::BinOpLitKind::DivInt => {
              let src = match registers.get(*v_src as usize) {
                None => {
                  return InvokeResult::runtime(RuntimeError::RegisterOutOfBounds, call_stack)
                }
                Some(v) => v.to_single(),
              };
              if *lit == 0 {
                let mut e = runtime::Instance::default();
                e.set_class_type("Ljava/lang/ArithmeticException;".to_owned());
                e.set_field(
                  "value".to_owned(),
                  runtime::Value::from(&instructions::LiteralValue::String("/ by zero".to_owned())),
                );

                throw!(
                  e,
                  instr,
                  cur_exception,
                  i,
                  labels,
                  &self.handlers,
                  call_stack
                );
              } else {
                let dest = match registers.get_mut(*v_dest as usize) {
                  None => {
                    return InvokeResult::runtime(RuntimeError::RegisterOutOfBounds, call_stack)
                  }
                  Some(v) => v,
                };

                *dest = runtime::Value::U32(src / *lit as i32)
              }
            }
            // instructions::BinOpLitKind::RsubInt => {}
            // instructions::BinOpLitKind::MulInt => {}
            // instructions::BinOpLitKind::DivInt => {}
            // instructions::BinOpLitKind::RemInt => {}
            // instructions::BinOpLitKind::AndInt => {}
            // instructions::BinOpLitKind::OrInt => {}
            // instructions::BinOpLitKind::XorInt => {}
            a => unimplemented!("BinOpLit: {:?}", a),
          }
        }
        a => unimplemented!("Interp. {:?}", a),
      }
    }

    InvokeResult::Ok(runtime::Value::Void)
  }

  /// Convert this function to a readable format
  pub fn build_ir(&self, name: &str) -> String {
    let mut out = Vec::new();
    out.push(format!("func {:?}", name));
    out.push(format!("\tRegisters (total): {}", self.n_regs));
    out.push(format!("\t       Parameters: {}", self.n_params));
    out.push(format!("\t Has return value: {}", self.return_));
    out.push(format!(""));

    for i in &self.instructions {
      out.push(format!("\t{:?}", i));
    }

    out.join("\n")
  }
}
