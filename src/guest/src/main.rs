// Copyright(c) Microsoft Corporation.
// Licensed under the MIT License.

#![no_std]
#![no_main]
extern crate alloc;

//==================================================================================================
// Imports
//==================================================================================================

use ::alloc::{
    string::String,
    vec::Vec,
};
use ::hyperlight_common::flatbuffer_wrappers::{
    function_call::FunctionCall,
    function_types::{
        ParameterValue,
        ReturnType,
    },
    guest_error::ErrorCode,
};
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

#[no_mangle]
pub extern "C" fn hyperlight_main() {
    let bytes: Vec<u8> = vmbus_read().unwrap();
    let _result: i32 = vmvus_write(bytes).unwrap();
}

fn vmbus_read() -> Result<Vec<u8>> {
    call_host_function("VmbusRead", None, ReturnType::VecBytes)?;
    let result: Vec<u8> = get_host_value_return_as_vecbytes()?;
    Ok(result)
}

fn vmvus_write(data: Vec<u8>) -> Result<i32> {
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
