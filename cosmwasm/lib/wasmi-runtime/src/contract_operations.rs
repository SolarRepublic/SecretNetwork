use std::format;
use std::prelude::v1::*;

use enclave_ffi_types::{Ctx, EnclaveError, UserSpaceBuffer};

use super::imports;
use super::results::{HandleSuccess, InitSuccess, QuerySuccess};
use crate::exports;

use std::io::{self, Write};
use std::slice;
use std::string::String;
use std::vec::Vec;

extern crate wasmi;
use wasmi::{
    Error as InterpreterError, Externals, FuncInstance, FuncRef, HostError, ImportsBuilder,
    ModuleImportResolver, ModuleInstance, ModuleRef, RuntimeArgs, RuntimeValue, Signature, Trap,
    ValueType,
};

/// Safe wrapper around reads from the contract storage
fn read_db(context: Ctx, key: &[u8]) -> Option<Vec<u8>> {
    unsafe { exports::recover_buffer(imports::ocall_read_db(context, key.as_ptr(), key.len())) }
}

/// Safe wrapper around writes to the contract storage
fn write_db(context: Ctx, key: &[u8], value: &[u8]) {
    unsafe {
        imports::ocall_write_db(
            context,
            key.as_ptr(),
            key.len(),
            value.as_ptr(),
            value.len(),
        )
    }
}

pub fn init(
    context: Ctx,    // need to pass this to read_db & write_db
    contract: &[u8], // contract wasm bytes
    env: &[u8],      // blockchain state
    msg: &[u8],      // probably function call and args
) -> Result<InitSuccess, EnclaveError> {
    // Load wasm binary and prepare it for instantiation.
    let module = wasmi::Module::from_buffer(contract).map_err(|_err| EnclaveError::InvalidWasm)?;

    // Create new imports resolver.
    // These are the signatures of rust functions available to invoke from wasm code.
    let imports = ImportsBuilder::new().with_resolver("env", &ResolveAll);

    // Instantiate a module with our imports and assert that there is no `start` function.
    /*
    if self.loaded_module.module().start_section().is_some() {
           panic!("assert_no_start called on module with `start` function");
    }
    */
    let not_started_instance =
        ModuleInstance::new(&module, &imports).map_err(|_err| EnclaveError::InvalidWasm)?;
    if not_started_instance.has_start() {
        return Err(EnclaveError::WasmModuleWithStart);
    }
    let instance = not_started_instance.not_started_instance();

    let mut runtime = Runtime {};
    //.invoke_export("allocate" env size
    // copy env to that pointer (figure out what wasmi returns and translate that pointer to my memory space)
    //.invoke_export("allocate" msg size
    // copy msg to that pointer  (figure out what wasmi returns and translate that pointer to my memory space)

    //.invoke_export("init" with both pointers that we got from allocate

    let x = instance
        .invoke_export("init", &[], &mut runtime)
        .map_err(|_err| EnclaveError::FailedFunctionCall)?; // TODO return _err to user
    todo!()
}

pub fn handle(
    context: Ctx,
    contract: &[u8],
    env: &[u8],
    msg: &[u8],
) -> Result<HandleSuccess, EnclaveError> {
    todo!()
    // init wasmi - maybe the same as init for now?
}

pub fn query(context: Ctx, contract: &[u8], msg: &[u8]) -> Result<QuerySuccess, EnclaveError> {
    todo!()
    // init wasmi
    // no access to write_db
}

// --------------------------------
// Functions to expose to WASM code
// --------------------------------
// TODO find better names for "Runtime" and "ResolveAll"

// ResolveAll maps function name to its function signature and also to function index in Runtime
// When instansiating a module we give it this resolver
// When invoking a function inside the module we can give it different runtimes (which we probably won't do)
struct ResolveAll;

// These functions
// fn read_db(key: *const c_void, value: *mut c_void) -> i32;
// fn write_db(key: *const c_void, value: *mut c_void);
// fn canonicalize_address(human: *const c_void, canonical: *mut c_void) -> i32;
// fn humanize_address(canonical: *const c_void, human: *mut c_void) -> i32;
impl ModuleImportResolver for ResolveAll {
    fn resolve_func(
        &self,
        func_name: &str,
        signature: &Signature,
    ) -> Result<FuncRef, InterpreterError> {
        let func_ref = match func_name {
            "read_db" => FuncInstance::alloc_host(
                Signature::new(&[][..], Some(ValueType::I32)),
                READ_DB_INDEX,
            ),
            "write_db" => FuncInstance::alloc_host(Signature::new(&[][..], None), WRITE_DB_INDEX),
            "canonicalize_address" => FuncInstance::alloc_host(
                Signature::new(&[][..], Some(ValueType::I32)),
                CANONICALIZE_ADDRESS_INDEX,
            ),
            "humanize_address" => FuncInstance::alloc_host(
                Signature::new(&[][..], Some(ValueType::I32)),
                HUMANIZE_ADDRESS_INDEX,
            ),
            _ => {
                return Err(InterpreterError::Function(format!(
                    "host module doesn't export function with name {}",
                    func_name
                )));
            }
        };
        Ok(func_ref)
    }
}

// Runtime maps function index to implementation
// When instansiating a module we give it the ResolveAll resolver
// When invoking a function inside the module we give it this runtime which is the acctual functions implementation ()
struct Runtime;

const READ_DB_INDEX: usize = 0;
const WRITE_DB_INDEX: usize = 1;
const CANONICALIZE_ADDRESS_INDEX: usize = 2;
const HUMANIZE_ADDRESS_INDEX: usize = 3;

impl Externals for Runtime {
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        match index {
            READ_DB_INDEX => Ok(Some(RuntimeValue::I32(2))), // TODO implement
            WRITE_DB_INDEX => Ok(Some(RuntimeValue::I32(2))), // TODO implement
            CANONICALIZE_ADDRESS_INDEX => Ok(Some(RuntimeValue::I32(2))), // TODO implement
            HUMANIZE_ADDRESS_INDEX => Ok(Some(RuntimeValue::I32(2))), // TODO implement
            _ => panic!("unknown function index"),
        }
    }
}
