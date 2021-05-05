//TODO? Find "pattern"-blocks (if, switch, etc)  -  reloop

use super::generated;
use super::generated::ASTInstruction;
use log::{debug, warn};
use std::collections::HashMap;

use dexparser::TryItem;

#[derive(Debug)]
pub struct CFInstruction<'a> {
  pub(self) instruction: &'a ASTInstruction,
  pub(self) entries: Vec<usize>,
  pub(self) exits: Vec<usize>,
}

/// Go from instructions to blocks with id's
pub fn analyse_control_flow<'a>(
  insns: &'a Vec<ASTInstruction>,
  tries: &Vec<TryItem>,
  // ) -> HashMap<usize, BasicBlock<'a>> {
) -> (HashMap<usize, BasicBlock<'a>>, Vec<HashMap<String, usize>>) {
  let mut cfa = Vec::new();
  let mut cfa_exits: Vec<Vec<usize>> = Vec::new();

  // Try blocks and handlers:
  // - Make sure the try-handlers are treated as their own blocks - no 'part of block X'
  // - Make a hashmap of block-handler and handler-block/handler-target
  //? Do we need to make sure the handler is left in the correct way (with jump or return?)

  let mut current_address = 0;
  // Vector of (from, to), exception-type -> target
  //TODO Catch-all address?
  let mut handlers: Vec<((usize, usize), HashMap<String, usize>)> = vec![];
  for (i, ins) in insns.iter().enumerate() {
    // Calculate start and end index of a try-block:
    for t in tries {
      if t.start_addr != current_address {
        continue;
      }

      let start_idx = i;
      let insn_count = {
        let mut word_count = 0;
        let mut ins_count = 0;

        // Note that insn_count is NOT instructions but words
        while word_count < t.insn_count {
          word_count += generated::instruction_length(&insns[i + ins_count]) as u16;
          ins_count += 1;
        }
        ins_count
      };

      // Calculate this blocks handlers' id
      let mut hs = HashMap::new();
      for h in &t.handler.handlers {
        let idx: usize = {
          let mut word_count = 0;
          let mut i_count = 0;
          while word_count < h.addr {
            word_count += generated::instruction_length(&insns[i_count]) as u32;
            i_count += 1;
          }
          i_count
        };

        hs.insert((*h.type_).to_owned(), idx);
      }
      handlers.push(((start_idx, start_idx + insn_count), hs));

      debug!(
        "T {}  -> {}  ({}): {:?}",
        start_idx,
        start_idx + insn_count,
        insn_count,
        t
      );
    }
    current_address += generated::instruction_length(ins) as u32;

    let ci = CFInstruction {
      instruction: ins,
      entries: vec![],
      exits: exits(i, &ins, insns),
    };

    cfa_exits.push(ci.exits.clone());

    cfa.push(ci);
  }

  debug!("Handlers: {:?}", handlers);

  // Link the nodes by filling in entries
  for (i, node_exits) in cfa_exits.iter().enumerate() {
    for n_idx in node_exits {
      match cfa.get_mut(*n_idx) {
        Some(a) => a.entries.push(i),
        None => warn!("An instruction is referencing a non-existing instruction"),
      }
    }
  }

  // Turn our array of instructions with their entries and exits into a bunch of basic blocks
  into_blocks(&cfa, &handlers)
}

