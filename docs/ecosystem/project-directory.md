# CKB Ecosystem Project Directory

## Description

Comprehensive directory of CKB ecosystem projects, tools, and resources organized by category and maintenance status. Provides curated listings of development frameworks, wallets, protocols, testing tools, and infrastructure services with GitHub links, documentation, and current status. Includes recommendations for new developers, legacy project information, and community resources. Essential reference for navigating the CKB development ecosystem and selecting appropriate tools.

## Core Development Tools (Recommended)

### Frontend/dApp Development
- **CCC (Recommended)**: Modern TypeScript/JavaScript SDK for CKB development
  - GitHub: https://github.com/ckb-devrel/ccc
  - Docs: https://docs.ckbccc.com/
  - Playground: https://live.ckbccc.com/
  - **Status**: Actively maintained, recommended for all new projects

- **Lumos (Legacy)**: Original CKB JS/TS framework
  - GitHub: https://github.com/ckb-js/lumos
  - **Status**: Still functional but not recommended for new projects
  - Use CCC for new development

### Smart Contract Development
- **ckb-script-templates (Current)**: Modern Rust script development
  - GitHub: https://github.com/cryptape/ckb-script-templates
  - **Status**: Replaces deprecated Capsule

- **ckb-std**: Official Rust library for CKB contracts
  - Docs: https://docs.rs/ckb-std/
  - GitHub: https://github.com/nervosnetwork/ckb-std
  - **Essential for**: All Rust smart contract development

- **Capsule (Deprecated)**: Legacy development framework
  - GitHub: https://github.com/nervosnetwork/capsule
  - **Status**: No longer maintained, use ckb-script-templates

### Testing and Debugging
- **ckb-testtool**: Testing library for CKB scripts
  - GitHub: https://github.com/nervosnetwork/ckb-testtool
  - **Status**: Actively maintained, essential for testing

- **ckb-standalone-debugger**: Debugging tools for contracts
  - GitHub: https://github.com/nervosnetwork/ckb-standalone-debugger

## Key CKB Protocols and Standards

### Digital Assets
- **Spore Protocol**: On-chain digital objects (DOBs)
  - Website: https://spore.pro/
  - Docs: https://docs.spore.pro/
  - GitHub: https://github.com/sporeprotocol
  - **Use Case**: NFTs, digital collectibles, on-chain content

- **xUDT**: Extensible User Defined Tokens
  - **Use Case**: Fungible tokens, custom assets
  - **Status**: Standard for token creation on CKB

### Layer 2 and Interoperability
- **RGB++**: Bitcoin layer 2 protocol using CKB
  - GitHub: https://github.com/ckb-cell
  - **Innovation**: Turing-complete contracts for Bitcoin
  - **Use Case**: Bitcoin DeFi, cross-chain assets

- **Fiber Network**: Payment and swap network
  - Website: https://www.fiber.world/
  - GitHub: https://github.com/nervosnetwork/fiber
  - **Use Case**: Scalable payments, Lightning-like functionality

## Production dApps and Examples

### DeFi Applications
- **UTXO Swap**: Decentralized exchange
  - SDK: https://github.com/UTXOSwap/utxoswap-sdk-js
  - **Use Case**: Asset trading, AMM

- **iCKB**: Liquid staking protocol
  - Website: https://ickb.org/
  - GitHub: https://github.com/ickb/contracts
  - **Use Case**: Liquid staking, DeFi primitives

- **Nerv DAO**: Universal DAO portal
  - Website: https://www.nervdao.com/
  - GitHub: https://github.com/ckb-devrel/nervdao
  - **Use Case**: DAO interaction, governance

### Infrastructure
- **OffCKB**: Local development toolkit
  - GitHub: https://github.com/ckb-devrel/offckb
  - **Use Case**: Full-stack dApp development, local testing

- **CKB Light Client**: Browser-compatible client
  - GitHub: https://github.com/nervosnetwork/ckb-light-client
  - **Use Case**: Lightweight blockchain interaction

## Wallet Integration

### Multi-Chain Wallets (CCC Compatible)
- **JoyID**: Web3 wallet with multi-chain support
- **MetaMask**: Ethereum wallet with CKB support via CCC
- **OKX Wallet**: Multi-chain wallet
- **UniSat**: Bitcoin wallet

### Native CKB Wallets
- **Neuron**: Official CKB desktop wallet
  - GitHub: https://github.com/nervosnetwork/neuron

## Development Resources

### Documentation
- **Nervos CKB Docs**: https://docs.nervos.org/
  - **Most comprehensive** CKB documentation
- **CKB Academy**: https://academy.ckb.dev/
  - Interactive learning platform

### Community
- **Nervos Talk**: https://talk.nervos.org/
  - Official community forum
- **CKB dApps Directory**: https://ckbdapps.com/
  - Curated dApp directory

## Code Examples and Templates

### CCC Examples
```typescript
// Modern CKB transfer with CCC
import { ccc } from "@ckb-ccc/ccc";

const client = new ccc.ClientPublicTestnet();
const signer = new ccc.SignerCkbPrivateKey(client, privateKey);

const tx = ccc.Transaction.from({
  outputs: [{ lock: toLock, capacity: ccc.fixedPointFrom(amount) }],
});

await tx.completeInputsByCapacity(signer);
await tx.completeFeeBy(signer);
const txHash = await signer.sendTransaction(tx);
```

### Spore Protocol Integration
```typescript
// Create a DOB (Digital Object) with Spore
import { spore } from "@ckb-ccc/spore";

const sporeData = {
  contentType: "image/jpeg",
  content: imageBytes,
  clusterId: clusterCell.cellOutput.type?.args,
};

const { tx } = await spore.createSpore({
  data: sporeData,
  toLock: recipientLock,
  fromInfos: [senderInfo],
});

const txHash = await signer.sendTransaction(tx);
```

## When to Use Which Tools

### For New Projects:
1. **Frontend**: Use CCC SDK
2. **Smart Contracts**: Use ckb-script-templates
3. **Testing**: Use ckb-testtool
4. **Local Development**: Use OffCKB

### For Existing Projects:
1. **Lumos → CCC**: Consider migration for better DX  
2. **Capsule → ckb-script-templates**: Migrate for continued support
3. **Legacy tools**: Evaluate modern alternatives

### For Specific Use Cases:
- **NFTs/Digital Assets**: Spore Protocol
- **Fungible Tokens**: xUDT standard
- **Bitcoin Integration**: RGB++ protocol
- **DeFi**: UTXO Swap, iCKB patterns
- **Payments**: Fiber Network

This ecosystem directory provides AI assistants with current, curated information about the CKB development landscape and recommended tools for different use cases.