# CKB Development Tool Recommendations (2024)

## Description

Choose the right CKB development tools for 2024 projects. Learn why CCC SDK is now preferred over Lumos for frontend development, understand the transition from deprecated Capsule to modern ckb-script-templates for smart contracts, and follow migration guides for updating existing projects. Get current recommendations for tool selection, development workflows, and modern CKB application building.

## Current Recommended Stack

### Frontend/Transaction Building: CCC (Recommended)
```typescript
// Install CCC - the newest and recommended SDK
npm install @ckb-ccc/ccc
```

**Why CCC is recommended:**
- **Official Standard**: All official Nervos examples now use CCC exclusively
- Modern, actively developed TypeScript/JavaScript SDK
- Intuitive transaction composition with auto-completion
- Built-in wallet integration across multiple chains
- Unified signing interface for seamless interoperability
- Better developer experience than Lumos
- Production-ready with comprehensive examples
- **Primary SDK**: Positioned as "highly recommended as the primary CKB development tool"

### Script Development: ckb-script-templates (Current)
```bash
# Modern Rust script development (replaces Capsule)
# Use via cargo generate (not installed as binary)
cargo generate --git https://github.com/cryptape/ckb-script-templates.git
```

**What happened to Capsule:**
- Capsule is **deprecated** and no longer maintained
- Functionality split into:
  - `ckb-script-templates` for project management
  - `ckb-testtool` for testing (still maintained)

### Testing: ckb-testtool (Active)
```toml
[dev-dependencies]
ckb-testtool = "0.12"
```

## Legacy Tool Status

### Lumos (Legacy - Not Recommended)
```typescript
// Lumos is no longer actively recommended for new projects
npm install @ckb-lumos/helpers
```

**Lumos status:**
- **Official Notice**: "No longer actively recommended for new projects" (per Nervos docs)
- Still functional but in maintenance mode
- **Migration Required**: Official docs recommend using CCC for "more robust development experience"
- Use CCC for all new projects
- **Note**: Quick Start guide erroneously mentions Lumos but all examples use CCC

### Migration Path
```typescript
// Old Lumos approach
import { TransactionSkeleton } from "@ckb-lumos/helpers";

// New CCC approach (recommended)
import { ccc } from "@ckb-ccc/ccc";
const tx = ccc.Transaction.from({
  outputs: [{ lock: toLock, capacity: amount }],
});
```

## Tool Selection Guidelines

### Use CCC When:
- **All new projects** (strongly recommended by Nervos)
- Building dApps or wallets
- Need modern wallet integration
- Want simplified transaction construction
- Require multi-chain support
- Building production applications
- Following official Nervos examples and tutorials

### Use Lumos When:
- **Legacy maintenance only**: Working with existing Lumos codebases
- Gradual migration from legacy code (migrate to CCC when possible)
- **Not recommended for**: Any new development or features

### Use ckb-script-templates When:
- Creating new smart contracts
- Need modern Rust tooling
- Building production scripts

### Avoid:
- **Capsule** (deprecated, use ckb-script-templates)
- **Old manual toolchains** (use modern Rust)

## Example: Modern CKB Transfer (CCC)
```typescript
import { ccc } from "@ckb-ccc/ccc";

const signer = new ccc.SignerCkbPrivateKey(client, privateKey);
const { script: toLock } = await ccc.Address.fromString(toAddress, client);

// Simple, intuitive transaction building
const tx = ccc.Transaction.from({
  outputs: [{ lock: toLock, capacity: ccc.fixedPointFrom(amount) }],
});

// Auto-complete inputs and fees
await tx.completeInputsByCapacity(signer);
await tx.completeFeeBy(signer);

// Send transaction
const txHash = await signer.sendTransaction(tx);
```

## Migration Benefits

### From Lumos to CCC:
- Simpler API surface
- Better TypeScript support
- Automatic capacity/fee management
- Modern wallet integrations
- Unified development experience

### From Capsule to ckb-script-templates:
- Modern Rust toolchain support
- Better project structure
- Maintained and updated regularly
- Aligned with current CKB development practices

## Getting Started Resources
- [CCC Documentation](https://docs.ckbccc.com/)
- [CCC Playground](https://live.ckbccc.com/)
- [ckb-script-templates](https://github.com/cryptape/ckb-script-templates)
- [Official CKB Docs](https://docs.nervos.org/)