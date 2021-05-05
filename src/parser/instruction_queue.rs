//TODO Memory efficiency: Borrow bytecode?

use super::{Code, ParserError};

#[derive(Debug)]
// pub struct InstructionQueue<'a> {
pub struct InstructionQueue {
  // pub bytecode: &'a Vec<u16>,
  pub bytecode: Vec<u8>,
  index: usize,
  has_begun: bool,
  indexes: Vec<usize>,
}

impl<'a> InstructionQueue {
  pub fn new(code: &'a Code) -> Self {
    let mut instructions_u8: Vec<u8> = Vec::new();
    for c in &code.insns {
      instructions_u8.push((c & 0xff) as u8);
      instructions_u8.push((c >> 8) as u8);
    }

    Self {
      bytecode: instructions_u8,
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

  pub fn _is_empty(&self) -> bool {
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
