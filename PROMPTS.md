# CKB Development Prompts

This document contains helpful prompts for AI assistants working with the CKB MCP servers. These prompts leverage the comprehensive documentation and resources available through the MCP system to provide sophisticated CKB development assistance.

## Prerequisites

Ensure you have access to the CKB MCP servers:
- **ckb-docs-server** (port 8002): Provides access to `ckb-dev-context://` documentation resources.
- **ckb-rpc-server** (port 8001): Enables live blockchain data queries.
- **ckb-tools-server** (port 8003): Offers development and build tools.

## 1. Development Workflow Prompts

### New Project Setup
```
I want to create a new CKB smart contract project using modern Rust tooling. Please:

1. Access ckb-dev-context://getting-started/developer-resources-and-tooling to understand the current recommended stack.
2. Guide me through setting up a new Rust contract project with proper structure.
3. Show me how to configure the testing framework and build system.
4. Explain the modern development workflow emphasizing Rust over C.
5. Reference ckb-dev-context://patterns/development-tools-and-templates for best practices.

Focus on using ckb-script-templates and modern Rust patterns, avoiding deprecated tools like Capsule.
```

### Project Migration and Modernization
```
I have an existing CKB project that may be using older patterns or deprecated tools. Please:

1. Analyze my current project structure and dependencies.
2. Reference ckb-dev-context://getting-started/developer-resources-and-tooling to identify outdated components.
3. Provide a step-by-step migration plan to modern tooling.
4. Show how to update from Lumos to CCC for frontend components.
5. Update any C-based examples to modern Rust implementations.
6. Reference ckb-dev-context://patterns/script-development-patterns for current best practices.

Ensure the migration emphasizes Rust development and modern SDK usage.
```

### Development Environment Troubleshooting
```
I'm having issues with my CKB development environment. Please help me debug:

1. Check if I have the correct versions of required tools (Rust, CKB node, etc.).
2. Reference ckb-dev-context://troubleshooting/common-script-errors for known issues.
3. Verify my MCP server connections and configurations.
4. Test my CKB node connectivity using the ckb-rpc-server.
5. Validate my project structure against current best practices.
6. Provide specific solutions for any identified problems.

Include debugging commands and health checks for each component.
```

## 2. Smart Contract Development Prompts

### Basic Contract Creation
```
I want to create a [LOCK_SCRIPT/TYPE_SCRIPT/UDT_CONTRACT] in Rust. Please:

1. Reference ckb-dev-context://patterns/minimal-lock-script or ckb-dev-context://patterns/minimal-type-script as appropriate.
2. Show me the complete modern Rust implementation with proper error handling.
3. Explain each component: entry point, argument parsing, validation logic.
4. Include comprehensive testing patterns from ckb-dev-context://patterns/script-development-patterns.
5. Provide gas optimization tips and security considerations.
6. Show deployment and integration examples.

Ensure all examples use modern Rust with ckb-std, not C implementations.
```

### Advanced Contract Patterns
```
I need to implement [SPECIFIC_FUNCTIONALITY] in my CKB smart contract. Please:

1. Access relevant documentation from ckb-dev-context://patterns/ directory.
2. Show advanced Rust patterns for this functionality.
3. Reference ckb-dev-context://patterns/system-scripts-and-core-patterns for core concepts.
4. Include proper state validation and security checks.
5. Provide testing strategies for complex logic.
6. Show integration with other contracts or protocols.
7. Include gas optimization and performance considerations.

Focus on production-ready Rust implementations with comprehensive error handling.
```

### Security Audit and Review
```
Please perform a security review of my CKB smart contract. Analyze:

1. Input validation and argument parsing security.
2. Integer overflow protection and safe arithmetic.
3. Proper error handling and return codes.
4. Cell data validation and bounds checking.
5. Potential reentrancy or race condition issues.
6. Gas usage optimization and DoS prevention.
7. Reference ckb-dev-context://patterns/system-scripts-and-core-patterns for security patterns.

Provide specific recommendations for each identified issue with code examples.
```

## 3. Transaction Building Prompts

