enum GenerationInstruction {
  Line(String),
  IndentationIncr,
  IndentationDecr,
}

pub struct FileContent {
  head: Vec<GenerationInstruction>,
  body: Vec<GenerationInstruction>,
  footer: Vec<GenerationInstruction>,
}

impl FileContent {
  pub fn new() -> Self {
    Self {
      head: Vec::new(),
      body: Vec::new(),
      footer: Vec::new(),
    }
  }

  pub fn indentation_incr(&mut self) {
    self.body.push(GenerationInstruction::IndentationIncr)
  }

  pub fn indentation_decr(&mut self) {
    self.body.push(GenerationInstruction::IndentationDecr)
  }

  pub fn writeln(&mut self, t: &str) {
    self
      .body
      .push(GenerationInstruction::Line(format!("{}\n", t)))
  }

  pub fn _h_indentation_incr(&mut self) {
    self.head.push(GenerationInstruction::IndentationIncr)
  }

  pub fn _h_indentation_decr(&mut self) {
    self.head.push(GenerationInstruction::IndentationDecr)
  }

  pub fn h_writeln(&mut self, t: &str) {
    self
      .head
      .push(GenerationInstruction::Line(format!("{}\n", t)))
  }

  pub fn _f_indentation_incr(&mut self) {
    self.footer.push(GenerationInstruction::IndentationIncr)
  }

  pub fn _f_indentation_decr(&mut self) {
    self.footer.push(GenerationInstruction::IndentationDecr)
  }

  pub fn f_writeln(&mut self, t: &str) {
    self
      .footer
      .push(GenerationInstruction::Line(format!("{}\n", t)))
  }

  pub fn generate(&self) -> String {
    format!(
      "{}\n{}\n{}",
      self.generate_instr(&self.head),
      self.generate_instr(&self.body),
      self.generate_instr(&self.footer)
    )
  }

  fn generate_instr(&self, instructions: &Vec<GenerationInstruction>) -> String {
    let mut s = String::new();

    let inden_size = 2;
    let mut inden: usize = 0;
    for gi in instructions {
      s.push_str(&match gi {
        GenerationInstruction::IndentationIncr => {
          inden += 1;
          String::from("")
        }
        GenerationInstruction::IndentationDecr => {
          inden -= 1;
          String::from("")
        }
        GenerationInstruction::Line(t) => format!("{}{}", " ".repeat(inden * inden_size), t),
      });
    }

    s
  }
}

pub struct Generator {
  pub c: FileContent,
  pub h: FileContent,
}

impl Generator {
  pub fn new() -> Self {
    Self {
      c: FileContent::new(),
      h: FileContent::new(),
    }
  }

  pub fn generate(&self) -> (String, String) {
    (self.c.generate(), self.h.generate())
  }
}
