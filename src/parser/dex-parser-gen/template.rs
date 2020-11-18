use crate::parser::{InstructionQueue, ParserError};
use log::error;
use ux::u4;

macro_rules! split_byte {
  ($b:expr) => {
    (u4::new(($b as u8 & 0xf)), u4::new(($b as u8 >> 4)))
  };
}

macro_rules! join_bytes {
  (16, $b0:expr, $b1:expr) => {
    u16::from_le_bytes([$b0 as u8, $b1 as u8])
  };
  (32, $b0:expr, $b1:expr, $b2:expr, $b3:expr) => {
    u32::from_le_bytes([$b0 as u8, $b1 as u8, $b2 as u8, $b3 as u8])
  };
  (64, $b0:expr, $b1:expr, $b2:expr, $b3:expr, $b4:expr, $b5:expr, $b6:expr, $b7:expr) => {
    u64::from_le_bytes([
      $b0 as u8, $b1 as u8, $b2 as u8, $b3 as u8, $b4 as u8, $b5 as u8, $b6 as u8, $b7 as u8,
    ])
  };
}

#[derive(Debug, Clone)]
pub struct IF10t(pub i8);
#[derive(Debug, Clone)]
pub struct IF20t(pub i16);
#[derive(Debug, Clone)]
pub struct IF30t(pub i32);
#[derive(Debug, Clone)]
pub struct IF10x();
#[derive(Debug, Clone)]
pub struct IF11n(pub u4, pub u4);
#[derive(Debug, Clone)]
pub struct IF21s(pub u8, pub i16);
#[derive(Debug, Clone)]
pub struct IF21h(pub u8, pub u16);
#[derive(Debug, Clone)]
pub struct IF31i(pub u8, pub u32);
#[derive(Debug, Clone)]
pub struct IF51l(pub u8, pub u64);
#[derive(Debug, Clone)]
pub struct IF11x(pub u8);
#[derive(Debug, Clone)]
pub struct IF12x(pub u4, pub u4);
#[derive(Debug, Clone)]
pub struct IF22x(pub u8, pub u16);
#[derive(Debug, Clone)]
pub struct IF23x(pub u8, pub u8, pub u8);
#[derive(Debug, Clone)]
pub struct IF32x(pub u16, pub u16);
#[derive(Debug, Clone)]
pub struct IF21c(pub u8, pub u16);
#[derive(Debug, Clone)]
pub struct IF31c(pub u8, pub u32);
#[derive(Debug, Clone)]
pub struct IF21t(pub u8, pub u16);
#[derive(Debug, Clone)]
pub struct IF31t(pub u8, pub u32);
#[derive(Debug, Clone)]
pub struct IF22b(pub u8, pub u8, pub i8);
#[derive(Debug, Clone)]
pub struct IF22s(pub u4, pub u4, pub i16);
#[derive(Debug, Clone)]
pub struct IF22c(pub u4, pub u4, pub u16);
#[derive(Debug, Clone)]
pub struct IF22t(pub u4, pub u4, pub i16);
#[derive(Debug, Clone)]
pub struct IF35c(pub u16, pub u4, pub u4, pub u4, pub u4, pub u4, pub u4);
#[derive(Debug, Clone)]
pub struct IF3rc(pub u16, pub u16, pub u8);
#[derive(Debug, Clone)]
pub struct IF00x();
#[derive(Debug, Clone)]
pub struct IF20bc(pub u8, pub u16);
#[derive(Debug, Clone)]
pub struct IF22cs(pub u4, pub u4, pub u16);
#[derive(Debug, Clone)]
pub struct IF35mi(pub u16, pub u4, pub u4, pub u4, pub u4, pub u4, pub u4);
#[derive(Debug, Clone)]
pub struct IF35ms(pub u16, pub u4, pub u4, pub u4, pub u4, pub u4, pub u4);
#[derive(Debug, Clone)]
pub struct IF3rmi(pub u16, pub u16, pub u8);
#[derive(Debug, Clone)]
pub struct IF3rms(pub u16, pub u16, pub u8);
#[derive(Debug, Clone)]
pub struct IFPackedSwitch(pub u8, pub i32, pub Vec<i32>);
#[derive(Debug, Clone)]
pub struct IFSparseSwitch(pub u8, pub Vec<(i32, i32)>);
#[derive(Debug, Clone)]
pub struct IFFillArrayData(pub u8, pub Vec<u8>);

