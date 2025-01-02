// Copyright(c) Microsoft Corporation.
// Licensed under the MIT License.

#![no_std]
#![no_main]
extern crate alloc;

//==================================================================================================
// Imports
//==================================================================================================

use ::alloc::{
    string::{
        String,
        ToString,
    },
    vec::Vec,
};
use ::hyperlight_common::flatbuffer_wrappers::{
    function_call::FunctionCall,
    function_types::{
        ParameterType,
        ParameterValue,
        ReturnType,
    },
    guest_error::ErrorCode,
    util::get_flatbuffer_result_from_int,
};
use ::hyperlight_guest::{
    error::{
        HyperlightGuestError,
        Result,
    },
    guest_function_definition::GuestFunctionDefinition,
    guest_function_register::register_function,
    host_function_call::{
        call_host_function,
        get_host_value_return_as_int,
    },
};

//==================================================================================================
// Standalone Functions
//==================================================================================================

#[no_mangle]
pub extern "C" fn hyperlight_main() {
    let print_output_def: GuestFunctionDefinition = GuestFunctionDefinition::new(
        "PrintOutput".to_string(),
        Vec::from(&[ParameterType::String]),
        ReturnType::Int,
        print_output as i64,
    );
    register_function(print_output_def);
}

#[no_mangle]
pub fn guest_dispatch_function(function_call: FunctionCall) -> Result<Vec<u8>> {
    let function_name: String = function_call.function_name.clone();
    return Err(HyperlightGuestError::new(ErrorCode::GuestFunctionNotFound, function_name));
}

fn print_output(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let ParameterValue::String(message) = function_call.parameters.clone().unwrap()[0].clone() {
        call_host_function(
            "HostPrint",
            Some(Vec::from(&[ParameterValue::String(message.to_string())])),
            ReturnType::Int,
        )?;
        let result: i32 = get_host_value_return_as_int()?;
        Ok(get_flatbuffer_result_from_int(result))
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to simple_print_output".to_string(),
        ))
    }
}
