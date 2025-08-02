# CKB Syscalls Complete Reference

## Description

Complete CKB syscalls reference with high-level function mappings, core load operations, and practical script development patterns. Features comprehensive tables mapping ckb-std high-level functions to syscalls, source constants, cell field constants, error handling patterns, and common validation examples. Essential for CKB smart contract development and script optimization.

## High-Level vs Syscall Mapping Table

| CKB-STD High-Level Function | CKB-STD Syscall Function | Description |
|------------------------------|---------------------------|-------------|
| [load_script()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/high_level/fn.load_script.html) | [load_script()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/syscalls/fn.load_script.html) | Load currently executing script |
| [load_script_hash()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/high_level/fn.load_script_hash.html) | [load_script_hash()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/syscalls/fn.load_script_hash.html) | Load hash of current script |
| [load_transaction()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/high_level/fn.load_transaction.html) | [load_transaction()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/syscalls/fn.load_transaction.html) | Load entire transaction structure |
| [load_tx_hash()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/high_level/fn.load_tx_hash.html) | [load_tx_hash()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/syscalls/fn.load_tx_hash.html) | Load transaction hash |
| [load_cell()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/high_level/fn.load_cell.html) | [load_cell()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/syscalls/fn.load_cell.html) | Load cell by index and source |
| [load_cell_data()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/high_level/fn.load_cell_data.html) | [load_cell_data()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/syscalls/fn.load_cell_data.html) | Load cell data field |
| [load_cell_capacity()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/high_level/fn.load_cell_capacity.html) | [load_cell_by_field()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/syscalls/fn.load_cell_by_field.html) | Load cell capacity field |
| [load_cell_lock()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/high_level/fn.load_cell_lock.html) | [load_cell_by_field()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/syscalls/fn.load_cell_by_field.html) | Load cell lock script |
| [load_cell_lock_hash()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/high_level/fn.load_cell_lock_hash.html) | [load_cell_by_field()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/syscalls/fn.load_cell_by_field.html) | Load cell lock script hash |
| [load_cell_type()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/high_level/fn.load_cell_type.html) | [load_cell_by_field()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/syscalls/fn.load_cell_by_field.html) | Load cell type script |
| [load_cell_type_hash()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/high_level/fn.load_cell_type_hash.html) | [load_cell_by_field()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/syscalls/fn.load_cell_by_field.html) | Load cell type script hash |
| [load_cell_data_hash()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/high_level/fn.load_cell_data_hash.html) | [load_cell_by_field()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/syscalls/fn.load_cell_by_field.html) | Load cell data hash |
| [load_cell_occupied_capacity()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/high_level/fn.load_cell_occupied_capacity.html) | [load_cell_by_field()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/syscalls/fn.load_cell_by_field.html) | Load occupied capacity |
| [load_input()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/high_level/fn.load_input.html) | [load_input()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/syscalls/fn.load_input.html) | Load input by index |
| [load_input_out_point()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/high_level/fn.load_input_out_point.html) | [load_input_by_field()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/syscalls/fn.load_input_by_field.html) | Load input out point field |
| [load_input_since()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/high_level/fn.load_input_since.html) | [load_input_by_field()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/syscalls/fn.load_input_by_field.html) | Load input since field |
| [load_header()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/high_level/fn.load_header.html) | [load_header()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/syscalls/fn.load_header.html) | Load block header |
| [load_header_epoch_number()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/high_level/fn.load_header_epoch_number.html) | [load_header_by_field()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/syscalls/fn.load_header_by_field.html) | Load header epoch number |
| [load_header_epoch_start_block_number()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/high_level/fn.load_header_epoch_start_block_number.html) | [load_header_by_field()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/syscalls/fn.load_header_by_field.html) | Load epoch start block |
| [load_header_epoch_length()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/high_level/fn.load_header_epoch_length.html) | [load_header_by_field()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/syscalls/fn.load_header_by_field.html) | Load epoch length |
| [load_witness_args()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/high_level/fn.load_witness_args.html) | [load_witness()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/syscalls/fn.load_witness.html) | Load structured witness |
| [find_cell_by_data_hash()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/high_level/fn.find_cell_by_data_hash.html) | N/A | Find cell by data hash |
| N/A | [load_cell_code()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/syscalls/fn.load_cell_code.html) | Load executable code |
| N/A | [load_cell_data_raw()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/syscalls/fn.load_cell_data_raw.html) | Load raw cell data |
| N/A | [debug()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/syscalls/fn.debug.html) | Debug output |
| N/A | [exit()](https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/syscalls/fn.exit.html) | Exit script execution |

