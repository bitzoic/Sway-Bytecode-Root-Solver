contract;

use std::{
    alloc::alloc_bytes,
    bytes::Bytes,
};

abi MyContract {
    fn swap_configurable(bytecode: Vec<u8>, config: Vec<(u64, Vec<u8>)>) -> Vec<u8>;
}

impl MyContract for Contract {
    fn swap_configurable(bytecode: Vec<u8>, config: Vec<(u64, Vec<u8>)>) -> Vec<u8> {
        // Copy the bytecode to a newly allocated memory to avoid memory ownership error.
        let mut bytecode_slice = raw_slice::from_parts::<u8>(alloc_bytes(bytecode.len()), bytecode.len());
        bytecode.buf.ptr.copy_bytes_to(
            bytecode_slice.ptr(), 
            bytecode.len()
        );

        // Iterate over every configurable
        let mut configurable_iterator = 0;
        while configurable_iterator < config.len() {
            let (offset, data) = config.get(configurable_iterator).unwrap();
            
            // Copy the configurable data into the bytecode
            data.buf.ptr.copy_bytes_to(
                bytecode_slice.ptr().add::<u8>(offset), 
                data.len()
            );

            configurable_iterator += 1;
        }

        Vec::from(bytecode_slice)
    }
}
