//! Tests for contracts that should cause compile errors

use fe_analyzer::namespace::items::{IngotId, ModuleId};
use fe_analyzer::TestDb;
use fe_common::diagnostics::diagnostics_string;
use fe_common::files::FileStore;
use insta::assert_snapshot;
use test_files::build_filestore;
use wasm_bindgen_test::wasm_bindgen_test;

fn error_string(path: &str, src: &str) -> String {
    let mut files = FileStore::new();
    let id = files.add_file(path, src);
    let deps = files.add_included_libraries();

    let db = TestDb::default();

    let module_id = ModuleId::try_new(&db, &files, id, &deps)
        .expect("failed to create new module")
        .value;

    match fe_analyzer::analyze_module(&db, module_id) {
        Ok(_) => panic!("expected analysis to fail with an error"),
        Err(diags) => diagnostics_string(&diags, &files),
    }
}

fn error_string_ingot(path: &str) -> String {
    let mut files = build_filestore(path);
    let file_ids = files.all_files();
    let deps = files.add_included_libraries();

    let db = TestDb::default();

    let ingot_id = IngotId::try_new(&db, &files, path, &file_ids, &deps)
        .expect("failed to create new ingot")
        .value;

    match fe_analyzer::analyze_ingot(&db, ingot_id) {
        Ok(_) => panic!("expected analysis to fail with an error"),
        Err(diags) => diagnostics_string(&diags, &files),
    }
}

macro_rules! test_ingot {
    ($name:ident) => {
        #[test]
        #[wasm_bindgen_test]
        fn $name() {
            let path = concat!("compile_errors/", stringify!($name));

            if cfg!(target_arch = "wasm32") {
                fe_common::assert_snapshot_wasm!(
                    concat!("snapshots/errors__", stringify!($name), ".snap"),
                    error_string_ingot(&path)
                );
            } else {
                assert_snapshot!(error_string_ingot(&path));
            }
        }
    };
}

macro_rules! test_file {
    ($name:ident) => {
        #[test]
        #[wasm_bindgen_test]
        fn $name() {
            let path = concat!("compile_errors/", stringify!($name), ".fe");
            let src = test_files::fixture(path);
            if cfg!(target_arch = "wasm32") {
                fe_common::assert_snapshot_wasm!(
                    concat!("snapshots/errors__", stringify!($name), ".snap"),
                    error_string(&path, &src)
                );
            } else {
                assert_snapshot!(error_string(&path, &src));
            }
        }
    };
}

macro_rules! test_stmt {
    ($name:ident, $stmt:expr) => {
        #[test]
        #[wasm_bindgen_test]
        fn $name() {
            let src = format!(
                "contract C:\n pub fn f(self):\n  {}",
                $stmt.replace('\n', "\n  ")
            );
            if cfg!(target_arch = "wasm32") {
                fe_common::assert_snapshot_wasm!(
                    concat!("snapshots/errors__", stringify!($name), ".snap"),
                    error_string("[snippet]", &src)
                );
            } else {
                assert_snapshot!(error_string("[snippet]", &src));
            }
        }
    };
}

