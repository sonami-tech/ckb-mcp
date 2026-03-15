## Description

CKB programming model from first principles. Cells as generalized UTXOs, transaction-oriented programming (off-chain computation, on-chain validation), lock scripts vs type scripts, script execution and grouping, Sources and GroupInput/GroupOutput, cell dependencies, witness structure, capacity as storage, and the complete validation pipeline. Start here before reading any other CKB documentation.

## Core Model: Cells and Transactions

CKB is a UTXO-based blockchain. State is stored in **cells** (generalized UTXOs). Transactions consume existing cells and create new ones. There is no global mutable state or contract accounts.

### Cell Structure

```
Cell {
  capacity: u64       // CKB tokens AND storage limit (1 CKB = 1 byte)
  lock: Script        // Controls who can spend this cell
  type: Script?       // Optional: validates data rules
  data: Bytes         // Arbitrary application data
}
```

- **capacity** serves two roles: token value AND maximum storage size. The cell's total serialized size (capacity field + lock + type + data) must not exceed `capacity` bytes.
- **lock script** determines spending authorization. Runs only when the cell is consumed as a transaction input.
- **type script** (optional) enforces data validation rules. Runs when cells with this type appear in inputs OR outputs.
- **data** stores arbitrary bytes: token amounts, contract state, serialized structures.

### Minimum Cell: 61 CKBytes

```
8 (capacity field) + 33 (minimal lock script) + 20 (typical args) = 61 bytes
```

Adding a type script adds ~33-65 bytes. Adding data adds its byte length.

## Transaction-Oriented Programming

Unlike Ethereum where contracts execute on-chain to compute state changes, CKB separates computation from validation:

1. **Off-chain**: Build the transaction — determine inputs to consume, outputs to create
2. **On-chain**: Scripts only **validate** that the proposed state change is legal

```
Transaction {
  cell_deps:    [CellDep]     // Read-only reference cells (contain script code)
  header_deps:  [Byte32]      // Block headers for time/epoch access
  inputs:       [CellInput]   // Live cells to consume (become dead)
  outputs:      [CellOutput]  // New cells to create (become live)
  outputs_data: [Bytes]       // Data for each output cell
  witnesses:    [Bytes]       // Signatures, proofs, auxiliary data
}
```

**Key consequences:**
- Transaction outcome is known before submission
- Invalid transactions are rejected without cost
- Independent transactions execute in parallel
- No risk of partial execution or reentrancy

## Lock Scripts vs Type Scripts

### Lock Scripts — "Who can spend this cell?"

