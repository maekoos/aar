use std::time::{SystemTime, UNIX_EPOCH};

pub use dex::method::AccessFlags;
use dex::{class, code, jtype, method, Dex};
use ux::u4;

type Dexx = Dex<memmap::Mmap>;

pub mod generated;
pub use generated::instruction_length;
use generated::parse_instruction;
pub use generated::ASTInstruction;

#[derive(Debug)]
pub enum ParserError {
  EOF,
}

#[derive(Debug)]
pub struct InstructionQueue {
  pub bytecode: Vec<u8>,
  index: usize,
  has_begun: bool,
  indexes: Vec<usize>,
}

impl InstructionQueue {
  pub fn new(code: &code::CodeItem) -> Self {
    Self {
      bytecode: code_to_bytecode(code),
      index: 0_usize,
      has_begun: false,
      indexes: vec![0_usize],
    }
  }

  pub fn incr(&mut self) -> Result<u8, ParserError> {
    self.index += 1;
    if !self.has_begun {
      self.has_begun = true;
      self.index -= 1;
    }

    if self.bytecode.len() == self.index {
      self.index -= 1;
      return Err(ParserError::EOF);
    }
    Ok(self.bytecode[self.index])
  }

  /// Increment and then make the current instruction a NOP
  pub fn incr_nop(&mut self) -> Result<u8, ParserError> {
    let v = self.incr()?;
    self.bytecode[self.index] = 0x00;
    Ok(v)
  }

  pub fn is_empty(&self) -> bool {
    self.bytecode.len() == self.index + 1
  }

  pub fn jmp(&mut self, o: i32) -> Result<(), ParserError> {
    self.indexes.push(self.index);
    self.index = if o.is_negative() {
      self.index - o.wrapping_abs() as u32 as usize
    } else {
      self.index + o as usize
    };

    if self.bytecode.len() == self.index {
      self.index = self.indexes.pop().unwrap();
      return Err(ParserError::EOF);
    }
    self.has_begun = false;
    Ok(())
  }

  pub fn jmp_back(&mut self) -> Result<(), ParserError> {
    self.index = self.indexes.pop().unwrap();
    Ok(())
  }
}

fn code_to_bytecode(insns: &code::CodeItem) -> Vec<u8> {
  insns
    .insns()
    .iter()
    .map(|x| x.to_le_bytes())
    .collect::<Vec<[u8; 2]>>()
    .concat()
}

#[derive(Debug)]
pub struct ASTClass {
  pub methods: Vec<ASTMethod>,
  pub jtype: String,
  pub class_id: u32,
}

impl ASTClass {
  pub fn parse(c: &class::Class, d: &Dexx) -> Result<Self, ParserError> {
    let mut acs = vec![];
    for m in c.methods() {
      acs.push(ASTMethod::parse(m, &d)?);
    }

    Ok(Self {
      methods: acs,
      jtype: c.jtype().to_string(),
      class_id: c.id(),
    })
  }
}

pub type Invoke = (usize, String, usize, String, u8, [u8; 5]);

#[derive(Debug, Clone)]
pub enum ASTI {
  Instruction(ASTInstruction),
  Target(usize),
  Goto(usize),
  PackedSwitch(u8, i32, Vec<usize>),
  SparseSwitch(u8, Vec<(i32, usize)>),
  IfNe(u4, u4, usize),
  IfEq(u4, u4, usize),
  MoveResult(usize, u8),
  InvokeVirtual(Invoke),
  InvokeDirect(Invoke),
  InvokeSuper(Invoke),
}

impl ASTI {
  pub fn len(&self) -> u8 {
    match self {
      ASTI::Instruction(i) => instruction_length(&i),
      ASTI::PackedSwitch(_, _, _) => 6,
      ASTI::SparseSwitch(_, _) => 6,
      ASTI::Goto(_) => 2,
      ASTI::Target(_) => 0,
      ASTI::IfNe(_, _, _) => 4,
      ASTI::IfEq(_, _, _) => 4,
      ASTI::MoveResult(_, _) => 2,
      ASTI::InvokeVirtual(_) => 6,
      ASTI::InvokeDirect(_) => 6,
      ASTI::InvokeSuper(_) => 6,
    }
  }
}

fn temp_id() -> u128 {
  let start = SystemTime::now();
  let since_the_epoch = start
    .duration_since(UNIX_EPOCH)
    .expect("Time went backwards");
  since_the_epoch.as_millis()
}

fn add_target(index: usize, orig_offset: i32, target_id: usize, asti: &mut Vec<ASTI>) {
  let is_pos = orig_offset > 0;
  if !is_pos {
    let mut offset_left: isize = orig_offset as isize * 2;
    let mut move_count: usize = 0;

    while offset_left < 0 {
      move_count += 1;
      let idx = index - move_count;
      offset_left += asti[idx].len() as isize;
    }
    assert_eq!(offset_left, 0);

    asti.insert(index - move_count, ASTI::Target(target_id));
  } else {
    let mut offset_left: isize = orig_offset as isize * 2;
    let mut move_count: usize = 0;
    while offset_left > 0 {
      let idx = index + move_count;
      offset_left -= asti[idx].len() as isize;
      move_count += 1;
    }
    assert_eq!(offset_left, 0);
    asti.insert(index + move_count, ASTI::Target(target_id));
  }
}

