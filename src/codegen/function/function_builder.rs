use std::collections::HashMap;

use super::instructions::*;
use super::interpreted::*;
use super::runtime;
use super::Function;

use log::debug;

pub struct FunctionBuilder {
  n_regs: usize,
  n_params: usize,
  return_: bool,
  instructions: Vec<(Instruction, Option<HandlerIdx>)>,
  next_handler: Option<usize>,
  handlers: Vec<HashMap<String, usize>>,
}

impl FunctionBuilder {
  pub fn new() -> Self {
    Self {
      n_regs: 0,
      n_params: 0,
      return_: false,
      instructions: Vec::new(),
      next_handler: None,
      handlers: Vec::new(),
    }
  }

  pub fn build(self) -> Function {
    Function::Interpreted(InterpretedFunction::new(
      self.n_regs,
      self.n_params,
      self.return_,
      self.instructions,
      self.handlers,
    ))
  }

  pub fn set_handlers(&mut self, new_handlers: Vec<HashMap<String, usize>>) {
    self.handlers = new_handlers;
  }

  pub fn set_next_handler(&mut self, handler: Option<usize>) {
    self.next_handler = handler;
  }

  pub fn set_n_regs(&mut self, new_n_regs: usize) {
    self.n_regs = new_n_regs;
  }

  pub fn set_n_params(&mut self, new_n_params: usize) {
    self.n_params = new_n_params;
  }

  pub fn set_return(&mut self, new_return: bool) {
    self.return_ = new_return;
  }

  fn push_instruction(&mut self, ins: Instruction) {
    self.instructions.push((ins, self.next_handler));
    self.next_handler = None;
  }

  pub fn label(&mut self, id: usize) {
    self.push_instruction(Instruction::Label(id))
  }

  pub fn nop(&mut self) {
    self.push_instruction(Instruction::Nop);
  }

  /// Move from one register to another
  pub fn move_v(&mut self, _kind: MoveKind, _va: u16, _vb: u16) {
    todo!();
  }

  /// Move the results to a register
  pub fn move_results(&mut self, kind: MoveKind, v: usize) {
    self.push_instruction(Instruction::MoveResult(kind, v));
  }

  /// Move exception to a register
  pub fn move_exception(&mut self, v: usize) {
    self.push_instruction(Instruction::MoveException(v));
  }

  /// Return a void or register-value
  pub fn return_v(&mut self, ty: ReturnType) {
    self.push_instruction(Instruction::Return(ty));
  }

  /// Move the literal value into the specified register
  pub fn const_set(&mut self, v: usize, lit: LiteralValue) {
    self.push_instruction(Instruction::ConstSet(v, lit))
  }

  /// Monitor enter and exit?
  pub fn monitor(&mut self) {
    todo!();
  }

  /// "Throw a ClassCastException if the reference in the given register cannot be cast to the indicated type."
  pub fn check_cast(&mut self) {
    todo!();
  }

  /// Store in the given destination register 1 if the indicated reference is an instance of the given type, or 0 if not.
  pub fn instance_of(&mut self, _va: u8, _vb: u8, _type_: TypeIndex) {
    todo!();
  }

  /// Store in the given destination register the length of the indicated array, in entries
  pub fn array_length(&mut self, _v_dest: u8, _v_arr: u8) {
    todo!();
  }

  /// Construct a new instance of the indicated type, storing a reference to it in the destination. The type must refer to a non-array class.
  pub fn new_instance(&mut self, v_dest: u8, type_: TypeIndex) {
    self.push_instruction(Instruction::NewInstance(v_dest, type_));
  }

  /// Construct a new array of the indicated type and size. The type must be an array type.
  pub fn new_array(&mut self, v_dest: u8, v_size: usize, type_: TypeIndex) {
    self.push_instruction(Instruction::NewArray(v_dest, v_size, type_));
  }

  /// Filled new array
  pub fn filled_new_array(&mut self) {
    todo!();
  }

  /// Filled new array (/range)
  pub fn filled_new_array_range(&mut self) {
    todo!();
  }

  /// Fill array data
  pub fn fill_array_data(&mut self, arr_v: usize, el_width: usize, data: Vec<u8>) {
    if el_width > 4 {
      unimplemented!("Parsing of raw data with el_width > 4");
    }

    let data = match el_width {
      0 => panic!("Array data with el_width 0 not possible (?)"),
      1 => data
        .iter()
        //TODO: test this..
        .map(|x| runtime::Value::U32(*x as i32))
        .collect(),
      2 => {
        unimplemented!();
        // if data.len() % 2 != 0 {
        //   panic!("Array data is uneven (2)");
        // }

        // let mut out = Vec::new();
        // let mut cur: i16 = 0;

        // for i in 0..data.len() {
        //   if i % 2 == 0 {
        //     cur = (data[i] as i16) << 8;
        //     continue;
        //   }

        //   cur |= data[i] as i16;
        //   out.push(runtime::Value::U32(cur as i32));
        // }

        // out
      }
      3 => unimplemented!(),
      4 => {
        //TODO test this...

        debug!("Data: {:?}", data);
        if data.len() % 4 != 0 {
          panic!("Array data is uneven (4)");
        }

        let mut out = Vec::new();

        for i in 0..data.len() / 4 {
          //TODO Check the endianness?
          out.push(runtime::Value::U32(i32::from_le_bytes([
            data[i * 4],
            data[i * 4 + 1],
            data[i * 4 + 2],
            data[i * 4 + 3],
          ])));
        }

        out
      }
      _ => unreachable!(),
    };
    debug!("Data: {:?}", data);
    debug!("El width: {:?}", el_width);

    self.push_instruction(Instruction::FillArrayData(arr_v, data));
  }