macro_rules! instr_format {
  (10t, $q:expr) => {{
    let b0 = $q.incr()?;
    IF10t(b0 as i8)
  }};
  (20t, $q:expr) => {{
    let _b0 = $q.incr()?;
    let b1 = $q.incr()?;
    let b2 = $q.incr()?;

    IF20t(join_bytes!(16, b1, b2) as i16)
  }};
  (30t, $q:expr) => {{
    let _b0 = $q.incr()?;
    let b1 = $q.incr()?;
    let b2 = $q.incr()?;
    let b3 = $q.incr()?;
    let b4 = $q.incr()?;

    IF30t(join_bytes!(32, b1, b2, b3, b4) as i32)
  }};
  (10x, $q:expr) => {{
    let _b0 = $q.incr()?;

    IF10x()
  }};
  (11n, $q:expr) => {{
    let b0 = $q.incr()?;
    let (a, b) = split_byte!(b0);

    IF11n(a, b)
  }};
  (21s, $q:expr) => {{
    let b0 = $q.incr()?;
    let b1 = $q.incr()?;
    let b2 = $q.incr()?;

    IF21s(b0, join_bytes!(16, b1, b2) as i16)
  }};
  (21h, $q:expr) => {{
    let b0 = $q.incr()?;
    let b1 = $q.incr()?;
    let b2 = $q.incr()?;

    IF21h(b0, join_bytes!(16, b1, b2))
  }};
  (31i, $q:expr) => {{
    let b0 = $q.incr()?;
    let b1 = $q.incr()?;
    let b2 = $q.incr()?;
    let b3 = $q.incr()?;
    let b4 = $q.incr()?;

    IF31i(b0, join_bytes!(32, b1, b2, b3, b4))
  }};
  (51l, $q:expr) => {{
    let b0 = $q.incr()?;
    let b1 = $q.incr()?;
    let b2 = $q.incr()?;
    let b3 = $q.incr()?;
    let b4 = $q.incr()?;
    let b5 = $q.incr()?;
    let b6 = $q.incr()?;
    let b7 = $q.incr()?;
    let b8 = $q.incr()?;

    IF51l(b0, join_bytes!(64, b1, b2, b3, b4, b5, b6, b7, b8))
  }};
  (11x, $q:expr) => {{
    let b0 = $q.incr()?;

    IF11x(b0)
  }};
  (12x, $q:expr) => {{
    let b0 = $q.incr()?;
    let (a, b) = split_byte!(b0);
    IF12x(a, b)
  }};
  (22x, $q:expr) => {{
    let b0 = $q.incr()?;
    let b1 = $q.incr()?;
    let b2 = $q.incr()?;

    IF22x(b0, join_bytes!(16, b1, b2))
  }};
  (23x, $q:expr) => {{
    let b0 = $q.incr()?;
    let b1 = $q.incr()?;
    let b2 = $q.incr()?;

    IF23x(b0, b1, b2)
  }};
  (32x, $q:expr) => {{
    let _b0 = $q.incr()?;
    let b1 = $q.incr()?;
    let b2 = $q.incr()?;
    let b3 = $q.incr()?;
    let b4 = $q.incr()?;

    IF32x(join_bytes!(16, b1, b2), join_bytes!(16, b3, b4))
  }};
  (21c, $q:expr) => {{
    let b0 = $q.incr()?;
    let b1 = $q.incr()?;
    let b2 = $q.incr()?;

    IF21c(b0, join_bytes!(16, b1, b2))
  }};
  (31c, $q:expr) => {{
    let b0 = $q.incr()?;
    let b1 = $q.incr()?;
    let b2 = $q.incr()?;
    let b3 = $q.incr()?;
    let b4 = $q.incr()?;

    IF31c(b0, join_bytes!(32, b1, b2, b3, b4))
  }};
  (21t, $q:expr) => {{
    let b0 = $q.incr()?;
    let b1 = $q.incr()?;
    let b2 = $q.incr()?;

    IF21t(b0, join_bytes!(16, b1, b2))
  }};
  (31t, $q:expr) => {{
    let b0 = $q.incr()?;
    let b1 = $q.incr()?;
    let b2 = $q.incr()?;
    let b3 = $q.incr()?;
    let b4 = $q.incr()?;

    IF31t(b0, join_bytes!(32, b1, b2, b3, b4))
  }};
  (22b, $q:expr) => {{
    let b0 = $q.incr()?;
    let b1 = $q.incr()?;
    let b2 = $q.incr()?;

    IF22b(b0, b1, b2 as i8)
  }};
  (22s, $q:expr) => {{
    let b0 = $q.incr()?;
    let b1 = $q.incr()?;
    let b2 = $q.incr()?;
    let (a, b) = split_byte!(b0);
    IF22s(a, b, join_bytes!(16, b1, b2) as i16)
  }};
  (22c, $q:expr) => {{
    let b0 = $q.incr()?;
    let b1 = $q.incr()?;
    let b2 = $q.incr()?;
    let (a, b) = split_byte!(b0);
    IF22c(a, b, join_bytes!(16, b1, b2) as u16)
  }};
  (22t, $q:expr) => {{
    let b0 = $q.incr()?;
    let b1 = $q.incr()?;
    let b2 = $q.incr()?;
    let (a, b) = split_byte!(b0);
    IF22t(a, b, join_bytes!(16, b1, b2) as i16)
  }};
  (35c, $q:expr) => {{
    let b0 = $q.incr()?;
    let b1 = $q.incr()?;
    let b2 = $q.incr()?;
    let b3 = $q.incr()?;
    let b4 = $q.incr()?;

    let (g, count) = split_byte!(b0);
    let b = join_bytes!(16, b1, b2);
    let (e, f) = split_byte!(b4);
    let (c, d) = split_byte!(b3);
    IF35c(b, count, c, d, e, f, g)
  }};
  (3rc, $q:expr) => {{
    let b0 = $q.incr()?;
    let b1 = $q.incr()?;
    let b2 = $q.incr()?;
    let b3 = $q.incr()?;
    let b4 = $q.incr()?;

    IF3rc(join_bytes!(16, b1, b2), join_bytes!(16, b3, b4), b0)
  }};
  (00x, $q:expr) => {{
    IF00x()
  }};
  (20bc, $q:expr) => {{
    let b0 = $q.incr()?;
    let b1 = $q.incr()?;
    let b2 = $q.incr()?;

    IF20bc(b0, join_bytes!(16, b1, b2))
  }};
  (22cs, $q:expr) => {{
    let b0 = $q.incr()?;
    let b1 = $q.incr()?;
    let b2 = $q.incr()?;

    let (a, b) = split_byte!(b0);
    IF22cs(a, b, join_bytes!(16, b1, b2))
  }};
  (35mi, $q:expr) => {{
    let b0 = $q.incr()?;
    let b1 = $q.incr()?;
    let b2 = $q.incr()?;
    let b3 = $q.incr()?;
    let b4 = $q.incr()?;

    let b = join_bytes!(16, b1, b2);
    let (a, g) = split_byte!(b0);
    let (c, d) = split_byte!(b3);
    let (e, f) = split_byte!(b4);
    IF35mi(b, a, c, d, e, f, g)
  }};
  (35ms, $q:expr) => {{
    let b0 = $q.incr()?;
    let b1 = $q.incr()?;
    let b2 = $q.incr()?;
    let b3 = $q.incr()?;
    let b4 = $q.incr()?;

    let b = join_bytes!(16, b1, b2);
    let (a, g) = split_byte!(b0);
    let (c, d) = split_byte!(b3);
    let (e, f) = split_byte!(b4);
    IF35ms(b, a, c, d, e, f, g)
  }};
  (3rmi, $q:expr) => {{
    let b0 = $q.incr()?;
    let b1 = $q.incr()?;
    let b2 = $q.incr()?;
    let b3 = $q.incr()?;
    let b4 = $q.incr()?;

    IF3rmi(join_bytes!(16, b1, b2), join_bytes!(16, b3, b4), b0)
  }};
  (3rms, $q:expr) => {{
    let b0 = $q.incr()?;
    let b1 = $q.incr()?;
    let b2 = $q.incr()?;
    let b3 = $q.incr()?;
    let b4 = $q.incr()?;

    IF3rms(join_bytes!(16, b1, b2), join_bytes!(16, b3, b4), b0)
  }};


  (PackedSwitch, $q:expr) => {{
    let instr: IF31t = instr_format!(31t, $q);
    let register_to_test = instr.0;
    let offset = instr.1 as i32;

    if offset < 0 {
      error!("Packed switch: Offset is negative. Not supported.");
      unimplemented!();
    }

    $q.jmp(offset * 2 - 5)?;

    assert_eq!($q.incr_nop()?, 0x00);
    assert_eq!($q.incr_nop()?, 0x01);

    let size = join_bytes!(16, $q.incr_nop()?, $q.incr_nop()?);
    // let tot_size = (size * 2) + 4;
    let first_key: i32 = join_bytes!(32, $q.incr_nop()?, $q.incr_nop()?, $q.incr_nop()?, $q.incr_nop()?) as i32;
    let mut targets: Vec<i32> = vec![];
    for _ in (0..size) {
      targets.push(join_bytes!(32, $q.incr_nop()?, $q.incr_nop()?, $q.incr_nop()?, $q.incr_nop()?) as i32);
    }

    $q.jmp_back()?;

    IFPackedSwitch(register_to_test, first_key, targets)
  }};
  (SparseSwitch, $q:expr) => {{
    let instr: IF31t = instr_format!(31t, $q);
    let register_to_test = instr.0;
    let offset = instr.1 as i32;

    if offset < 0 {
      error!("Sparse switch: Offset is negative. Not supported.");
      unimplemented!();
    }

    $q.jmp(offset * 2 - 5)?;

    assert_eq!($q.incr_nop()?, 0x00);
    assert_eq!($q.incr_nop()?, 0x02);

    let size = join_bytes!(16, $q.incr_nop()?, $q.incr_nop()?);
    // let tot_size = (size * 4) + 2;
    let mut keys: Vec<i32> = vec![];
    let mut targets: Vec<i32> = vec![];
    for _ in (0..size) {
      keys.push(join_bytes!(32, $q.incr_nop()?, $q.incr_nop()?, $q.incr_nop()?, $q.incr_nop()?) as i32);
    }
    for _ in (0..size) {
      targets.push(join_bytes!(32, $q.incr_nop()?, $q.incr_nop()?, $q.incr_nop()?, $q.incr_nop()?) as i32);
    }
    let mut c = 0;
    let key_targets: Vec<(i32, i32)> = keys.into_iter().map(|key| {c+=1; (key, targets[c-1])}).collect();

    $q.jmp_back()?;

    IFSparseSwitch(register_to_test, key_targets)
  }};
  (FillArrayData, $q:expr) => {{
    let instr: IF31t = instr_format!(31t, $q);
    let array_reference = instr.0;
    let offset = instr.1 as i32;

    if offset < 0 {
      error!("Fill array data: Offset is negative. Not supported.");
      unimplemented!();
    }

    $q.jmp(offset * 2 - 5)?;

    assert_eq!($q.incr_nop()?, 0x00);
    assert_eq!($q.incr_nop()?, 0x03);

    let element_width	= join_bytes!(16, $q.incr_nop()?, $q.incr_nop()?);
    let size = join_bytes!(32, $q.incr_nop()?, $q.incr_nop()?, $q.incr_nop()?, $q.incr_nop()?);
    // let tot_size = (size * element_width as u32 + 1) / 2 + 4;

    let mut data: Vec<u8> = vec![];

    for _ in (0..size * element_width as u32) {
      data.push($q.incr_nop()?);
    }

    $q.jmp_back()?;

    IFFillArrayData(array_reference, data)
  }};
}