### Basic Transaction Construction
```
I need to build a CKB transaction for [TRANSFER/MULTI_SIG/DAO_OPERATION]. Please:

1. Reference ckb-dev-context://patterns/transaction-building-patterns for modern approaches.
2. Show the complete Rust implementation using ckb-sdk-rust.
3. Include proper cell collection and capacity management.
4. Demonstrate fee calculation and optimization.
5. Show error handling for common failure cases.
6. Provide both programmatic and testing examples.
7. Include witness generation and signing patterns.

Emphasize modern SDK usage and best practices for transaction construction.
```

### Advanced Transaction Patterns
```
I need to implement [BATCH_OPERATIONS/COMPLEX_MULTI_INPUT/CUSTOM_LOGIC] transactions. Please:

1. Access ckb-dev-context://patterns/transaction-building-patterns for advanced patterns.
2. Show how to handle complex input/output scenarios.
3. Demonstrate proper capacity balancing for multiple operations.
4. Include witness optimization for multiple signatures.
5. Show testing patterns for complex transactions.
6. Reference ckb-dev-context://api-reference/sdk-examples-and-patterns for SDK usage.
7. Include performance optimization and gas efficiency tips.

Focus on production-ready implementations with comprehensive error handling.
```

### Frontend Transaction Integration
```
I need to integrate CKB transactions into my web application. Please:

1. Reference ckb-dev-context://getting-started/developer-resources-and-tooling for CCC setup.
2. Show modern TypeScript/JavaScript patterns using CCC.
3. Demonstrate multi-wallet integration (MetaMask, Unisat, OKX, JoyID).
4. Include proper error handling and user feedback.
5. Show transaction simulation and fee estimation.
6. Reference ckb-dev-context://patterns/dao-development-patterns for DAO integration.
7. Include testing patterns for frontend integration.

Avoid deprecated Lumos patterns in favor of modern CCC implementations.
```

## 4. Protocol Integration Prompts

### Omnilock Integration
```
I want to integrate Omnilock for cross-chain wallet support. Please:

1. Reference ckb-dev-context://protocols/omnilock-protocol for protocol details.
2. Access ckb-dev-context://patterns/omnilock-development for implementation patterns.
3. Show how to support Ethereum, Bitcoin, and other wallet types.
4. Include ckb-dev-context://api-reference/omnilock-api-examples for specific usage.
5. Demonstrate transaction signing for different wallet types.
6. Include proper error handling for cross-chain scenarios.
7. Show testing strategies for multi-wallet support.

Focus on production-ready integration with modern SDK patterns.
```

### NFT and Digital Asset Development
```
I need to build NFT functionality using [SPORE/COTA] protocol. Please:

1. Reference ckb-dev-context://protocols/spore-protocol or ckb-dev-context://protocols/cota-protocol.
2. Access corresponding patterns documentation for implementation guidance.
3. Show complete mint, transfer, and burn operations.
4. Include metadata handling and content storage patterns.
5. Demonstrate batch operations for efficiency.
6. Reference API examples for SDK integration.
7. Include testing and deployment strategies.

Provide modern implementations with proper error handling and optimization.
```

### DAO Operations and Staking
```
I want to implement DAO operations [DEPOSIT/WITHDRAWAL/COMPENSATION]. Please:

1. Reference ckb-dev-context://patterns/dao-development-patterns for comprehensive guidance.
2. Show both Rust backend and TypeScript frontend implementations.
3. Include proper compensation calculation and validation.
4. Demonstrate multi-phase withdrawal patterns.
5. Show integration with iCKB for enhanced liquidity.
6. Include testing patterns for DAO lifecycle operations.
7. Reference ckb-dev-context://patterns/system-scripts-and-core-patterns for core DAO logic.

Focus on production-ready implementations with proper security validations.
```

## 5. Architecture and Design Prompts

