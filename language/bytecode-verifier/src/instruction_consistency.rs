// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines the transfer functions for verifying type safety of a procedure body.
//! It does not utilize control flow, but does check each block independently

use crate::binary_views::BinaryIndexedView;
use libra_types::vm_status::StatusCode;
use vm::{
    access::ModuleAccess,
    errors::{err_at_offset, VMResult},
    file_format::{
        Bytecode, CodeUnit, CompiledModule, CompiledScript, FieldHandleIndex, FunctionHandleIndex,
        StructDefinitionIndex,
    },
};

pub struct InstructionConsistency<'a> {
    resolver: BinaryIndexedView<'a>,
}

impl<'a> InstructionConsistency<'a> {
    pub fn verify_module(module: &'a CompiledModule) -> VMResult<()> {
        let checker = Self {
            resolver: BinaryIndexedView::Module(module),
        };
        for func_def in module.function_defs() {
            match &func_def.code {
                None => (),
                Some(code) => checker.check_instructions(code)?,
            }
        }
        Ok(())
    }

    pub fn verify_script(script: &'a CompiledScript) -> VMResult<()> {
        let checker = Self {
            resolver: BinaryIndexedView::Script(script),
        };
        checker.check_instructions(&script.as_inner().code)
    }

    fn check_instructions(&self, code: &CodeUnit) -> VMResult<()> {
        for (offset, instr) in code.code.iter().enumerate() {
            match instr {
                Bytecode::MutBorrowField(field_handle_index) => {
                    self.check_field_op(offset, *field_handle_index, /* generic */ false)?;
                }
                Bytecode::MutBorrowFieldGeneric(field_inst_index) => {
                    let field_inst = self.resolver.field_instantiation_at(*field_inst_index)?;
                    self.check_field_op(offset, field_inst.handle, /* generic */ true)?;
                }
                Bytecode::ImmBorrowField(field_handle_index) => {
                    self.check_field_op(offset, *field_handle_index, /* generic */ false)?;
                }
                Bytecode::ImmBorrowFieldGeneric(field_inst_index) => {
                    let field_inst = self.resolver.field_instantiation_at(*field_inst_index)?;
                    self.check_field_op(offset, field_inst.handle, /* non_ */ true)?;
                }
                Bytecode::Call(idx) => {
                    self.check_function_op(offset, *idx, /* generic */ false)?;
                }
                Bytecode::CallGeneric(idx) => {
                    let func_inst = self.resolver.function_instantiation_at(*idx);
                    self.check_function_op(offset, func_inst.handle, /* generic */ true)?;
                }
                Bytecode::Pack(idx) => {
                    self.check_type_op(offset, *idx, /* generic */ false)?;
                }
                Bytecode::PackGeneric(idx) => {
                    let struct_inst = self.resolver.struct_instantiation_at(*idx)?;
                    self.check_type_op(offset, struct_inst.def, /* generic */ true)?;
                }
                Bytecode::Unpack(idx) => {
                    self.check_type_op(offset, *idx, /* generic */ false)?;
                }
                Bytecode::UnpackGeneric(idx) => {
                    let struct_inst = self.resolver.struct_instantiation_at(*idx)?;
                    self.check_type_op(offset, struct_inst.def, /* generic */ true)?;
                }
                Bytecode::MutBorrowGlobal(idx) => {
                    self.check_type_op(offset, *idx, /* generic */ false)?;
                }
                Bytecode::MutBorrowGlobalGeneric(idx) => {
                    let struct_inst = self.resolver.struct_instantiation_at(*idx)?;
                    self.check_type_op(offset, struct_inst.def, /* generic */ true)?;
                }
                Bytecode::ImmBorrowGlobal(idx) => {
                    self.check_type_op(offset, *idx, /* generic */ false)?;
                }
                Bytecode::ImmBorrowGlobalGeneric(idx) => {
                    let struct_inst = self.resolver.struct_instantiation_at(*idx)?;
                    self.check_type_op(offset, struct_inst.def, /* generic */ true)?;
                }
                Bytecode::Exists(idx) => {
                    self.check_type_op(offset, *idx, /* generic */ false)?;
                }
                Bytecode::ExistsGeneric(idx) => {
                    let struct_inst = self.resolver.struct_instantiation_at(*idx)?;
                    self.check_type_op(offset, struct_inst.def, /* generic */ true)?;
                }
                Bytecode::MoveFrom(idx) => {
                    self.check_type_op(offset, *idx, /* generic */ false)?;
                }
                Bytecode::MoveFromGeneric(idx) => {
                    let struct_inst = self.resolver.struct_instantiation_at(*idx)?;
                    self.check_type_op(offset, struct_inst.def, /* generic */ true)?;
                }
                Bytecode::MoveTo(idx) => {
                    self.check_type_op(offset, *idx, /* generic */ false)?;
                }
                Bytecode::MoveToGeneric(idx) => {
                    let struct_inst = self.resolver.struct_instantiation_at(*idx)?;
                    self.check_type_op(offset, struct_inst.def, /* generic */ true)?;
                }
                _ => (),
            }
        }
        Ok(())
    }

    //
    // Helpers for instructions that come in a generic and non generic form.
    // Verifies the generic form uses a generic member and the non generic form
    // a non generic one.
    //

    fn check_field_op(
        &self,
        offset: usize,
        field_handle_index: FieldHandleIndex,
        generic: bool,
    ) -> VMResult<()> {
        let field_handle = self.resolver.field_handle_at(field_handle_index)?;
        self.check_type_op(offset, field_handle.owner, generic)
    }

    fn check_type_op(
        &self,
        offset: usize,
        struct_def_index: StructDefinitionIndex,
        generic: bool,
    ) -> VMResult<()> {
        let struct_def = self.resolver.struct_def_at(struct_def_index)?;
        let struct_handle = self.resolver.struct_handle_at(struct_def.struct_handle);
        if struct_handle.type_parameters.is_empty() == generic {
            return Err(err_at_offset(
                StatusCode::GENERIC_MEMBER_OPCODE_MISMATCH,
                offset,
            ));
        }
        Ok(())
    }

    fn check_function_op(
        &self,
        offset: usize,
        func_handle_index: FunctionHandleIndex,
        generic: bool,
    ) -> VMResult<()> {
        let function_handle = self.resolver.function_handle_at(func_handle_index);
        if function_handle.type_parameters.is_empty() == generic {
            return Err(err_at_offset(
                StatusCode::GENERIC_MEMBER_OPCODE_MISMATCH,
                offset,
            ));
        }
        Ok(())
    }
}