test_stmt! { array_non_primitive, "let x: Array<(u8, u8), 10>" }
test_stmt! { array_mixed_types, "let x: Array<u16, 3> = [1, address(0), \"hi\"]" }
test_stmt! { array_size_mismatch, "let x: Array<u8, 3> = []\nlet y: Array<u8, 3> = [1, 2]" }
test_stmt! { array_constructor_call, "u8[3]([1, 2, 3])" }
test_stmt! { assert_reason_not_string, "assert true, 1" }
test_stmt! { assign_int, "5 = 6" }
test_stmt! { assign_call, "self.f() = 10" }
test_stmt! { assign_type_mismatch, "let x: u256 = 10\nx = address(0)" }
test_stmt! { aug_assign_non_numeric, "let a: u256 = 1\nlet b: bool = true\na += b" }
test_stmt! { binary_op_add_uints, "let a: u256 = 1\nlet b: u8 = 2\na + b" }
test_stmt! { binary_op_lshift_bool, "let a: bool = true\nlet b: i256\na << b" }
test_stmt! { binary_op_lshift_with_int, "let a: u256 = 1\nlet b: i256 = 2\na << b" }
test_stmt! { binary_op_pow_int, "let a: u256 = 1\nlet b: i256 = 2\na ** b" }
test_stmt! { binary_op_boolean_mismatch1, "10 and true" }
test_stmt! { binary_op_boolean_mismatch2, "false or 1" }
test_stmt! { binary_op_boolean_mismatch3, "1 or 2" }
test_stmt! { bool_constructor, "bool(true)" }
test_stmt! { bool_cast, "bool(0)" }
test_stmt! { break_without_loop, "break" }
test_stmt! { break_without_loop_2, "if true:\n  break" }
test_stmt! { call_undefined_function_on_contract, "self.doesnt_exist()" }
test_stmt! { call_address_with_wrong_type, "address(true)" }
test_stmt! { call_keccak_without_parameter, "keccak256()" }
test_stmt! { call_keccak_with_wrong_type, "keccak256(true)" }
test_stmt! { call_keccak_with_2_args, "keccak256(1, 2)" }
test_stmt! { call_keccak_with_generic_args, "keccak256<10>(1)" }
test_stmt! { cast_address_to_u64, "u64(address(0))" }

