## Description

Comprehensive guide explaining how CKB smart contracts access time and epoch information through header dependencies. Covers the critical limitation that scripts cannot access current block headers, header_deps transaction field, syscalls for header access, temporal validation patterns, and security considerations. Essential for developers implementing time-based contracts like time-locks, vesting schedules, and epoch-based validation logic.

## Critical Limitation: No Current Header Access

**Scripts can ONLY access headers explicitly included in `header_deps` - there is no syscall to access the current block's header.** This fundamental constraint creates:

- **Temporal Gap**: Scripts always see "stale" time information (1+ blocks behind actual current state)
- **Transaction Builder Burden**: Clients must predict which headers scripts will need
- **Mempool Staleness**: Long-pending transactions have increasingly outdated header data
- **Precision Loss**: Time-critical contracts lack exact execution timing awareness
- **MEV Gaming Risk**: Miners can exploit header selection for strategic advantage
- **No Atomic Current-State Operations**: Cannot perform logic based on exact execution moment

**Design Trade-off**: CKB prioritizes **determinism** (essential for consensus) over **real-time accuracy** (useful for applications).

## Header Dependencies Mechanism

### Transaction Structure

Every CKB transaction includes a `header_deps` field containing block header hashes:

```rust
pub struct Transaction {
    pub version: Uint32,
    pub cell_deps: CellDepVec,
    pub header_deps: Byte32Vec,  // Block header hashes for script access
    pub inputs: CellInputVec,
    pub outputs: CellOutputVec,
    pub outputs_data: BytesVec,
    pub witnesses: BytesVec,
}
```

### Header Access Process

1. **Transaction Builder**: Determines which block headers contain needed epoch/time data
2. **Header Selection**: Includes relevant historical block header hashes in `header_deps`
3. **Script Execution**: Scripts use syscalls to access provided header data by index
4. **Temporal Validation**: Scripts implement time-based logic using historical header information

## Header Access Syscalls

### Core Syscalls

```c
// Load complete header structure
int ckb_load_header(void* addr, uint64_t* len, size_t offset, 
                   size_t index, size_t source);

// Load specific header field
int ckb_load_header_by_field(void* addr, uint64_t* len, size_t offset,
                            size_t index, size_t source, size_t field);
```

### Header Field Constants

- `CKB_HEADER_FIELD_EPOCH_NUMBER` (0) - Current epoch number
- `CKB_HEADER_FIELD_EPOCH_START_BLOCK_NUMBER` (1) - Epoch start block
- `CKB_HEADER_FIELD_EPOCH_LENGTH` (2) - Epoch duration in blocks
- `CKB_HEADER_FIELD_TIMESTAMP` (7) - Block timestamp (milliseconds)
- `CKB_HEADER_FIELD_NUMBER` (8) - Block number

### Practical Usage

```c
// Load epoch number from first header dependency
uint64_t epoch_number;
uint64_t len = 8;
int ret = ckb_load_header_by_field(&epoch_number, &len, 0, 0, 
                                  CKB_SOURCE_HEADER_DEP, 
                                  CKB_HEADER_FIELD_EPOCH_NUMBER);
if (ret != CKB_SUCCESS) {
    return ret;  // Handle error
}

// Load block timestamp
uint64_t timestamp;
len = 8;
ret = ckb_load_header_by_field(&timestamp, &len, 0, 0,
                              CKB_SOURCE_HEADER_DEP,
                              CKB_HEADER_FIELD_TIMESTAMP);
```

## Time-Based Contract Patterns

### Since Field Integration

The `since` field provides time-based validation working with header dependencies:

#### Since Field Encoding (64-bit)
```
Bit 63: Absolute (0) or Relative (1)
Bit 62-61: Metric type
  00: Block number
  01: Epoch with fractional part  
  10: Timestamp (milliseconds)
  11: Reserved
Bit 60-0: Value
```

#### Basic Time-Lock Implementation

```c
#include "ckb_syscalls.h"

#define MIN_LOCK_EPOCHS 100

int main() {
    // Load lock epoch from script args
    uint64_t lock_epoch;
    uint64_t len = 8;
    int ret = ckb_load_script_args(&lock_epoch, &len, 0);
    if (ret != CKB_SUCCESS) return ret;
    
    // Load current epoch from header dependency
    uint64_t current_epoch;
    len = 8;
    ret = ckb_load_header_by_field(&current_epoch, &len, 0, 0,
                                  CKB_SOURCE_HEADER_DEP,
                                  CKB_HEADER_FIELD_EPOCH_NUMBER);
    if (ret != CKB_SUCCESS) return ret;
    
    // Enforce time lock
    if (current_epoch < lock_epoch + MIN_LOCK_EPOCHS) {
        return -1;  // Time lock not yet expired
    }
    
    return 0;  // Time lock expired, allow unlock
}
```

### Epoch-Based Validation Pattern

```c
int validate_epoch_requirement(uint64_t required_epoch) {
    uint64_t current_epoch;
    uint64_t len = 8;
    int ret = ckb_load_header_by_field(&current_epoch, &len, 0, 0,
                                      CKB_SOURCE_HEADER_DEP,
                                      CKB_HEADER_FIELD_EPOCH_NUMBER);
    if (ret != CKB_SUCCESS) return ret;
    
    return (current_epoch >= required_epoch) ? CKB_SUCCESS : ERROR_EPOCH_NOT_REACHED;
}
```

## Validation Rules and Constraints

### Historical Evolution

#### Pre-CKB2021: Immature Rule (Removed)
Originally enforced 4-epoch minimum age for header dependencies. This rule was removed in CKB2021.