## Core Load Functions

### Script and Transaction Data
```rust
use ckb_std::high_level::*;

// Load current script information
let script = load_script()?;
let args: Bytes = script.args().unpack();

// Load transaction hash
let tx_hash = load_tx_hash()?;

// Load transaction structure
let transaction = load_transaction()?;
```

### Cell Operations
```rust
// Load cell by index and source
let cell = load_cell(index, source)?;

// Load cell data only
let data = load_cell_data(index, source)?;

// Load specific cell field
let capacity = load_cell_by_field(index, source, CellField::Capacity)?;
let lock_hash = load_cell_by_field(index, source, CellField::LockHash)?;
```

### Witness Operations
```rust
// Load witness by index
let witness = load_witness(index, source)?;

// Load structured witness
let witness_args = load_witness_args(index, source)?;
let lock_field = witness_args.lock().to_opt();
let input_type = witness_args.input_type().to_opt();
let output_type = witness_args.output_type().to_opt();
```

## Source Constants
```rust
use ckb_std::ckb_constants::Source;

Source::Input           // All input cells
Source::Output          // All output cells  
Source::CellDep         // All dep cells
Source::HeaderDep       // All header deps
Source::GroupInput      // Input cells with same script
Source::GroupOutput     // Output cells with same script
```

## Cell Field Constants
```rust
use ckb_std::ckb_constants::CellField;

CellField::Capacity          // Cell capacity (CKB amount)
CellField::DataHash         // Hash of cell data
CellField::Lock             // Lock script
CellField::LockHash         // Hash of lock script
CellField::Type             // Type script (optional)
CellField::TypeHash         // Hash of type script
CellField::OccupiedCapacity // Used capacity
```

## Error Handling Pattern
```rust
use ckb_std::error::SysError;

#[repr(i8)]
pub enum Error {
    IndexOutOfBound = 1,
    ItemMissing,
    LengthNotEnough,
    Encoding,
    // Custom errors start from 5+
    CustomError = 5,
}

impl From<SysError> for Error {
    fn from(err: SysError) -> Self {
        match err {
            SysError::IndexOutOfBound => Self::IndexOutOfBound,
            SysError::ItemMissing => Self::ItemMissing,
            SysError::LengthNotEnough(_) => Self::LengthNotEnough,
            SysError::Encoding => Self::Encoding,
            _ => Self::CustomError,
        }
    }
}
```

## Iteration Pattern
```rust
use ckb_std::high_level::QueryIter;

// Iterate over all input cells
for cell_data in QueryIter::new(load_cell_data, Source::Input) {
    // Process each cell data
}

// Iterate with manual index
let mut index = 0;
loop {
    match load_cell_data(index, Source::Input) {
        Ok(data) => {
            // Process data
            index += 1;
        }
        Err(SysError::IndexOutOfBound) => break,
        Err(e) => return Err(e.into()),
    }
}
```

## Common Validation Patterns
```rust
// Check if script args match expected pattern
let script = load_script()?;
let args = script.args().raw_data();
if args.len() != 20 {
    return Err(Error::InvalidArgs);
}

// Verify witness exists
let witness_args = load_witness_args(0, Source::GroupInput)?;
let signature = witness_args
    .lock()
    .to_opt()
    .ok_or(Error::MissingSignature)?;

// Compare cell data
let input_data = load_cell_data(0, Source::GroupInput)?;
let output_data = load_cell_data(0, Source::GroupOutput)?;
if input_data != output_data {
    return Err(Error::DataMismatch);
}
```