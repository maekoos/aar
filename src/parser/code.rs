use std::fs::File;
use std::io::prelude::*;

use super::codegen::{self, instructions};
use super::generated::{self, ASTInstruction};
use super::{
	control_flow, format_classname, format_name, AccessFlag, Code, DexFile, EncodedMethod,
	InstructionQueue, ParserError,
};

/// Generate codegen-IR from a method
pub fn generate_code(
	c: &Code,
	m: &EncodedMethod,
	c_name: &str,
	_class: &dexparser::ClassDefinition,
	dex: &DexFile,
	fnb: &mut codegen::FunctionBuilder,
) {
	let is_instance = !m.access_flags.contains(&AccessFlag::ACC_STATIC);
	let n_params = m.method.prototype.parameters.len() + if is_instance { 1 } else { 0 };

	fnb.set_n_regs(c.registers_size as _);
	fnb.set_n_params(n_params);
	fnb.set_return(&*m.method.prototype.return_type != "V");

	//TODO? Init instance fields before use?
	// if *m.method.name == "<init>" {
	// 	if let Some(cd) = &class.class_data {
	// 		for f in &cd.instance_fields {
	// 			if f.field.type_.starts_with("L") { todo!(); }
	// 		}
	// 	}
	// }

	//TODO Move this into the CFA, since we are otherwise iterating through the instruction-list multiple times
	//TODO Maybe make the `InstructionQueue` and `generated::parse_instruction` an iterator and parse each instruction when we it?
	let mut iq = InstructionQueue::new(c);
	let mut insns = Vec::new();
	loop {
		match generated::parse_instruction(&mut iq) {
			Ok(ins) => insns.push(ins),
			//TODO Handle this?
			Err(ParserError::EOF) => break,
		}
	}

	let analysis = control_flow::analyse_control_flow(&insns, &c.tries);
	//TODO Dynamic (and optional) out-directory
	let mut file = File::create(&format!("out/analysis/{}__{}", c_name, m.method.name)).unwrap();
	file.write_all(control_flow::format_analysis(&analysis).as_bytes())
		.unwrap();

	let mut flow: Vec<(&usize, &control_flow::BasicBlock)> = analysis.0.iter().collect();
	flow.sort_by_key(|x| x.0);

	let handlers = analysis.1;
	fnb.set_handlers(handlers);

	let mut last_target = 0;
	for (i, block) in flow {
		if block.entries.len() != 0 && block.entries != vec![last_target] || block.is_handler {
			fnb.label(*i);
		}

		for i in &block.body {
			fnb.set_next_handler(block.handler);

			match i {
				ASTInstruction::Nop(_) => {}
				ASTInstruction::Move(_) => todo!(),
				ASTInstruction::MoveFrom16(_) => todo!(),
				ASTInstruction::Move16(_) => todo!(),
				ASTInstruction::MoveWide(_) => todo!(),
				ASTInstruction::MoveWideFrom16(_) => todo!(),
				ASTInstruction::MoveWide16(_) => todo!(),
				ASTInstruction::MoveObject(_) => todo!(),
				ASTInstruction::MoveObjectFrom16(_) => todo!(),
				ASTInstruction::MoveObject16(_) => todo!(),
				ASTInstruction::MoveResult(generated::IF11x(a)) => {
					fnb.move_results(instructions::MoveKind::Single, *a as usize)
				}
				ASTInstruction::MoveResultWide(_) => todo!(),
				ASTInstruction::MoveResultObject(generated::IF11x(a)) => {
					fnb.move_results(instructions::MoveKind::Object, *a as usize)
				}
				ASTInstruction::MoveException(generated::IF11x(a)) => {
					fnb.move_exception(*a as usize)
				}
				ASTInstruction::ReturnVoid(_) => fnb.return_v(instructions::ReturnType::Void),
				ASTInstruction::Return(generated::IF11x(v)) => {
					fnb.return_v(instructions::ReturnType::Single(*v as _))
				}
				ASTInstruction::ReturnWide(_) => todo!(),
				ASTInstruction::ReturnObject(generated::IF11x(v)) => {
					fnb.return_v(instructions::ReturnType::Object(*v as usize))
				}
				ASTInstruction::Const4(generated::IF11n(dest, signed_int)) => fnb.const_set(
					u8::from(*dest) as _,
					instructions::LiteralValue::Lit(u8::from(*signed_int) as i8 as i32), //? Does this make a difference?
				),
				ASTInstruction::Const16(generated::IF21s(v, lit)) => {
					fnb.const_set(*v as _, instructions::LiteralValue::Lit(*lit as i16 as i32))
				}
				ASTInstruction::Const(_) => todo!(),
				ASTInstruction::ConstHigh16(_) => todo!(),
				ASTInstruction::ConstWide16(_) => todo!(),
				ASTInstruction::ConstWide32(_) => todo!(),
				ASTInstruction::ConstWide(_) => todo!(),
				ASTInstruction::ConstWideHigh16(_) => todo!(),
				ASTInstruction::ConstString(generated::IF21c(v, s_idx)) => fnb.const_set(
					*v as _,
					instructions::LiteralValue::String(
						match dex.file_data.string_data.get(*s_idx as usize) {
							None => todo!(),
							Some(s) => (**s).clone(),
						},
					),
				),
				ASTInstruction::ConstStringJumbo(_) => todo!(),
				ASTInstruction::ConstClass(_) => todo!(),
				ASTInstruction::MonitorEnter(_) => todo!(),
				ASTInstruction::MonitorExit(_) => todo!(),
				ASTInstruction::CheckCast(_) => todo!(),
				ASTInstruction::InstanceOf(_) => todo!(),
				ASTInstruction::ArrayLength(_) => todo!(),
				ASTInstruction::NewInstance(generated::IF21c(dest, ty)) => {
					fnb.new_instance(*dest, *ty as usize)
				}
				ASTInstruction::NewArray(generated::IF22c(v_dest, v_size, type_)) => fnb.new_array(
					u8::from(*v_dest),
					u8::from(*v_size) as usize,
					*type_ as usize,
				),
				ASTInstruction::FilledNewArray(_) => todo!(),
				ASTInstruction::FilledNewArrayRange(_) => todo!(),
				ASTInstruction::FillArrayData(generated::IFFillArrayData(v, el_width, data)) => {
					fnb.fill_array_data(*v as usize, *el_width, data.clone())
				}
				ASTInstruction::Throw(_) => todo!(),
				ASTInstruction::Goto(_) | ASTInstruction::Goto16(_) | ASTInstruction::Goto32(_) => {
					fnb.goto(block.exits[0])
				}
				ASTInstruction::PackedSwitch(_) => todo!(),
				ASTInstruction::SparseSwitch(_) => todo!(),
				ASTInstruction::CmplFloat(_) => todo!(),
				ASTInstruction::CmpgFloat(_) => todo!(),
				ASTInstruction::CmplDouble(_) => todo!(),
				ASTInstruction::CmpgDouble(_) => todo!(),
				ASTInstruction::CmpLong(_) => todo!(),
				ASTInstruction::IfEq(_) => todo!(),
				ASTInstruction::IfNe(generated::IF22t(v1, v2, _)) => {
					//TODO: Better solution to jumps? Can't just trust the order of the exits...
					fnb.if_test(
						instructions::IfKind::Ne,
						u8::from(*v1),
						u8::from(*v2),
						block.exits[1],
					)
				}
				ASTInstruction::IfLt(_) => todo!(),
				ASTInstruction::IfGe(generated::IF22t(v1, v2, _)) => {
					//TODO: Better solution to jumps? Can't just trust the order of the exits...
					fnb.if_test(
						instructions::IfKind::Ge,
						u8::from(*v1),
						u8::from(*v2),
						block.exits[1],
					)
				}
				ASTInstruction::IfGt(_) => todo!(),
				ASTInstruction::IfLe(_) => todo!(),
				ASTInstruction::IfEqz(_) => todo!(),
				ASTInstruction::IfNez(_) => todo!(),
				ASTInstruction::IfLtz(_) => todo!(),
				ASTInstruction::IfGez(_) => todo!(),
				ASTInstruction::IfGtz(_) => todo!(),
				ASTInstruction::IfLez(_) => todo!(),
				ASTInstruction::Aget(generated::IF23x(v_dest, v_arr, v_idx)) => {
					fnb.array_get(instructions::GetPutKind::Single, *v_dest, *v_arr, *v_idx)
				}
				ASTInstruction::AgetWide(_) => todo!(),
				ASTInstruction::AgetObject(generated::IF23x(v_dest, v_arr, v_idx)) => {
					fnb.array_get(instructions::GetPutKind::Object, *v_dest, *v_arr, *v_idx)
				}
				ASTInstruction::AgetBoolean(generated::IF23x(v_dest, v_arr, v_idx)) => {
					fnb.array_get(instructions::GetPutKind::Boolean, *v_dest, *v_arr, *v_idx)
				}
				ASTInstruction::AgetByte(_) => todo!(),
				ASTInstruction::AgetChar(_) => todo!(),
				ASTInstruction::AgetShort(_) => todo!(),
				ASTInstruction::Aput(_) => todo!(),
				ASTInstruction::AputWide(_) => todo!(),
				ASTInstruction::AputObject(generated::IF23x(v_dest, v_arr, v_idx)) => {
					fnb.array_get(instructions::GetPutKind::Object, *v_dest, *v_arr, *v_idx)
				}
				ASTInstruction::AputBoolean(_) => todo!(),
				ASTInstruction::AputByte(_) => todo!(),
				ASTInstruction::AputChar(_) => todo!(),
				ASTInstruction::AputShort(_) => todo!(),
				ASTInstruction::Iget(generated::IF22c(v_dest, v_inst, field_idx)) => fnb
					.instance_get(
						instructions::GetPutKind::Single,
						u8::from(*v_dest),
						u8::from(*v_inst),
						get_field_name(*field_idx as usize, false, dex),
					),
				ASTInstruction::IgetWide(_) => todo!(),
				ASTInstruction::IgetObject(generated::IF22c(v_dest, v_inst, field_idx)) => {
					fnb.instance_get(
						instructions::GetPutKind::Object,
						u8::from(*v_dest),
						u8::from(*v_inst),
						get_field_name(*field_idx as usize, false, dex),
					);
				}
				ASTInstruction::IgetBoolean(_) => todo!(),
				ASTInstruction::IgetByte(_) => todo!(),
				ASTInstruction::IgetChar(_) => todo!(),
				ASTInstruction::IgetShort(_) => todo!(),
				ASTInstruction::Iput(generated::IF22c(v_src, v_inst, field_ref_idx)) => fnb
					.instance_put(
						instructions::GetPutKind::Single,
						u8::from(*v_src),
						u8::from(*v_inst),
						get_field_name(*field_ref_idx as _, false, dex),
					),
				ASTInstruction::IputWide(_) => todo!(),
				ASTInstruction::IputObject(generated::IF22c(src, inst, field_ref_idx)) => fnb
					.instance_put(
						instructions::GetPutKind::Object,
						u8::from(*src),
						u8::from(*inst),
						get_field_name(*field_ref_idx as _, false, dex),
					),
				ASTInstruction::IputBoolean(_) => todo!(),
				ASTInstruction::IputByte(_) => todo!(),
				ASTInstruction::IputChar(_) => todo!(),
				ASTInstruction::IputShort(_) => todo!(),
				ASTInstruction::Sget(generated::IF21c(v_dest, static_idx)) => fnb.static_get(
					instructions::GetPutKind::Single,
					*v_dest,
					get_field_name(*static_idx as usize, true, dex),
				),
				ASTInstruction::SgetWide(_) => todo!(),
				ASTInstruction::SgetObject(generated::IF21c(v_dest, static_idx)) => fnb.static_get(
					instructions::GetPutKind::Object,
					*v_dest,
					get_field_name(*static_idx as usize, true, dex),
				),
				ASTInstruction::SgetBoolean(_) => todo!(),
				ASTInstruction::SgetByte(_) => todo!(),
				ASTInstruction::SgetChar(_) => todo!(),
				ASTInstruction::SgetShort(_) => todo!(),
				ASTInstruction::Sput(generated::IF21c(v_src, static_idx)) => fnb.static_put(
					instructions::GetPutKind::Single,
					u8::from(*v_src),
					get_field_name(*static_idx as usize, true, dex),
					// *static_idx,
				),
				ASTInstruction::SputWide(_) => todo!(),
				ASTInstruction::SputObject(_) => todo!(),
				ASTInstruction::SputBoolean(_) => todo!(),
				ASTInstruction::SputByte(_) => todo!(),
				ASTInstruction::SputChar(_) => todo!(),
				ASTInstruction::SputShort(_) => todo!(),
				ASTInstruction::InvokeVirtual(generated::IF35c(
					method_ref_idx,
					argc,
					a1,
					a2,
					a3,
					a4,
					a5,
				)) => fnb.invoke(
					instructions::InvokeKind::Virtual,
					get_method_full_name(*method_ref_idx as usize, dex),
					u8::from(*argc),
					[
						u8::from(*a1),
						u8::from(*a2),
						u8::from(*a3),
						u8::from(*a4),
						u8::from(*a5),
					],
				),
				ASTInstruction::InvokeSuper(_) => todo!(),
				//TODO: macro for invoke?
				ASTInstruction::InvokeDirect(generated::IF35c(
					method_ref_idx,
					argc,
					a1,
					a2,
					a3,
					a4,
					a5,
				)) => fnb.invoke(
					instructions::InvokeKind::Direct,
					get_method_full_name(*method_ref_idx as usize, dex),
					u8::from(*argc),
					[
						u8::from(*a1),
						u8::from(*a2),
						u8::from(*a3),
						u8::from(*a4),
						u8::from(*a5),
					],
				),
				ASTInstruction::InvokeStatic(generated::IF35c(
					method_ref_idx,
					argc,
					a1,
					a2,
					a3,
					a4,
					a5,
				)) => fnb.invoke(
					instructions::InvokeKind::Static,
					get_method_full_name(*method_ref_idx as usize, dex),
					u8::from(*argc),
					[
						u8::from(*a1),
						u8::from(*a2),
						u8::from(*a3),
						u8::from(*a4),
						u8::from(*a5),
					],
				),
				// ASTInstruction::InvokeStatic(_) => todo!(),
				ASTInstruction::InvokeInterface(_) => todo!(),
				ASTInstruction::InvokeVirtualRange(_) => todo!(),
				ASTInstruction::InvokeSuperRange(_) => todo!(),
				ASTInstruction::InvokeDirectRange(_) => todo!(),
				ASTInstruction::InvokeStaticRange(_) => todo!(),
				ASTInstruction::InvokeInterfaceRange(_) => todo!(),
				ASTInstruction::NegInt(_) => todo!(),
				ASTInstruction::NotInt(_) => todo!(),
				ASTInstruction::NegLong(_) => todo!(),
				ASTInstruction::NotLong(_) => todo!(),
				ASTInstruction::NegFloat(_) => todo!(),
				ASTInstruction::NegDouble(_) => todo!(),
				ASTInstruction::IntToLong(_) => todo!(),
				ASTInstruction::IntToFloat(_) => todo!(),
				ASTInstruction::IntToDouble(_) => todo!(),
				ASTInstruction::LongToInt(_) => todo!(),
				ASTInstruction::LongToFloat(_) => todo!(),
				ASTInstruction::LongToDouble(_) => todo!(),
				ASTInstruction::FloatToInt(_) => todo!(),
				ASTInstruction::FloatToLong(_) => todo!(),
				ASTInstruction::FloatToDouble(_) => todo!(),
				ASTInstruction::DoubleToInt(_) => todo!(),
				ASTInstruction::DoubleToLong(_) => todo!(),
				ASTInstruction::DoubleToFloat(_) => todo!(),
				ASTInstruction::IntToByte(_) => todo!(),
				ASTInstruction::IntToChar(_) => todo!(),
				ASTInstruction::IntToShort(_) => todo!(),
				ASTInstruction::AddInt(_) => todo!(),
				ASTInstruction::SubInt(_) => todo!(),
				ASTInstruction::MulInt(_) => todo!(),
				ASTInstruction::DivInt(_) => todo!(),
				ASTInstruction::RemInt(_) => todo!(),
				ASTInstruction::AndInt(_) => todo!(),
				ASTInstruction::OrInt(_) => todo!(),
				ASTInstruction::XorInt(_) => todo!(),
				ASTInstruction::ShlInt(_) => todo!(),
				ASTInstruction::ShrInt(_) => todo!(),
				ASTInstruction::UshrInt(_) => todo!(),
				ASTInstruction::AddLong(_) => todo!(),
				ASTInstruction::SubLong(_) => todo!(),
				ASTInstruction::MulLong(_) => todo!(),
				ASTInstruction::DivLong(_) => todo!(),
				ASTInstruction::RemLong(_) => todo!(),
				ASTInstruction::AndLong(_) => todo!(),
				ASTInstruction::OrLong(_) => todo!(),
				ASTInstruction::XorLong(_) => todo!(),
				ASTInstruction::ShlLong(_) => todo!(),
				ASTInstruction::ShrLong(_) => todo!(),
				ASTInstruction::UshrLong(_) => todo!(),
				ASTInstruction::AddFloat(_) => todo!(),
				ASTInstruction::SubFloat(_) => todo!(),
				ASTInstruction::MulFloat(_) => todo!(),
				ASTInstruction::DivFloat(_) => todo!(),
				ASTInstruction::RemFloat(_) => todo!(),
				ASTInstruction::AddDouble(_) => todo!(),
				ASTInstruction::SubDouble(_) => todo!(),
				ASTInstruction::MulDouble(_) => todo!(),
				ASTInstruction::DivDouble(_) => todo!(),
				ASTInstruction::RemDouble(_) => todo!(),
				ASTInstruction::AddInt2addr(generated::IF12x(v_dest_and_src_a, v_src_b)) => fnb
					.bin_op_2_addr(
						instructions::BinOpKind::AddInt,
						u8::from(*v_dest_and_src_a),
						u8::from(*v_src_b),
					),
				ASTInstruction::SubInt2addr(_) => todo!(),
				ASTInstruction::MulInt2addr(_) => todo!(),
				ASTInstruction::DivInt2addr(_) => todo!(),
				ASTInstruction::RemInt2addr(_) => todo!(),
				ASTInstruction::AndInt2addr(_) => todo!(),
				ASTInstruction::OrInt2addr(_) => todo!(),
				ASTInstruction::XorInt2addr(_) => todo!(),
				ASTInstruction::ShlInt2addr(_) => todo!(),
				ASTInstruction::ShrInt2addr(_) => todo!(),
				ASTInstruction::UshrInt2addr(_) => todo!(),
				ASTInstruction::AddLong2addr(_) => todo!(),
				ASTInstruction::SubLong2addr(_) => todo!(),
				ASTInstruction::MulLong2addr(_) => todo!(),
				ASTInstruction::DivLong2addr(_) => todo!(),
				ASTInstruction::RemLong2addr(_) => todo!(),
				ASTInstruction::AndLong2addr(_) => todo!(),
				ASTInstruction::OrLong2addr(_) => todo!(),
				ASTInstruction::XorLong2addr(_) => todo!(),
				ASTInstruction::ShlLong2addr(_) => todo!(),
				ASTInstruction::ShrLong2addr(_) => todo!(),
				ASTInstruction::UshrLong2addr(_) => todo!(),
				ASTInstruction::AddFloat2addr(_) => todo!(),
				ASTInstruction::SubFloat2addr(_) => todo!(),
				ASTInstruction::MulFloat2addr(_) => todo!(),
				ASTInstruction::DivFloat2addr(_) => todo!(),
				ASTInstruction::RemFloat2addr(_) => todo!(),
				ASTInstruction::AddDouble2addr(_) => todo!(),
				ASTInstruction::SubDouble2addr(_) => todo!(),
				ASTInstruction::MulDouble2addr(_) => todo!(),
				ASTInstruction::DivDouble2addr(_) => todo!(),
				ASTInstruction::RemDouble2addr(_) => todo!(),
				ASTInstruction::AddIntLit16(_) => todo!(),
				ASTInstruction::RsubInt(_) => todo!(),
				ASTInstruction::MulIntLit16(_) => todo!(),
				ASTInstruction::DivIntLit16(_) => todo!(),
				ASTInstruction::RemIntLit16(_) => todo!(),
				ASTInstruction::AndIntLit16(_) => todo!(),
				ASTInstruction::OrIntLit16(_) => todo!(),
				ASTInstruction::XorIntLit16(_) => todo!(),
				ASTInstruction::AddIntLit8(generated::IF22b(v_dest, v_src, lit)) => fnb.bin_op_lit(
					instructions::BinOpLitKind::AddInt,
					*v_dest,
					*v_src,
					*lit as i8 as i16,
				),
				ASTInstruction::RsubIntLit8(_) => todo!(),
				ASTInstruction::MulIntLit8(_) => todo!(),
				ASTInstruction::DivIntLit8(generated::IF22b(v_dest, v_src, lit)) => fnb.bin_op_lit(
					instructions::BinOpLitKind::DivInt,
					*v_dest,
					*v_src,
					*lit as i8 as i16,
				),
				ASTInstruction::RemIntLit8(generated::IF22b(v_dest, v_src, lit)) => fnb.bin_op_lit(
					instructions::BinOpLitKind::DivInt,
					*v_dest,
					*v_src,
					*lit as i8 as i16,
				),
				ASTInstruction::AndIntLit8(_) => todo!(),
				ASTInstruction::OrIntLit8(_) => todo!(),
				ASTInstruction::XorIntLit8(_) => todo!(),
				ASTInstruction::ShlIntLit8(_) => todo!(),
				ASTInstruction::ShrIntLit8(_) => todo!(),
				ASTInstruction::UshrIntLit8(_) => todo!(),
				ASTInstruction::IgetVolatile(_) => todo!(),
				ASTInstruction::IputVolatile(_) => todo!(),
				ASTInstruction::SgetVolatile(_) => todo!(),
				ASTInstruction::SputVolatile(_) => todo!(),
				ASTInstruction::IgetObjectVolatile(_) => todo!(),
				ASTInstruction::IgetWideVolatile(_) => todo!(),
				ASTInstruction::IputWideVolatile(_) => todo!(),
				ASTInstruction::SgetWideVolatile(_) => todo!(),
				ASTInstruction::SputWideVolatile(_) => todo!(),
				ASTInstruction::Breakpoint(_) => todo!(),
				ASTInstruction::ThrowVerificationError(_) => todo!(),
				ASTInstruction::ExecuteInline(_) => todo!(),
				ASTInstruction::ExecuteInlineRange(_) => todo!(),
				ASTInstruction::InvokeObjectInitRange(_) => todo!(),
				ASTInstruction::ReturnVoidBarrier(_) => todo!(),
				ASTInstruction::IgetQuick(_) => todo!(),
				ASTInstruction::IgetWideQuick(_) => todo!(),
				ASTInstruction::IgetObjectQuick(_) => todo!(),
				ASTInstruction::IputQuick(_) => todo!(),
				ASTInstruction::IputWideQuick(_) => todo!(),
				ASTInstruction::IputObjectQuick(_) => todo!(),
				ASTInstruction::InvokeVirtualQuick(_) => todo!(),
				ASTInstruction::InvokeVirtualQuickRange(_) => todo!(),
				ASTInstruction::InvokeSuperQuick(_) => todo!(),
				ASTInstruction::InvokeSuperQuickRange(_) => todo!(),
				ASTInstruction::IputObjectVolatile(_) => todo!(),
				ASTInstruction::SgetObjectVolatile(_) => todo!(),
				ASTInstruction::SputObjectVolatile(_) => todo!(),
			}
		}

		last_target = *i;
	}
}

fn get_method_full_name(m_id: usize, dex: &DexFile) -> String {
	let (c_name, m_name) = match dex.file_data.methods.get(m_id) {
		None => todo!("handle errors"),
		Some(v) => (v.definer.clone(), v.name.clone()),
	};

	format!("{}__{}", format_classname(&c_name), format_name(&m_name))
}

fn get_field_name(f_ref_idx: usize, is_static: bool, dex: &DexFile) -> String {
	match dex.file_data.fields.get(f_ref_idx) {
		None => unimplemented!(),
		Some(v) => {
			if is_static {
				format!(
					"{}__{}",
					format_classname(&*v.definer),
					format_name(&*v.name)
				)
			} else {
				(*v.name).clone()
			}
		}
	}
}