  /// Throw the indicated exception
  pub fn throw(&mut self, _v_ex: u8) {
    todo!();
  }

  /// Goto the specified label
  pub fn goto(&mut self, label: usize) {
    self.push_instruction(Instruction::GoTo(label));
  }

  /// Switch statement
  pub fn switch(&mut self) {
    todo!();
  }

  /// Perform the indicated floating point or long comparison
  pub fn cmp_kind(&mut self, _v_dest: u8, _v_b: u8, _v_c: u8, _kind: CmpKind) {
    todo!();
  }

  /// Branch to the given destination if the given two registers' values compare as specified.
  pub fn if_test(&mut self, kind: IfKind, v_first: u8, v_second: u8, target_label: usize) {
    self.push_instruction(Instruction::If(kind, v_first, v_second, target_label));
  }

  /// Branch to the given destination if the given two registers' values compare as specified.
  pub fn ifz_test(&mut self, _v_test: u8, _target_label: usize) {
    todo!();
  }

  /// Perform the identified array operation at the identified index of the given array, storing into the dest register.
  pub fn array_get(&mut self, kind: GetPutKind, v_dest: u8, v_arr: u8, v_idx: u8) {
    self.push_instruction(Instruction::ArrayGet(kind, v_dest, v_arr, v_idx));
  }

  /// Perform the identified array operation at the identified index of the given array, loading from the src register.
  pub fn array_put(&mut self, _v_src: u8, _v_arr: u8, _v_idx: u8, _kind: GetPutKind) {
    todo!();
  }

  /// Perform the identified object instance field operation with the identified field, storing into the dest register.
  pub fn instance_get(&mut self, kind: GetPutKind, v_dest: u8, v_inst: u8, field: String) {
    self.push_instruction(Instruction::InstanceGet(kind, v_dest, v_inst, field));
  }

  /// Perform the identified object instance field operation with the identified field, loading from the src register.
  pub fn instance_put(&mut self, kind: GetPutKind, v_src: u8, v_inst: u8, field_ref: String) {
    self.push_instruction(Instruction::InstancePut(kind, v_src, v_inst, field_ref));
  }

  /// Perform the identified object static field operation with the identified static field, storing into the dest register.
  pub fn static_get(&mut self, kind: GetPutKind, v_dest: u8, static_name: String) {
    self.push_instruction(Instruction::StaticGet(kind, v_dest, static_name));
  }

  /// Perform the identified object static field operation with the identified static field, loading from the src register.
  pub fn static_put(&mut self, kind: GetPutKind, v_src: u8, static_idx: String) {
    self.push_instruction(Instruction::StaticPut(kind, v_src, static_idx));
  }

  /// Call the indicated method
  pub fn invoke(&mut self, kind: InvokeKind, method: String, argc: u8, args: [u8; 5]) {
    self.push_instruction(Instruction::Invoke(kind, method, argc, args));
  }

  /// Call the indicated method
  pub fn invoke_range(&mut self, _kind: InvokeKind, _method: u16, _argc: u8, _v_arg1: u16) {
    todo!();
  }

  /// Perform the identified unary operation on the source register, storing the result in the destination register.
  pub fn un_op(&mut self, _kind: UnOpKind, _v_dest: u8, _v_src: u8) {
    todo!();
  }

  /// Perform the identified binary operation on the two source registers, storing the result in the destination register.
  pub fn bin_op(&mut self, _kind: BinOpKind, _v_dest: u8, _v_src_a: u8, _v_src_b: u8) {
    todo!();
  }

  /// Perform the identified binary operation on the two source registers, storing the result in the destination register.
  pub fn bin_op_2_addr(&mut self, kind: BinOpKind, v_dest_and_src_a: u8, v_src_b: u8) {
    self.push_instruction(Instruction::BinOp2Addr(kind, v_dest_and_src_a, v_src_b));
  }

  /// Perform the indicated binary op on the src register and literal value, storing the result in the destination register.
  pub fn bin_op_lit(&mut self, kind: BinOpLitKind, v_dest: u8, v_src: u8, lit: i16) {
    self.push_instruction(Instruction::BinOpLit(kind, v_dest, v_src, lit));
  }

  /*
  /// invoke-polymorphic
  pub fn invoke_polymorphic(&mut self) {
    todo!();
  }

  /// invoke-polymorphic/range
  pub fn invoke_polymorphic_range(&mut self) {
    todo!();
  }

  /// invoke-custom
  pub fn invoke_custom(&mut self) {
    todo!();
  }

  /// invoke-custom/range
  pub fn invoke_custom_range(&mut self) {
    todo!();
  }

  pub fn const_method_handle(&mut self) {
    todo!();
  }

  pub fn const_method_type(&mut self) {
    todo!();
  }
  */
}
