## Description

Complete history of CKB network evolution, including hard fork dates, network name changes, and major protocol upgrades. Essential reference for understanding deployment contexts and version compatibility across different network eras. Covers the progression from Lina to Mirana to Meepo on mainnet, and Aggron to Pudge to Meepo on testnet.

## Network Evolution Overview

The Nervos CKB blockchain follows a continuous evolution model where network names change with major upgrades, but the underlying blockchain remains the same unbroken chain. Each hard fork is called an "edition" and is named after a Dota hero.

## Mainnet Evolution

### Lina Era (Genesis)
- **Period**: November 16, 2019 - May 10, 2021
- **Launch**: November 16, 2019 (Genesis Block)
- **Duration**: ~18 months
- **Key Features**: 
  - Initial mainnet launch
  - Basic UTXO model implementation
  - Foundation CKB-VM and scripting capabilities

### Mirana Era
- **Period**: May 10, 2021 - July 1, 2025
- **Hard Fork Date**: May 10, 2021 at 1:00 AM UTC
- **Activation**: Epoch 5,414
- **Duration**: ~4 years
- **CKB Version**: 0.103.0 (CKB2021 Edition)
- **Key Features**:
  - CKB-VM v2 activation
  - Cryptographic-friendly instruction set
  - Virtual machine version control
  - Extensible block headers
  - New cryptography standards
  - Improved network efficiency and performance
  - Enhanced smart contract composability

### Meepo Era (Current)
- **Period**: July 1, 2025 - Present
- **Hard Fork Date**: July 1, 2025 at 1:59 AM UTC
- **Activation**: Epoch 12,293
- **CKB Version**: 0.200.0+ (CKB Edition Meepo 2024)
- **Key Features**:
  - Spawn syscall for script interoperability
  - Block extension fields for community governance
  - CKB-VM optimizations
  - Reduced cycle consumption
  - Enhanced modularity capabilities

## Testnet Evolution

### Aggron Era
- **Period**: 2019 - October 24, 2021
- **Purpose**: Testing environment for Lina mainnet
- **Launch**: After CKB Mainnet Lina went live
- **End**: October 24, 2021 (CKB2021 hard fork)

### Pudge Era
- **Period**: October 24, 2021 - October 25, 2024
- **Hard Fork Date**: October 24, 2021
- **Activation**: Epoch 3,113
- **CKB Version**: 0.101.0 (CKB2021 Edition)
- **Purpose**: Testing environment for Mirana features
- **Duration**: ~3 years

### Meepo Era (Current)
- **Period**: October 25, 2024 - Present
- **Hard Fork Date**: October 25, 2024
- **CKB Version**: 0.119.0+
- **Purpose**: Testing environment for Meepo features
- **Features**: Same as planned mainnet Meepo implementation

## Hard Fork Summary

| Network | Era Name | Launch Date | Activation | Duration | Major Changes |
|---------|----------|-------------|------------|----------|---------------|
| Mainnet | Lina | Nov 16, 2019 | Genesis | ~18 months | Initial launch, basic UTXO |
| Mainnet | Mirana | May 10, 2021 | Epoch 5,414 | ~4 years | CKB-VM v2, performance upgrades |
| Mainnet | Meepo | Jul 1, 2025 | Epoch 12,293 | Current | Spawn syscall, modularity |
| Testnet | Aggron | 2019 | - | ~2 years | Lina testing environment |
| Testnet | Pudge | Oct 24, 2021 | Epoch 3,113 | ~3 years | Mirana testing environment |
| Testnet | Meepo | Oct 25, 2024 | - | Current | Testing Meepo features |

## Hard Fork Deployment Process

Nervos CKB follows a systematic three-phase deployment process for hard forks:

### Stage 1: Proposal and Development (~9 months)
- Detailed hard fork proposal creation and community discussions
- Finalized proposal published as RFC (Request for Comments)
- Development, initial testing, and local preview
- Release candidate (RC) version published
- Updates to SDKs, explorers, and wallets
- Example: CKB2021 RFC and development phase

### Stage 2: Testnet Activation
- Public preview network deployment for stability testing
- Hard fork activated on testnet first
- Extended testing period with real network conditions
- Node binary release and epoch number announcement
- Example: CKB2021 activated on Aggron→Pudge (October 24, 2021)

### Stage 3: Mainnet Activation (minimum 3-month preparation)
- Mainnet hard fork binary release with activation epoch
- At least three-month preparation period for network participants
- Hard fork activated on mainnet after successful testnet operation
- Requires majority node upgrade for successful deployment
- Example: CKB2021 activated on Lina→Mirana (May 10, 2021)

## Network Naming Convention and Hard Fork Policy

### Naming Convention
Starting with the second hard fork (Meepo), both mainnet and testnet adopt the same edition name:
- Format: "CKB Edition [Hero Name] (Year)"
- Example: "CKB Edition Meepo (2024)"
- Names inspired by Dota heroes for consistency
- Suffixes like "Mainnet" or "Testnet" only added when necessary for disambiguation

### Hard Fork Timing Policy
- **Minimum Interval**: At least one year (2,190 epochs) between hard forks
- **Purpose**: Prevent network instability from frequent upgrades
- **Benefits**: Allows substantial, forward-compatible upgrades with sufficient testing time
- **Developer Impact**: Provides adequate time for adaptation and testing

## Key Technical Changes by Era

### CKB2021 (Lina→Mirana, Aggron→Pudge)
- **CKB-VM Version 2**: Enhanced virtual machine capabilities
- **New Instruction Set**: Cryptographic-friendly operations
- **Performance Optimizations**: Faster script execution
- **Extensible Headers**: Support for future protocol extensions
- **Enhanced Security**: New cryptographic standards

### CKB Edition Meepo (2024)
- **Spawn Syscall (RFC 0050)**: Enables Unix-like process management with inter-process communication, pipes, and resource limits
- **CKB-VM V2 (RFC 0049)**: Enhanced security, efficiency, and scalability with Macro-Operation Fusion (MOPs) optimizations
- **Block Extension Fields**: Support for community governance features
- **Data Structure Updates (RFC 0048)**: Removed version field reservation rule to enable softfork signaling
- **Improved Composability**: Enhanced application development capabilities with isolated execution environments

## Deployment Considerations

When working with CKB deployments across different eras:

1. **Network Continuity**: All eras represent the same continuous blockchain
2. **Backward Compatibility**: Generally maintained for existing deployments
3. **Feature Availability**: New features only available post-activation
4. **Version Requirements**: Client software must support the target era
5. **Testing Strategy**: Always test on testnet before mainnet deployment

## Impact on Existing Deployments

- **Hash Compatibility**: Script hashes generally remain valid across eras
- **Feature Deprecation**: Rare, with advance notice and migration paths
- **Performance Changes**: Some operations may become more efficient
- **New Capabilities**: Access to enhanced features post-upgrade

## References

- [CKB Hard Fork Introduction](https://docs.nervos.org/docs/history-and-hard-forks/intro-to-hard-fork)
- [CKB2021 RFC Specification](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0037-ckb2021/0037-ckb2021.md)
- [CKB GitHub Releases](https://github.com/nervosnetwork/ckb/releases)
- [CKB Explorer Hard Fork Status](https://explorer.nervos.org/en/hardfork) - Real-time hard fork activation status and details
- [Meepo Hardfork Technical Details](https://blog.cryptape.com/meepo-hardfork-and-spawn-syscall-unlocking-modularity-on-ckb)
- [Nervos Network Medium Updates](https://medium.com/nervosnetwork)