- Execute ONLY on inputs (cells being consumed)
- Typical use: verify a signature matches the cell owner
- Return 0 = spending authorized; non-zero = denied
- GroupOutput is EMPTY for lock scripts (they don't execute on outputs)

```rust
// Minimal lock script: verify preimage hash
fn main() -> Result<(), Error> {
    let script = load_script()?;
    let expected_hash = script.args().raw_data();

    let witness_args = load_witness_args(0, Source::GroupInput)?;
    let preimage = witness_args.lock().to_opt().ok_or(Error::NoWitness)?.raw_data();

    if blake2b_256(&preimage) == expected_hash.as_ref() {
        Ok(())
    } else {
        Err(Error::HashMismatch)
    }
}
```

### Type Scripts — "What rules govern this cell's data?"

- Execute on BOTH inputs AND outputs with this type script
- Typical use: enforce token conservation, validate state transitions
- GroupInput returns inputs with this type; GroupOutput returns outputs with this type

```rust
// Minimal type script: UDT conservation check
fn main() -> Result<(), Error> {
    let script = load_script()?;
    let owner_lock_hash = script.args().raw_data();

    // Owner mode: skip validation if owner is spending
    if QueryIter::new(load_cell_lock_hash, Source::Input)
        .any(|hash| hash[..] == owner_lock_hash[..]) {
        return Ok(());
    }

    // Conservation: input tokens >= output tokens
    let inputs: u128 = sum_amounts(Source::GroupInput)?;
    let outputs: u128 = sum_amounts(Source::GroupOutput)?;
    if outputs > inputs { return Err(Error::Inflation); }
    Ok(())
}
```

## Script Execution and Grouping

CKB groups cells by identical scripts (same code_hash + hash_type + args) and executes each unique script **once** for the entire group.

### Sources

```rust
Source::Input       // All transaction inputs
Source::Output      // All transaction outputs
Source::CellDep     // Cell dependencies
Source::HeaderDep   // Header dependencies
Source::GroupInput   // Inputs with SAME script as currently executing
Source::GroupOutput  // Outputs with SAME script as currently executing
```

### Critical: GroupOutput behavior differs by script type

| Script Type | GroupInput | GroupOutput |
|------------|-----------|-------------|
| Lock script | Inputs with same lock | **Always empty** (locks don't run on outputs) |
| Type script | Inputs with same type | Outputs with same type |

If a lock script needs to count outputs with its script, it must manually iterate `Source::Output` and compare lock hashes.

## Cell Dependencies (cell_deps)

Scripts are stored as data in cells on-chain. `cell_deps` tells the VM where to find script code:

```rust
CellDep {
    out_point: OutPoint,  // Points to cell containing code
    dep_type: DepType,    // Code (single cell) or DepGroup (bundle)
}
```

### Script Resolution

```rust
Script {
    code_hash: Byte32,    // Identifies which code to run
    hash_type: HashType,  // How to match code_hash to cell_dep
    args: Bytes,          // Arguments passed to the script
}
```

- **Data/Data1**: `code_hash` = hash of cell data → find cell_dep whose data hashes to this value
- **Type**: `code_hash` = hash of cell's type script → enables upgradeable contracts (code changes, hash stays same)

## Witnesses

Witnesses carry data for script validation (signatures, proofs, state transition data):

```rust
WitnessArgs {
    lock: Option<Bytes>,        // For lock script (e.g., signature)
    input_type: Option<Bytes>,  // For input type script
    output_type: Option<Bytes>, // For output type script
}
```

The transaction hash used for signing **excludes** witnesses (prevents circular dependency).

## Validation Pipeline

When a transaction is submitted, CKB validates in order:

1. **Structure check**: All referenced input cells must be live (unspent)
2. **Capacity conservation**: Sum(input capacities) ≥ Sum(output capacities). Difference = miner fee.
3. **Occupied capacity**: Each output cell's serialized size ≤ its capacity value
4. **Lock script execution**: For each unique lock script group in inputs, execute once
5. **Type script execution**: For each unique type script group in inputs AND outputs, execute once
6. All scripts must return 0 (success)

## Script Development Environment

Scripts compile to RISC-V (`riscv64imac-unknown-none-elf`) and run in CKB-VM:

```rust
#![no_std]
#![no_main]

ckb_std::entry!(program_entry);
ckb_std::default_alloc!();

pub fn program_entry() -> i8 {
    match main() {
        Ok(_) => 0,
        Err(err) => err as i8,
    }
}
```

Key syscalls via `ckb_std::high_level`:
- `load_script()` — current script and args
- `load_cell_data(index, source)` — cell data bytes
- `load_cell_lock_hash(index, source)` — lock script hash
- `load_witness_args(index, source)` — witness data
- `load_tx_hash()` — transaction hash for signing
- `QueryIter::new(loader, source)` — iterate cells

## Quick Reference

| Concept | CKB | Ethereum Equivalent |
|---------|-----|-------------------|
| State storage | Cells (UTXOs) | Account storage slots |
| Smart contract | Script (validates) | Contract (executes) |
| Computation | Off-chain | On-chain |
| Token balance | Cell data field | Contract mapping |
| Authorization | Lock script | msg.sender check |
| Data rules | Type script | Contract logic |
| Contract code | Stored in cell data, referenced via cell_dep | Deployed at address |
| Upgrades | Type hash_type (code changes, reference stable) | Proxy pattern |

## Related Documentation

- [Cell Model](ckb://docs/concepts/cell-model) — Detailed cell structure and patterns
- [Transaction Structure](ckb://docs/concepts/transaction-structure) — Full transaction anatomy
- [Script Groups](ckb://docs/concepts/script-groups) — Group execution mechanics
- [Syscalls](ckb://docs/concepts/syscalls) — Complete syscall reference
- [Minimal Lock Script](ckb://docs/scripts/lock-script-minimal) — Lock script template
- [Minimal Type Script](ckb://docs/scripts/type-script-minimal) — Type script template