/// Calculate the exits of an instruction
fn exits(idx: usize, ins: &ASTInstruction, instructions: &Vec<ASTInstruction>) -> Vec<usize> {
  //TODO Fill this with all "non-linear" instructions
  match ins {
    ASTInstruction::IfEq(generated::IF22t(_, _, word_offset))
    | ASTInstruction::IfNe(generated::IF22t(_, _, word_offset))
    | ASTInstruction::IfLt(generated::IF22t(_, _, word_offset))
    | ASTInstruction::IfGe(generated::IF22t(_, _, word_offset))
    | ASTInstruction::IfGt(generated::IF22t(_, _, word_offset))
    | ASTInstruction::IfLe(generated::IF22t(_, _, word_offset))
    | ASTInstruction::IfEqz(generated::IF21t(_, word_offset))
    | ASTInstruction::IfNez(generated::IF21t(_, word_offset))
    | ASTInstruction::IfLtz(generated::IF21t(_, word_offset))
    | ASTInstruction::IfGez(generated::IF21t(_, word_offset))
    | ASTInstruction::IfGtz(generated::IF21t(_, word_offset))
    | ASTInstruction::IfLez(generated::IF21t(_, word_offset)) => {
      // TODO Is `as i16` necessary?
      debug!("IF word offset: {n:X?} ({n})", n = word_offset);
      vec![
        idx + 1,
        add_offset(*word_offset as i16 as isize, idx, instructions),
      ]
    }
    ASTInstruction::Goto(generated::IF10t(word_offset)) => {
      vec![add_offset(*word_offset as isize, idx, instructions)]
    }
    ASTInstruction::Goto16(_) | ASTInstruction::Goto32(_) => unimplemented!(),
    ASTInstruction::Return(_)
    | ASTInstruction::ReturnVoid(_)
    | ASTInstruction::ReturnObject(_)
    | ASTInstruction::ReturnVoidBarrier(_)
    | ASTInstruction::ReturnWide(_) => vec![],
    _ => vec![idx + 1],
  }
}

/// Add a word_offset to an instruction index
fn add_offset(wo: isize, idx: usize, instructions: &Vec<ASTInstruction>) -> usize {
  // wo > 0: including current instruction and not including the target instruction
  // wo < 0: not including current instruction, including target instruction

  if wo < 0 {
    //TODO Count down from idx decrementing w_count every time
    let mut w_count: usize = 0;
    let mut i_count: usize = 0;
    while w_count < wo.abs() as usize {
      // -1 to skip the first instruction
      w_count += generated::instruction_length(&instructions[idx - i_count - 1]) as usize;
      i_count += 1;
    }

    assert_eq!(w_count, wo.abs() as usize);

    debug!(
      "New index (neg): {} {} ({:?})",
      i_count,
      idx - i_count as usize,
      instructions[idx - i_count as usize]
    );
    return idx - i_count as usize;
  }

  let mut w_count = 0;
  let mut i_count: isize = 0;

  assert!(wo > 0);

  while w_count < wo {
    let a = generated::instruction_length(&instructions[idx + i_count as usize]);
    w_count += a as isize;
    i_count += 1;
  }

  idx + i_count as usize
}

#[derive(Debug)]
pub struct BasicBlock<'a> {
  pub entries: Vec<usize>,
  pub exits: Vec<usize>,
  pub body: Vec<&'a ASTInstruction>,
  pub is_handler: bool,
  // pub handlers: Vec<usize>,
  pub handler: Option<usize>,
}

impl<'a> BasicBlock<'a> {
  pub fn new() -> Self {
    Self {
      entries: vec![],
      exits: vec![],
      body: vec![],
      is_handler: false,
      // handlers: vec![],
      handler: None,
    }
  }
}