### Cell Model Application Design
```
I'm designing a CKB application with [SPECIFIC_REQUIREMENTS]. Please help me:

1. Reference ckb-dev-context://concepts/cell-model for fundamental concepts.
2. Access ckb-dev-context://concepts/advanced-cell-concepts for complex patterns.
3. Design optimal cell structures for my use case.
4. Plan state transitions and validation logic.
5. Consider capacity requirements and optimization strategies.
6. Include security considerations and attack vector analysis.
7. Reference ckb-dev-context://patterns/ for relevant implementation patterns.

Provide a comprehensive architecture document with implementation roadmap.
```

### UTXO State Management Strategy
```
I need to design state management for my CKB application. Please:

1. Reference ckb-dev-context://concepts/transaction-structure for transaction patterns.
2. Show how to implement stateful operations using UTXO model.
3. Design efficient cell collection and management strategies.
4. Include state validation and consistency patterns.
5. Consider parallel processing and concurrency issues.
6. Reference ckb-dev-context://patterns/operation-detection for state tracking.
7. Include testing strategies for stateful applications.

Focus on scalable, efficient patterns for production applications.
```

### Performance Optimization Review
```
Please analyze my CKB application for performance optimization opportunities:

1. Review transaction construction efficiency.
2. Analyze cell collection and management patterns.
3. Check gas usage and cycle optimization.
4. Review memory usage and allocation patterns.
5. Reference ckb-dev-context://patterns/system-scripts-and-core-patterns for optimization techniques.
6. Include specific recommendations with code examples.
7. Provide benchmarking and testing strategies.

Focus on measurable improvements with quantified benefits.
```

## 6. Troubleshooting and Debugging Prompts

### Transaction Failure Diagnosis
```
My CKB transaction is failing with [ERROR_MESSAGE/ERROR_CODE]. Please help debug:

1. Use ckb-rpc-server to analyze the failing transaction.
2. Reference ckb-dev-context://troubleshooting/common-script-errors for known issues.
3. Check script validation and execution problems.
4. Analyze capacity and fee-related issues.
5. Verify witness and signature problems.
6. Reference ckb-dev-context://patterns/system-scripts-and-core-patterns for validation logic.
7. Provide specific fixes with code examples.

Include step-by-step debugging process and verification methods.
```

### Script Execution Problems
```
My CKB script is having execution issues [CYCLES/VALIDATION/LOGIC_ERRORS]. Please:

1. Analyze the script logic and execution flow.
2. Check for common pitfalls in ckb-dev-context://troubleshooting/common-script-errors.
3. Review gas usage and cycle optimization opportunities.
4. Verify input validation and error handling.
5. Reference ckb-dev-context://patterns/script-development-patterns for best practices.
6. Include testing strategies to prevent similar issues.
7. Provide optimized implementations.

Focus on both immediate fixes and long-term optimization.
```

### Network and RPC Issues
```
I'm experiencing connectivity or RPC problems with my CKB setup. Please help:

1. Use ckb-rpc-server to test blockchain connectivity.
2. Verify node synchronization and network status.
3. Check RPC endpoint configuration and authentication.
4. Test transaction broadcasting and confirmation.
5. Analyze indexer synchronization issues.
6. Include debugging commands and health checks.
7. Reference network-specific troubleshooting guides.

Provide systematic diagnosis and resolution steps.
```

## 7. Integration and Deployment Prompts

### Production Deployment Guide
```
I'm ready to deploy my CKB application to [MAINNET/TESTNET]. Please:

1. Reference ckb-dev-context://deployment/binary-deployment for deployment patterns.
2. Review security checklist and audit requirements.
3. Show testing strategies for production readiness.
4. Include monitoring and logging setup.
5. Demonstrate transaction fee optimization for production.
6. Reference ckb-dev-context://patterns/development-tools-and-templates for tooling.
7. Provide rollback and upgrade strategies.

Focus on production-grade deployment with proper safeguards.
```

### Testing and Quality Assurance
```
I need comprehensive testing for my CKB project. Please:

1. Reference testing patterns from ckb-dev-context://patterns/script-development-patterns.
2. Show unit testing strategies for contracts.
3. Include integration testing with blockchain interactions.
4. Demonstrate property-based testing approaches.
5. Include performance and load testing patterns.
6. Reference ckb-dev-context://patterns/system-scripts-and-core-patterns for test examples.
7. Show continuous integration setup.

Provide complete testing framework with automation strategies.
```

