// Copyright(c) Microsoft Corporation.
// Licensed under the MIT License.

#![no_std]
#![no_main]
extern crate alloc;

//==================================================================================================
// Imports
//==================================================================================================

use alloc::string::ToString;
use ::alloc::{
    string::String,
    vec::Vec,
};
use hyperlight_common::flatbuffer_wrappers::{function_types::ParameterType, util::{get_flatbuffer_result_from_vec, get_flatbuffer_result_from_void}};
use ::hyperlight_common::flatbuffer_wrappers::{
    function_call::FunctionCall,
    function_types::{
        ParameterValue,
        ReturnType,
    },
    guest_error::ErrorCode,
};
use hyperlight_guest::{guest_function_definition::GuestFunctionDefinition, guest_function_register::register_function};
use ::hyperlight_guest::{
    error::{
        HyperlightGuestError,
        Result,
    },
    host_function_call::{
        call_host_function,
        get_host_value_return_as_int,
        get_host_value_return_as_vecbytes,
    },
};

//==================================================================================================
// Standalone Functions
//==================================================================================================


fn direct_echo(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let ParameterValue::VecBytes(data) = function_call.parameters.clone().unwrap()[0].clone() {
        Ok(get_flatbuffer_result_from_vec(&data))
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to get_size_prefixed_buffer".to_string(),
        ))
    }
}

fn vmbus_echo(_function_call: &FunctionCall) -> Result<Vec<u8>> {
    match vmbus_func() {
        Ok(_) => {
            Ok(get_flatbuffer_result_from_void())
        }
        Err(_e) => {
            Err(HyperlightGuestError::new(
                ErrorCode::GuestError,
                "Error executing function".to_string(),
            ))
        }
    }
}

fn vmbus_func() -> Result<()> {
    let bytes: Vec<u8> = vmbus_read()?;
    let _result: i32 = vmbus_write(bytes)?;
    Ok(())
}

#[no_mangle]
pub extern "C" fn hyperlight_main() {
    let function_definition = GuestFunctionDefinition::new(
        "VmbusEcho".to_string(),
        Vec::new(),
        ReturnType::Void,
        vmbus_echo as i64,
    );
    register_function(function_definition);

    let direct_echo_function_definition= GuestFunctionDefinition::new(
        "DirectEcho".to_string(),
        Vec::from(&[ParameterType::VecBytes]),
        ReturnType::VecBytes,
        direct_echo as i64,
    );
    register_function(direct_echo_function_definition);
}

fn vmbus_read() -> Result<Vec<u8>> {
    call_host_function("VmbusRead", None, ReturnType::VecBytes)?;
    let result: Vec<u8> = get_host_value_return_as_vecbytes()?;
    Ok(result)
}

fn vmbus_write(data: Vec<u8>) -> Result<i32> {
    call_host_function(
        "VmbusWrite",
        Some(Vec::from(&[ParameterValue::VecBytes(data)])),
        ReturnType::Int,
    )?;
    let result: i32 = get_host_value_return_as_int()?;
    Ok(result)
}

#[no_mangle]
pub fn guest_dispatch_function(function_call: FunctionCall) -> Result<Vec<u8>> {
    let function_name: String = function_call.function_name.clone();
    return Err(HyperlightGuestError::new(ErrorCode::GuestFunctionNotFound, function_name));
}