/// Turn array of CFInstructions into blocks (a hashmap of id and basic block)
/// https://en.wikipedia.org/wiki/Basic_block#Creation_algorithm
fn into_blocks<'a>(
  cfa: &Vec<CFInstruction<'a>>,
  // Start, End, (type->target-idx)
  handlers: &Vec<((usize, usize), HashMap<String, usize>)>,
) -> (HashMap<usize, BasicBlock<'a>>, Vec<HashMap<String, usize>>) {
  let mut blocks: HashMap<usize, BasicBlock> = HashMap::new();
  let mut cur_block = BasicBlock::new();
  let mut cur_block_id = 0;

  // Array with the handlers to return. This is filled while the blocks are generated
  //TODO Block index?
  // [Type -> Block index]
  let mut new_handlers = Vec::new();

  let handler_indices: Vec<usize> = {
    let mut out = vec![];
    for h in handlers {
      for (_, v) in &h.1 {
        out.push(*v);
      }
    }

    out
  };

  // Map last instruction in each block to the
  // actual block id to later be able to find the
  // block when updating entries.
  let mut block_entries: HashMap<usize, usize> = HashMap::new();

  let mut last_was_goto = false;
  for (i, c) in cfa.iter().enumerate() {
    // Identify the leaders in the code:
    // - First instruction
    // - Target of a jump
    // - Following a jump/goto
    // - Start of a try-block
    // - Target of a try-handler
    let is_start_of_try_block = {
      match handlers.iter().find(|&x| x.0 .0 == i) {
        None => false,
        Some(_) => true,
      }
    };
    let is_start_of_try_handler = {
      match handler_indices.iter().find(|&&x| x == i) {
        Some(_) => true,
        None => false,
      }
    };
    if i != 0 && c.entries != vec![i - 1]
      || last_was_goto
      || is_start_of_try_handler
      || is_start_of_try_block
    {
      last_was_goto = false;

      // Add the last block(current_block) to the blocks and make a new one
      // Also add i-1 to block_entries to later be able to look it up
      debug!("block_entries: {}->{}", i - 1, cur_block_id);
      block_entries.insert(i - 1, cur_block_id);
      debug!("Adding block: {}", cur_block_id);
      blocks.insert(cur_block_id, cur_block);

      cur_block = BasicBlock::new();
      cur_block_id = i;
      cur_block.entries = c.entries.clone();
      cur_block.is_handler = is_start_of_try_handler;

      //TODO! Add this handler's handlers to the new_handlers
      if is_start_of_try_block {
        //TODO Set the block ID dynamically
        for h in handlers.iter().filter(|&x| (x.0).0 <= i && (x.0).1 >= i) {
          let idx = new_handlers.len();
          //TODO Remove clone
          new_handlers.push(h.1.clone());
          //* cur_block.handlers.push(idx);
          if cur_block.handler == None {
            cur_block.handler = Some(idx);
          } else {
            unreachable!("Unexpected: two try blocks seem overlap");
          }
        }
      }
    }

    // All of the blocks until the next leader is in this block
    cur_block.body.push(c.instruction);
    //TODO: Can this be done in some better way?
    cur_block.exits = c.exits.clone();

    // Set last_was_goto if the current instruction is not linear
    //TODO or this is the end of a try-block
    if c.exits != vec![i + 1] {
      last_was_goto = true;
    }
  }
  // Add the last block, since the loop didn't do it
  //TODO?: Nicer way of doing this? Are there situations where this fails?
  blocks.insert(cur_block_id, cur_block);

  debug!("Block entries: {:?}", block_entries);
  // Add the block_entries hashmap to the entries vector in the actual blocks
  for (_, block) in blocks.iter_mut() {
    block.entries = block
      .entries
      .iter()
      .map(|e| {
        *block_entries
          .get(e)
          .expect(&format!("Can not find entry in block_entries: {}", e))
      })
      .collect();
  }

  (blocks, new_handlers)
}

pub fn format_analysis(
  analysis: &(HashMap<usize, BasicBlock>, Vec<HashMap<String, usize>>),
) -> String {
  // pub fn format_analysis(a: &HashMap<usize, BasicBlock>) -> String {
  let mut out = Vec::new();

  let mut a: Vec<(&usize, &BasicBlock)> = analysis.0.iter().collect();
  a.sort_by_key(|x| x.0);

  for (key, value) in a {
    let mut instructions = Vec::new();
    for i in &value.body {
      instructions.push(format!("\t\t{:?}", i));
    }

    out.push(format!(
      "{key}| En: {entries:?} ; Ex: {exits:?}\n\t{handlers:?} {is_handler}\n{instructions}",
      key = key,
      entries = value.entries,
      exits = value.exits,
      is_handler = if value.is_handler { "(handler)" } else { "" },
      handlers = value.handler,
      instructions = instructions.join("\n")
    ));
  }

  out.push(format!("; Handlers\n{:#?}", analysis.1));

  out.join("\n\n")
}