test_stmt! { call_balance_of_without_parameter, "balance_of()" }
test_stmt! { call_balance_of_with_wrong_type, "balance_of(true)" }
test_stmt! { call_balance_of_with_2_args, "balance_of(address(0), 2)" }
test_stmt! { call_balance_of_with_generic_args, "balance_of<10>(address(0))" }
test_stmt! { call_balance_with_arg, "balance(address(0))" }
test_stmt! { call_balance_with_generic_args, "balance<10>()" }
test_stmt! { call_send_value_without_parameter, "send_value()" }
test_stmt! { call_send_value_with_1_arg, "send_value(address(0))" }
test_stmt! { call_send_value_with_3_args, "send_value(address(0), 0, 0)" }
test_stmt! { call_send_value_with_wrong_type, "send_value(true, 0)" }
test_stmt! { call_send_value_with_wrong_type2, "send_value(address(0), true)" }
test_stmt! { call_send_value_with_generic_args, "send_value<10>(address(0), 1)" }
test_stmt! { clone_arg_count, "let x: Array<u256, 2> = [5, 6]\nlet y: Array<u256, 2> = x.clone(y)" }
test_stmt! { continue_without_loop, "continue" }
test_stmt! { continue_without_loop_2, "if true:\n  continue" }
test_stmt! { emit_undefined_event, "emit MyEvent()" }
test_stmt! { emit_type_name, "emit u8()" }
test_stmt! { emit_variable, "let x: u8 = 10\nemit x()" }
test_stmt! { int_type_generic_arg_list, "let x: u256<>" }
test_stmt! { int_type_generic_arg, "let x: u256<10>" }
test_stmt! { int_type_constructor_generic_arg_list, "u256<>(10)" }
test_stmt! { int_type_constructor_generic_arg, "u256<1>(10)" }
test_stmt! { map_three_type_args, "let x: Map<u8, u8, u8>" }
test_stmt! { map_int_type_arg, "let x: Map<address, 100>" }
test_stmt! { map_int_type_args, "let x: Map<10, 20>" }
test_stmt! { map_no_type_args, "let x: Map<>" }
test_stmt! { map_no_type_arg_list, "let x: Map" }
test_stmt! { map_one_type_arg, "let x: Map<y>" }
test_stmt! { map_map_key_type, "let x: Map<Map<u8, u8>, address>" }
test_stmt! { map_constructor, "Map<u8, u8>()" }
test_stmt! { non_bool_and, "let x: bool = true\nlet y: u256 = 1\nx = x and y" }
test_stmt! { non_bool_or, "let x: bool = true\nlet y: u256 = 1\nx = x or y" }
test_stmt! { overflow_i128_neg, "i128(-170141183460469231731687303715884105729)" }
test_stmt! { overflow_i128_pos, "i128(170141183460469231731687303715884105728)" }
test_stmt! { overflow_i16_neg, "i16(-32769)" }
test_stmt! { overflow_i16_pos, "i16(32768)" }
test_stmt! { overflow_i256_neg, "i256(-57896044618658097711785492504343953926634992332820282019728792003956564819969)" }
test_stmt! { overflow_i256_pos, "i256(57896044618658097711785492504343953926634992332820282019728792003956564819968)" }
test_stmt! { overflow_i32_neg, "i32(-2147483649)" }
test_stmt! { overflow_i32_pos, "i32(2147483648)" }
test_stmt! { overflow_i64_neg, "i64(-9223372036854775809)" }
test_stmt! { overflow_i64_pos, "i64(9223372036854775808)" }
test_stmt! { overflow_i8_neg, "let x: i8 = -129" }
test_stmt! { overflow_i8_pos, "let x: i8 = 128" }
test_stmt! { overflow_literal_too_big, "115792089237316195423570985008687907853269984665640564039457584007913129639936" }
test_stmt! { overflow_literal_too_small, "-115792089237316195423570985008687907853269984665640564039457584007913129639936" }
test_stmt! { overflow_u128_neg, "let x: u128 = -1" }
test_stmt! { overflow_u128_pos, "u128(340282366920938463463374607431768211456)" }
test_stmt! { overflow_u16_neg, "u16(-1)" }
test_stmt! { overflow_u16_pos, "u16(65536)" }
test_stmt! { overflow_u256_neg, "u256(-1)" }
test_stmt! { overflow_u256_pos, "u256(115792089237316195423570985008687907853269984665640564039457584007913129639936)" }
test_stmt! { overflow_u32_neg, "u32(-1)" }
test_stmt! { overflow_u32_pos, "u32(4294967296)" }
test_stmt! { overflow_u64_neg, "u64(-1)" }
test_stmt! { overflow_u64_pos, "u64(18446744073709551616)" }
test_stmt! { overflow_u8_neg, "u8(-1)" }
test_stmt! { overflow_u8_pos, "u8(256)" }
test_stmt! { overflow_u8_assignment, "let x: u8 = 260" }
test_stmt! { pow_with_signed_exponent, "let base: i128\nlet xp: i128\nbase ** exp" }
// Exponent can be unsigned but needs to be same size or smaller
test_stmt! { pow_with_wrong_capacity, "let base: i128\nlet exp: u256\nbase ** exp" }
test_stmt! { shadow_builtin_type_with_var, "let u8: u8 = 10" }
test_stmt! { shadow_builtin_fn_with_var, "let keccak256: u8 = 10" }
test_file! { shadow_builtin_type }
test_file! { shadow_builtin_function }
test_file! { self_misuse }
test_stmt! { string_capacity_mismatch, "String<3>(\"too long\")" }
test_stmt! { string_non_int_type_arg, "let x: String<u8>" }
test_stmt! { string_no_type_arg_list, "let x: String" }
test_stmt! { string_no_type_args, "let x: String<>" }
test_stmt! { string_two_int_type_args, "let x: String<1, 2>" }
test_stmt! { string_two_type_args, "let x: String<1, u8>" }
test_stmt! { string_constructor_non_int_type_arg, "String<foo>()" } // issue #532
test_stmt! { string_constructor_no_type_arg_list, "String()" }
test_stmt! { string_constructor_no_type_args, "String<>()" }
test_stmt! { string_constructor_two_int_type_args, "String<1, 2>()" }
test_stmt! { string_constructor_two_type_args, "String<1, u8>()" }
test_stmt! { ternary_type_mismatch, "10 if 100 else true" }
test_stmt! { change_sign_and_size_in_cast, "let x: i8\nlet y: u16 = u16(x)" }
test_stmt! { change_sign_and_type_in_cast, "let x: bool\nlet y: u16 = u16(x)" }
test_stmt! { type_constructor_arg_count, "let x: u8 = u8(1, 10)" }
test_stmt! { unary_minus_on_bool, "let x: bool = true\n-x" }
test_stmt! { unary_not_on_int, "let x: u256 = 10\nnot x" }
test_stmt! { undefined_generic_type, "let x: foobar<u256> = 10" }
test_stmt! { undefined_name, "let x: u16 = y\nlet z: u16 = y" }
test_stmt! { undefined_type, "let x: foobar = 10" }
test_stmt! { unexpected_return, "return 1" }
test_stmt! { unit_type_constructor, "()()" }
test_stmt! { revert_reason_not_struct, "revert 1" }
test_stmt! { invalid_ascii, "String<2>(\"ä\")" }
test_stmt! { invert_non_numeric, "~true" }