#### Current Rules (Post-CKB2021)
- ✅ **No epoch-based restrictions**
- ✅ **Any block header can be included**
- ✅ **Recent headers are permitted** 
- ⚠️ **Headers must exist in canonical chain**

### Chain Reorganization Considerations

During blockchain reorganizations, header dependencies may become invalid:

```c
// Mitigation: Use multiple header dependencies for robustness
int robust_epoch_check(uint64_t required_epoch) {
    for (size_t i = 0; i < 3; i++) {  // Try multiple headers
        uint64_t epoch;
        uint64_t len = 8;
        int ret = ckb_load_header_by_field(&epoch, &len, 0, i,
                                          CKB_SOURCE_HEADER_DEP,
                                          CKB_HEADER_FIELD_EPOCH_NUMBER);
        
        if (ret == CKB_SUCCESS && epoch >= required_epoch) {
            return CKB_SUCCESS;
        }
    }
    
    return ERROR_EPOCH_REQUIREMENT_NOT_MET;
}
```

## Security Considerations

### Transaction Determinism

All CKB scripts must produce identical results regardless of execution time:

```c
// ❌ WRONG: Non-deterministic
int bad_time_check() {
    time_t current_time = time(NULL);  // Non-deterministic!
    return current_time > unlock_time ? 0 : -1;
}

// ✅ CORRECT: Deterministic via header dependencies
int good_time_check() {
    uint64_t block_timestamp;
    uint64_t len = 8;
    int ret = ckb_load_header_by_field(&block_timestamp, &len, 0, 0,
                                      CKB_SOURCE_HEADER_DEP,
                                      CKB_HEADER_FIELD_TIMESTAMP);
    return (ret == CKB_SUCCESS && block_timestamp > unlock_time) ? 0 : -1;
}
```

### Time Manipulation Resistance

Use epoch-based validation for critical time constraints:

```c
// Prefer epoch-based validation over timestamp-based
int secure_time_validation(uint64_t lock_epoch) {
    uint64_t current_epoch;
    uint64_t len = 8;
    int ret = ckb_load_header_by_field(&current_epoch, &len, 0, 0,
                                      CKB_SOURCE_HEADER_DEP,
                                      CKB_HEADER_FIELD_EPOCH_NUMBER);
    if (ret != CKB_SUCCESS) return ret;
    
    // Epoch progression is more predictable than timestamps
    return (current_epoch >= lock_epoch) ? CKB_SUCCESS : ERROR_TIME_LOCK_ACTIVE;
}
```

### Input Validation

Always validate header dependency access:

```c
int safe_header_access(size_t index) {
    uint64_t epoch;
    uint64_t len = 8;
    int ret = ckb_load_header_by_field(&epoch, &len, 0, index,
                                      CKB_SOURCE_HEADER_DEP,
                                      CKB_HEADER_FIELD_EPOCH_NUMBER);
    
    switch (ret) {
        case CKB_SUCCESS:
            return process_epoch(epoch);
        case CKB_INDEX_OUT_OF_BOUND:
            return ERROR_INVALID_HEADER_INDEX;
        case CKB_ITEM_MISSING:
            return ERROR_HEADER_NOT_FOUND;
        default:
            return ERROR_SYSCALL_FAILED;
    }
}
```

## Best Practices

### Header Selection Strategy

1. **Include recent, well-confirmed headers** to avoid reorganization issues
2. **Use multiple header dependencies** for redundancy
3. **Minimize header count** for performance (typically 1-3 headers)
4. **Cache frequently accessed header data** to avoid repeated syscalls

### Error Handling

```c
// Comprehensive error handling pattern
int load_header_with_fallback(uint64_t* epoch) {
    for (size_t i = 0; i < MAX_HEADER_DEPS; i++) {
        uint64_t len = 8;
        int ret = ckb_load_header_by_field(epoch, &len, 0, i,
                                          CKB_SOURCE_HEADER_DEP,
                                          CKB_HEADER_FIELD_EPOCH_NUMBER);
        if (ret == CKB_SUCCESS) {
            return CKB_SUCCESS;
        }
        
        if (ret == CKB_INDEX_OUT_OF_BOUND) {
            break;  // No more header dependencies
        }
    }
    
    return ERROR_NO_VALID_HEADERS;
}
```

### Time Tolerance Design

Build temporal tolerance into contract logic:

```c
// Allow reasonable time windows for validation
int validate_with_tolerance(uint64_t target_epoch, uint64_t tolerance) {
    uint64_t current_epoch;
    int ret = load_current_epoch(&current_epoch);
    if (ret != CKB_SUCCESS) return ret;
    
    // Accept if within tolerance window
    if (current_epoch >= target_epoch - tolerance && 
        current_epoch <= target_epoch + tolerance) {
        return CKB_SUCCESS;
    }
    
    return ERROR_OUTSIDE_TIME_WINDOW;
}
```

## Common Troubleshooting

### Issue: `CKB_INDEX_OUT_OF_BOUND` Error

**Solution**: Validate header dependency existence before access.

### Issue: Header Not Found During Validation  

**Solution**: Use multiple header dependencies and well-confirmed headers.

### Issue: Timestamp Validation Inconsistencies

**Solution**: Prefer epoch-based validation for critical constraints.

## Key Takeaways

1. **Scripts cannot access current block headers** - only explicitly provided historical headers
2. **All time-based validation operates on "stale" data** with inherent temporal gaps
3. **Transaction builders must predict needed headers** before script execution
4. **Determinism is prioritized over real-time accuracy** in CKB's design
5. **Epoch-based validation is more secure** than timestamp-based for critical logic
6. **Build temporal tolerance into contract design** to handle precision limitations

This fundamental understanding of header dependencies is essential for developing robust time-based smart contracts on CKB.