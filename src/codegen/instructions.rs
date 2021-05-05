use super::runtime;

pub type TypeIndex = usize;
pub type StaticIndex = u16;
// pub type MethodIndex = u16;
pub type FieldRefIndex = u16;

/// IRInstructions - later passed to selected backend
#[derive(Debug)]
pub enum Instruction {
  Label(usize),
  Nop,
  /// Move from .1 to .2
  Move(MoveKind, usize, usize),
  /// Move result to .1
  MoveResult(MoveKind, usize),
  /// Move exception to .0
  MoveException(usize),
  Return(ReturnType),
  ConstSet(usize, LiteralValue),
  NewInstance(u8, usize),
  NewArray(u8, usize, TypeIndex),
  FillArrayData(usize, Vec<runtime::Value>),
  GoTo(usize),
  If(IfKind, u8, u8, usize),
  /// Get from array (kind, v_dest, v_arr, v_idx)
  ArrayGet(GetPutKind, u8, u8, u8),
  StaticGet(GetPutKind, u8, String),
  StaticPut(GetPutKind, u8, String),
  InstanceGet(GetPutKind, u8, u8, String),
  InstancePut(GetPutKind, u8, u8, String),
  Invoke(InvokeKind, String, u8, [u8; 5]),
  BinOp2Addr(BinOpKind, u8, u8),
  BinOpLit(BinOpLitKind, u8, u8, i16),
}

#[derive(Debug)]
pub enum IfKind {
  Eq,
  Ne,
  Lt,
  Ge,
  Gt,
  Le,
}

#[derive(Debug)]
pub enum UnOpKind {
  NegInt,
  NotInt,
  NegLong,
  NotLong,
  NegFloat,
  NegDouble,
  IntToLong,
  IntToFloat,
  IntToDouble,
  LongToInt,
  LongToFloat,
  LongToDouble,
  FloatToInt,
  FloatToLong,
  FloatToDouble,
  DoubleToInt,
  DoubleToLong,
  DoubleToFloat,
  IntToByte,
  IntToChar,
  IntToShort,
}

#[derive(Debug)]
pub enum BinOpKind {
  AddInt,
  SubInt,
  MulInt,
  DivInt,
  RemInt,
  AndInt,
  OrInt,
  XorInt,
  ShlInt,
  ShrInt,
  UshrInt,
  AddLong,
  SubLong,
  MulLong,
  DivLong,
  RemLong,
  AndLong,
  OrLong,
  XorLong,
  ShlLong,
  ShrLong,
  UshrLong,
  AddFloat,
  SubFloat,
  MulFloat,
  DivFloat,
  RemFloat,
  AddDouble,
  SubDouble,
  MulDouble,
  DivDouble,
  RemDouble,
}

#[derive(Debug)]
pub enum BinOpLitKind {
  AddInt,
  RsubInt, // (reverse subtract)
  MulInt,
  DivInt,
  RemInt,
  AndInt,
  OrInt,
  XorInt,
}

#[derive(Debug)]
pub enum InvokeKind {
  Virtual,
  Super,
  Direct,
  Static,
  Interface,
}

#[derive(Debug)]
pub enum GetPutKind {
  Single,
  Wide,
  Object,
  Boolean,
  Byte,
  Char,
  Short,
}

#[derive(Debug)]
pub enum CmpKind {}

#[derive(Debug)]
pub enum ReturnType {
  Void,
  Single(usize),
  Wide(usize),
  Object(usize),
}

#[derive(Debug)]
pub enum MoveKind {
  Single,
  Wide,
  Object,
  // Result,
  // Exception,
}

#[derive(Debug)]
pub enum LiteralValue {
  /// Literal 32-bit
  Lit(i32),
  /// Literal 64-bit
  Wide(i64),
  /// String from StringIDX
  String(String),
  /// TypeIDX
  Class(usize),
}