test_file! { bad_tuple_attr1 }
test_file! { bad_tuple_attr2 }
test_file! { bad_tuple_attr3 }
test_file! { call_builtin_object }
test_file! { call_create_with_wrong_type }
test_file! { call_create2_with_wrong_type }
test_file! { call_event_with_wrong_types }
test_file! { call_undefined_function_on_external_contract }
test_file! { call_undefined_function_on_memory_struct }
test_file! { call_undefined_function_on_storage_struct }
test_file! { call_non_pub_fn_on_external_contract }
test_file! { cannot_move }
test_file! { cannot_move2 }
test_file! { circular_dependency_create }
test_file! { circular_dependency_create2 }
test_file! { circular_type_alias }
test_file! { const_assign }
test_file! { const_generics_param }
test_file! { const_local }
test_file! { duplicate_arg_in_contract_method }
test_file! { duplicate_contract_in_module }
test_file! { duplicate_event_in_contract }
test_file! { duplicate_field_in_contract }
test_file! { duplicate_field_in_struct }
test_file! { duplicate_method_in_contract }
test_file! { duplicate_struct_in_module }
test_file! { duplicate_typedef_in_module }
test_file! { duplicate_var_in_child_scope }
test_file! { duplicate_var_in_contract_method }
test_file! { duplicate_var_in_for_loop }
test_file! { emit_bad_args }
test_file! { external_call_type_error }
test_file! { external_call_wrong_number_of_params }
test_file! { indexed_event }
test_file! { invalid_compiler_version }
test_file! { invalid_block_field }
test_file! { invalid_chain_field }
test_file! { invalid_contract_field }
test_file! { invalid_msg_field }
test_file! { invalid_string_field }
test_file! { invalid_struct_field }
test_file! { invalid_tuple_field }
test_file! { invalid_tx_field }
test_file! { invalid_var_declaration_1 }
test_file! { invalid_var_declaration_2 }
test_file! { issue_451 }
test_file! { mislabeled_call_args }
test_file! { mislabeled_call_args_self }
test_file! { mislabeled_call_args_external_contract_call }
test_file! { mismatch_return_type }
test_file! { missing_return }
test_file! { missing_return_in_else }
test_file! { missing_return_after_if }
test_file! { module_const_unknown_type }
test_file! { module_const_non_base_type }
test_file! { module_const_call }
test_file! { needs_mem_copy }
test_file! { not_callable }
test_file! { not_in_scope }
test_file! { not_in_scope_2 }
test_file! { private_struct_field }
test_file! { return_addition_with_mixed_types }
test_file! { return_call_to_fn_with_param_type_mismatch }
test_file! { return_call_to_fn_without_return }
test_file! { return_from_init }
test_file! { return_lt_mixed_types }
test_file! { return_type_undefined }
test_file! { return_complex_struct }
test_file! { return_type_not_fixedsize }
test_file! { undefined_type_param }

test_file! { strict_boolean_if_else }
test_file! { struct_private_constructor }
test_file! { struct_call_bad_args }
test_file! { struct_call_without_kw_args }
test_file! { non_pub_init }
test_file! { init_wrong_return_type }
test_file! { init_duplicate_def }
test_file! { init_call_on_self }
test_file! { init_call_on_external_contract }
test_file! { call_wrong_return_type }
test_file! { call_duplicate_def }
test_file! { call_call_on_self }
test_file! { call_call_on_external_contract }
test_file! { call_with_pub_fns }
test_file! { abi_encode_u256 }
test_file! { abi_encode_from_storage }
test_file! { assert_sto_msg_no_copy }
test_file! { for_loop_sto_iter_no_copy }
test_file! { revert_sto_error_no_copy }

test_file! { call_to_mut_fn_without_self }
test_file! { call_to_pure_fn_on_self }
test_file! { missing_self }
test_file! { self_not_first }
test_file! { self_in_standalone_fn }
test_file! { unsafe_misuse }
test_file! { unsafe_nesting }

test_ingot! { bad_ingot }
test_ingot! { mainless_ingot }