#[derive(Debug, Clone)]
pub struct ASTMethod {
  pub id: usize,
  pub name: String,
  pub body: Vec<ASTI>,
  pub params: Vec<jtype::Type>,
  pub return_type: jtype::Type,
  pub targets: Vec<usize>,
  pub registers: usize,
  pub access_flags: dex::method::AccessFlags,
}

impl ASTMethod {
  pub fn parse(m: &method::Method, d: &Dexx) -> Result<Self, ParserError> {
    let (registers, body) = match m.code() {
      Some(c) => {
        let mut q = InstructionQueue::new(c);
        let mut asti: Vec<ASTI> = vec![];
        while !q.is_empty() {
          asti.push(ASTI::Instruction(parse_instruction(&mut q)?));
        }

        let mut i: usize = 0;
        let mut target_id = 0;
        let mut result_idx = 0;
        while i < asti.len() {
          let instr = &asti[i];
          match instr {
            ASTI::Instruction(ASTInstruction::Goto(g)) => {
              let offset = g.0;
              add_target(i, offset as i32, target_id, &mut asti);
              if offset < 0 {
                i += 1;
              }
              asti[i] = ASTI::Goto(target_id);
              target_id += 1;
            }
            ASTI::Instruction(ASTInstruction::PackedSwitch(ps)) => {
              let register = ps.0;
              let first = ps.1;
              let mut targets = vec![];

              for jump in ps.2.clone().iter() {
                let offset: i32 = *jump;
                add_target(i, offset, target_id, &mut asti);
                if offset < 0 {
                  i += 1;
                }
                targets.push(target_id);
                target_id += 1;
              }

              asti[i] = ASTI::PackedSwitch(register, first, targets);
            }
            ASTI::Instruction(ASTInstruction::SparseSwitch(ss)) => {
              let register = ss.0;
              let mut targets: Vec<(i32, usize)> = vec![];

              for (key, jump) in ss.1.clone().iter() {
                let offset: i32 = *jump;
                add_target(i, offset, target_id, &mut asti);
                if offset < 0 {
                  i += 1;
                }
                targets.push((*key, target_id));
                target_id += 1;
              }

              asti[i] = ASTI::SparseSwitch(register, targets);
            }
            ASTI::Instruction(ASTInstruction::IfEq(n)) => {
              let first = n.0;
              let second = n.1;
              let offset = n.2;

              add_target(i, offset as i32, target_id, &mut asti);
              if offset < 0 {
                i += 1;
              }
              asti[i] = ASTI::IfEq(first, second, target_id);
              target_id += 1;
            }
            ASTI::Instruction(ASTInstruction::IfNe(n)) => {
              let first = n.0;
              let second = n.1;
              let offset = n.2;

              add_target(i, offset as i32, target_id, &mut asti);
              if offset < 0 {
                i += 1;
              }
              asti[i] = ASTI::IfNe(first, second, target_id);
              target_id += 1;
            }
            ASTI::Instruction(ASTInstruction::MoveResult(a)) => {
              asti[i] = ASTI::MoveResult(result_idx, a.0);
            }
            ASTI::Instruction(ASTInstruction::InvokeVirtual(a)) => {
              let b = parse_invoke(
                d,
                a.0 as u64,
                a.1.into(),
                a.2.into(),
                a.3.into(),
                a.4.into(),
                a.5.into(),
                a.6.into(),
              );

              result_idx += 1;
              asti[i] = ASTI::InvokeVirtual((result_idx, b.0, b.1, b.2, b.3, b.4));
            }
            ASTI::Instruction(ASTInstruction::InvokeDirect(a)) => {
              let b = parse_invoke(
                d,
                a.0 as u64,
                a.1.into(),
                a.2.into(),
                a.3.into(),
                a.4.into(),
                a.5.into(),
                a.6.into(),
              );

              result_idx += 1;
              asti[i] = ASTI::InvokeDirect((result_idx, b.0, b.1, b.2, b.3, b.4));
            }
            ASTI::Instruction(ASTInstruction::InvokeSuper(a)) => {
              let b = parse_invoke(
                d,
                a.0 as u64,
                a.1.into(),
                a.2.into(),
                a.3.into(),
                a.4.into(),
                a.5.into(),
                a.6.into(),
              );

              result_idx += 1;
              asti[i] = ASTI::InvokeSuper((result_idx, b.0, b.1, b.2, b.3, b.4));
            }
            _ => {}
          }

          i += 1;
        }

        (c.registers_size(), asti)
      }
      None => (0, vec![]),
    };

    Ok(Self {
      id: temp_id() as usize,
      name: m.name().to_string(),
      body,
      params: m.params().to_owned(),
      return_type: m.return_type().to_owned(),
      targets: Vec::new(),
      registers: registers as usize,
      access_flags: m.access_flags(),
    })
  }
}

fn parse_invoke(
  d: &Dexx,
  method_id: u64,
  pcount: u8,
  p1: u8,
  p2: u8,
  p3: u8,
  p4: u8,
  p5: u8,
) -> (String, usize, String, u8, [u8; 5]) {
  let m = d.get_method_item(method_id).unwrap();
  let name = d.get_string(m.name_idx()).unwrap();
  let p = d.get_proto_item(m.proto_idx() as u64).unwrap();
  let shorty = d.get_string(p.shorty()).unwrap().to_string();

  let params: [u8; 5] = [p1, p2, p3, p4, p5];

  return (
    shorty,
    m.class_idx() as usize,
    name.to_string(),
    pcount,
    params,
  );
}