## Usage Tips

### How to Use These Prompts Effectively

1. **Be Specific**: Replace bracketed placeholders with your specific requirements.
2. **Reference Context**: The prompts leverage MCP documentation resources for comprehensive answers.
3. **Modern Focus**: All prompts emphasize current best practices and Rust development.
4. **Production Ready**: Examples and patterns focus on production-grade implementations.
5. **Comprehensive Coverage**: Each prompt addresses multiple aspects of the development process.

### Customizing Prompts

- Add your specific use case details to the bracketed sections.
- Combine multiple prompts for complex scenarios.
- Reference additional documentation resources as needed.
- Adapt the complexity level based on your experience.

### Getting the Best Results

- Ensure your MCP servers are running and accessible.
- Have your project context ready for analysis.
- Be prepared to iterate and refine based on initial responses.
- Use the comprehensive documentation resources for deeper understanding.

These prompts are designed to work with the rich CKB MCP ecosystem to provide sophisticated, actionable development assistance that follows modern best practices and emphasizes Rust development patterns.

## 8. MCP Development Prompts

### Adding New Repository Resources
```
Add the following submodules to the `resources` directory:

{{repo_urls}}

Then, for each submodule:

1. **Scan** all files and subdirectories, recursively. This includes source code, documentation, scripts, and examples. Skip compressed files such as `.zip`, `.tar.gz`, or `.7z`.

2. **Analyze** the contents to determine which parts are relevant to smart contract and application development on the CKB blockchain. This includes, but is not limited to:

   * Example smart contracts
   * System scripts
   * Tooling
   * Developer guides
   * Reference configurations or schemas

3. **Extract and integrate** useful information and examples into the MCP server. Do not simply copy everything. Instead, evaluate the material for relevance, and incorporate only the information that would assist in writing, testing, or understanding CKB smart contracts and apps.

4. If information already exists in the MCP documentation:

   * **Merge** new and relevant content.
   * Avoid duplication.
   * Retain the most current or complete version of any given information.

5. **Audit** all existing MCP documentation:

   * Update examples and recommendations to use **Rust** instead of **C** wherever possible.
   * Even if the original submodule uses C, MCP documentation should emphasize Rust as the preferred language for modern smart contract development.
   * Treat C as legacy in this context. Rust is the standard going forward due to its better safety guarantees, tooling, and alignment with current best practices.

6. **Error handling**: If any submodule fails to import or a file fails to parse, log or report the issue to screen. Continue processing remaining content.

Note: The `resources` directory should always be used as the destination path for all submodules.
```

### Updating Resource Submodules
```
Update all existing submodules in the `resources` directory and ensure the MCP documentation is up to date.

1. **Update Submodules**

   * For each submodule in the `resources` directory:

     * Fetch and merge the latest commit from the submodule's default remote tracking branch.
   * After all submodules have been updated:

     * Stage all updated submodule references in the parent repository.
     * Create a single commit that records the updated submodule pointers.
   * Pushing to the remote is not required.

2. **Rescan Updated Content**

   * Recursively scan all files and subdirectories within each updated submodule, skipping compressed files (e.g. `.zip`, `.tar.gz`, `.7z`).
   * Identify content relevant to smart contract and application development on the CKB blockchain. This includes:

     * Smart contract examples
     * System scripts
     * Developer guides
     * Build or deployment tools
     * Utility code or reference materials

3. **Integrate New Information into MCP**

   * Extract and integrate any newly relevant content into the MCP knowledge base.
   * Merge updates into existing content when appropriate to avoid duplication and preserve completeness.

4. **Language Preference**

   * Continue to prefer Rust over C in all examples and guidance.
   * Treat C as legacy. C-based material may be retained for reference but should not override or be prioritized over Rust.

5. **Error Handling**

   * Log or display any errors encountered during submodule updates or content scanning. Continue processing remaining items despite errors.
```
