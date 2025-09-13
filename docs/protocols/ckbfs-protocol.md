## Description

CKBFS (CKB File System) protocol for decentralized file storage on CKB blockchain. File publishing, versioning, content integrity verification, and API integration. Practical examples for building file storage applications, content management systems, and data archival solutions for developers needing blockchain-based file storage with witness-based storage patterns and Adler-32 checksum validation.

## Overview

CKBFS (CKB File Storage) is a witness-based content storage system for the Nervos CKB blockchain that enables **permanent file storage** on-chain. The protocol provides a simple, flexible mechanism for storing files larger than block size limitations (~500KB) while maintaining immutability, transparency, and data integrity through cryptographic checksums.

**Key Deployments:**
- **Mainnet CKBFS Contract**: `0x31e6376287d223b8c0410d562fb422f04d1d617b2947596a14c3d2efb7218d3a`
- **Mainnet Adler32 Contract**: `0x2138683f76944437c0c643664120d620bdb5858dd6c9d1d156805e279c2c536f`

## Key Features

### Permanent File Storage
- Files stored using CKBFS are permanent and immutable
- Cells containing file data cannot be destroyed
- Provides long-term data persistence guarantees

### Witnesses-Based Storage
- File contents are stored in transaction witnesses
- Enables storage of files larger than individual block limits
- Reduces on-chain storage costs compared to cell data storage

### Checksum Validation
- Uses Adler-32 algorithm for file integrity verification
- Automatic checksum validation during file operations
- Supports checksum recovery across multiple transactions

### File Branching and Forking
- Supports creating file variants and versions
- Enables collaborative content development
- Maintains transparent history of file modifications

## Technical Architecture

### Core Design Principles

1. **Simplicity**: Minimal complexity in protocol design
2. **Permanence**: Files cannot be deleted once published
3. **Flexibility**: User-defined lock scripts for access control
4. **Transparency**: Complete file tracking through backlinks

### Cell Structure and Data Format

CKBFS uses Molecule serialization for all data structures. The actual implementation uses the following schema:

```rust
// CKBFS cell structure (from ckbfs-types)
struct CKBFSCell {
    capacity: u64,           // Required capacity for file indexing
    lock: Script,            // User-defined access control
    type: Script,            // CKBFS type script (with hash type ID args)
    data: CKBFSData,         // Molecule-encoded file metadata
}

// Actual CKBFSData structure (Molecule schema)
table CKBFSData {
    indexes: Indexes,        // Vector of u32 file location indexes
    checksum: Uint32,        // Adler-32 checksum for integrity
    content_type: Bytes,     // MIME type (e.g., "text/plain")
    filename: Bytes,         // Original filename as UTF-8 bytes  
    backlinks: BackLinkVec,  // References to related/previous files
}

// BackLink structure for file relationships
table BackLink {
    indexes: Indexes,        // Referenced file indexes
    checksum: Uint32,        // Referenced file checksum
    tx_hash: Byte32,         // Transaction hash containing the reference
}

// Witness format: "CKBFS" + version_byte + file_content
// Version byte: 0x0 for current protocol version
const CKBFS_WITNESSES_OFFSET: usize = 6; // Skip "CKBFS" + version byte
```

### Storage Mechanism

#### File Publication Pattern (TypeScript SDK Example)

Based on the ckbfs-ts implementation, here's how files are published:

```typescript
// 1. File chunking for large files (30KB chunks)
const chunkSize = 30 * 1024;
const fileChunks: Buffer[] = [];
for (let i = 0; i < fileBuffer.length; i += chunkSize) {
    fileChunks.push(Buffer.from(fileBuffer.slice(i, i + chunkSize)));
}

// 2. Calculate Adler-32 checksum for integrity
const checksum = adler32(fileBuffer);

// 3. Build Molecule data structure
const ckbfsData = CKBFSData.pack({
    index: [1], // File location index
    checksum: checksum,
    contentType: "text/plain",
    filename: "example.txt",
    backLinks: [], // No backlinks for new files
});

// 4. Create witnesses with CKBFS format
const ckbfsWitnesses = fileChunks.map((chunk) => {
    return Buffer.concat([
        textEncoder.encode("CKBFS"), // Protocol identifier
        new Uint8Array([0x0]),       // Version byte
        chunk                        // File content chunk
    ]).toString("hex");
});

// 5. Build transaction with type script using hashTypeId pattern
const typeArgs = hashTypeId(firstInput, outputIndex);
const ckbfsTypeScript = {
    codeHash: CKBFS_CODE_HASH,
    hashType: "data1",
    args: typeArgs,
};
```

#### Rust Contract Implementation

The CKBFS contract enforces protocol rules:

```rust
// Contract entry point (from ckbfs-api analysis)
pub fn main() -> Result<(), Error> {
    let script = load_script()?;
    let args: Bytes = script.args().unpack();
    
    // Determine operation type
    let (input_count, output_count) = count_cells()?;
    
    match (input_count, output_count) {
        (0, 1) => validate_file_creation()?,      // New file
        (1, 1) => validate_file_update()?,       // Append operation  
        (1, 0) => return Err(Error::DeletionForbidden), // Prevent deletion
        _ => return Err(Error::InvalidTransaction),
    }
    
    Ok(())
}

// Deletion prevention (core protocol feature)
fn validate_file_creation() -> Result<(), Error> {
    // Validate type ID generation
    let type_id = load_script_args()?;
    if !validate_type_id(&type_id, 0) {
        return Err(Error::InvalidTypeId);
    }
    
    // Validate witness format and checksum
    validate_witnesses_and_checksum()?;
    
    Ok(())
}

// Cross-contract checksum validation
fn validate_witnesses_and_checksum() -> Result<(), Error> {
    let ckbfs_data = load_ckbfs_data()?;
    let file_content = load_witnesses_for_ckbfs(0, Source::Input)?;
    
    // Delegate checksum validation to Adler32 contract
    let code_hash = load_cell_data(1, Source::CellDep)?; // Adler32 contract
    let exec_args = [&ckbfs_data.checksum.to_le_bytes(), &file_content].concat();
    
    match ckb_std::high_level::exec_cell(code_hash, ScriptHashType::Data1, &exec_args) {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::ChecksumMismatch),
    }
}
```

#### File Appending
```rust
// Appending data to existing file
fn append_to_file(
    existing_cell: CellInput,
    additional_data: &[u8],
) -> Result<Transaction, CKBFSError> {
    // 1. Verify existing file integrity
    verify_file_checksum(&existing_cell)?;
    
    // 2. Calculate new checksum for combined data
    let new_checksum = calculate_combined_checksum(
        &existing_cell, 
        additional_data
    );
    
    // 3. Create new version with updated metadata
    let updated_metadata = update_file_metadata(
        &existing_cell.metadata,
        additional_data.len(),
        new_checksum,
    );
    
    // 4. Build append transaction
    build_append_transaction(existing_cell, additional_data, updated_metadata)
}
```

### File Transfer and Access Control

```rust
// Transferring file ownership
fn transfer_file(
    current_cell: CellInput,
    new_lock_script: Script,
    signature: Signature,
) -> Result<Transaction, CKBFSError> {
    // 1. Verify current owner signature
    verify_signature(&current_cell.lock, &signature)?;
    
    // 2. Create new cell with updated lock script
    let new_cell = CellOutput {
        capacity: current_cell.capacity,
        lock: new_lock_script,
        type: current_cell.type.clone(),
        data: current_cell.data.clone(), // Metadata unchanged
    };
    
    // 3. Build transfer transaction
    build_transfer_transaction(current_cell, new_cell)
}
```

## TypeScript SDK Integration

### CKBFS API Library

The CKBFS TypeScript SDK provides a comprehensive client library for file operations:

```typescript
import { CKBFS, NetworkType, ProtocolVersion } from "ckbfs-api";

// Initialize CKBFS client
const ckbfs = new CKBFS(
  'your-private-key',
  NetworkType.Testnet,
  {
    version: ProtocolVersion.V2,
    chunkSize: 30 * 1024,  // 30KB chunks
    useTypeID: false
  }
);
```

### Publishing Files with TypeScript

```typescript
// Publish file from filesystem
const txHash = await ckbfs.publishFile('./document.pdf', {
  contentType: 'application/pdf',
  filename: 'document.pdf'
});

// Publish content directly
const content = new TextEncoder().encode("Hello, CKBFS!");
const txHash = await ckbfs.publishContent(content, {
  contentType: 'text/plain',
  filename: 'greeting.txt',
  capacity: 300n * 100000000n  // 300 CKB
});
```

### File Appending Operations

```typescript
// Append content to existing file
const ckbfsCell = await findCKBFSCell(originalTxHash);
const appendTxHash = await ckbfs.appendContent(
  "Additional content",
  ckbfsCell
);

// Append binary data
const additionalData = new Uint8Array([1, 2, 3, 4]);
const appendTxHash = await ckbfs.appendContent(
  additionalData,
  ckbfsCell
);
```

### Multiple Retrieval Methods

```typescript
// Method 1: Retrieve from blockchain by OutPoint
const outPoint = { txHash: "0x...", index: 0 };
const content = await getFileContentFromChain(client, outPoint, ckbfsData);

// Method 2: Direct witness decoding (faster)
const decoded = decodeWitnessContent(witnessHex);

// Method 3: Generic identifier interface
const fileData = await getFileContentFromChainByIdentifier(
  client,
  'ckbfs://type-id-or-outpoint',
  { network: 'testnet', version: ProtocolVersion.V2 }
);
```

### Identifier Formats

CKBFS supports multiple identifier formats for flexible file access:

```typescript
// TypeID hex format
const typeIdHex = "0xbce89252cece632ef819943bed9cd0e2576f8ce26f9f02075b621b1c9a28056a";

// CKBFS TypeID URI
const typeIdUri = "ckbfs://bce89252cece632ef819943bed9cd0e2576f8ce26f9f02075b621b1c9a28056a";

// CKBFS OutPoint URI
const outPointUri = "ckbfs://431c9d668c1815d26eb4f7ac6256eb350ab351474daea8d588400146ab228780i0";

// Use any format
const content = await getFileContentFromChainByIdentifier(client, typeIdUri);
```

### Version Control with TypeScript

```typescript
class FileVersionManager {
  async getVersionHistory(identifier: string): Promise<Version[]> {
    const versions = [];
    let current = identifier;
    
    while (current) {
      const fileData = await getFileContentFromChainByIdentifier(
        client, 
        current
      );
      
      versions.push({
        identifier: current,
        content: fileData.content,
        timestamp: fileData.timestamp,
        checksum: fileData.checksum
      });
      
      // Follow backlink to previous version
      current = fileData.backlinks?.[0];
    }
    
    return versions.reverse(); // Oldest first
  }
}
```

### Application Integration Patterns

```typescript
// Content Management System
class DecentralizedCMS {
  private ckbfs: CKBFS;
  
  constructor(privateKey: string) {
    this.ckbfs = new CKBFS(privateKey, NetworkType.Mainnet);
  }
  
  async publishArticle(article: Article): Promise<string> {
    const content = JSON.stringify(article);
    const encodedContent = new TextEncoder().encode(content);
    
    return this.ckbfs.publishContent(encodedContent, {
      contentType: 'application/json',
      filename: `article-${article.id}.json`
    });
  }
}

// Data Archival System
class DataArchive {
  async archiveData(data: any, metadata: ArchiveMetadata): Promise<ArchiveRecord> {
    const compressed = await compress(JSON.stringify(data));
    
    const txHash = await ckbfs.publishContent(compressed, {
      contentType: 'application/gzip',
      filename: metadata.filename
    });
    
    return {
      id: txHash,
      originalSize: JSON.stringify(data).length,
      compressedSize: compressed.length,
      checksum: calculateAdler32(compressed),
      timestamp: Date.now(),
      metadata
    };
  }
}
```

## Use Cases

### 1. Decentralized Content Publishing
```rust
// Publishing blog posts or articles
let article_data = include_bytes!("article.md");
let tx = publish_file(
    article_data,
    "my-article.md".to_string(),
    author_lock_script,
)?;
```

### 2. Software Distribution
```rust
// Distributing software packages
let package_data = include_bytes!("package.tar.gz");
let tx = publish_file(
    package_data,
    "my-package-v1.0.0.tar.gz".to_string(),
    maintainer_lock_script,
)?;
```

### 3. Legal Document Storage
```rust
// Storing contracts and legal documents
let contract_data = include_bytes!("contract.pdf");
let tx = publish_file(
    contract_data,
    "legal-contract-2024.pdf".to_string(),
    multi_sig_lock_script, // Requires multiple signatures
)?;
```

### 4. Academic Research
```rust
// Publishing research papers with version control
let paper_data = include_bytes!("research-paper.pdf");
let tx = publish_file(
    paper_data,
    "blockchain-research-v2.pdf".to_string(),
    research_group_lock_script,
)?;
```

## Protocol Operations

### File Discovery and Reading

```rust
// Discovering files by owner
fn find_files_by_owner(lock_script_hash: H256) -> Vec<CKBFSFile> {
    // Query indexer for CKBFS cells with matching lock script
    let cells = indexer.get_cells_by_lock_script_hash(lock_script_hash);
    
    cells.into_iter()
        .filter(|cell| is_ckbfs_cell(cell))
        .map(|cell| parse_ckbfs_file(cell))
        .collect()
}

// Reading file contents
fn read_file_contents(file_cell: CKBFSCell) -> Result<Vec<u8>, CKBFSError> {
    // 1. Get all transactions containing file data
    let file_transactions = get_file_transactions(&file_cell)?;
    
    // 2. Extract file chunks from witnesses
    let mut file_data = Vec::new();
    for tx in file_transactions {
        let chunk = extract_file_chunk_from_witnesses(&tx)?;
        file_data.extend(chunk);
    }
    
    // 3. Verify file integrity
    let calculated_checksum = adler32_checksum(&file_data);
    if calculated_checksum != file_cell.metadata.checksum {
        return Err(CKBFSError::ChecksumMismatch);
    }
    
    Ok(file_data)
}
```

### File Versioning and History

```rust
// Getting file version history
fn get_file_history(file_cell: CKBFSCell) -> Vec<CKBFSVersion> {
    let mut versions = Vec::new();
    let mut current_cell = file_cell;
    
    // Traverse backlinks to find all versions
    while let Some(previous_cell) = get_previous_version(&current_cell) {
        versions.push(CKBFSVersion {
            version: current_cell.metadata.version,
            size: current_cell.metadata.file_size,
            checksum: current_cell.metadata.checksum,
            created_at: current_cell.metadata.created_at,
            cell_reference: current_cell.out_point,
        });
        current_cell = previous_cell;
    }
    
    versions.reverse(); // Return chronological order
    versions
}
```

## Security Considerations

### Access Control
- **Lock Script Flexibility**: Use appropriate lock scripts for intended access patterns
- **Multi-signature Support**: Implement multi-sig for sensitive files
- **Time Locks**: Add time-based restrictions if needed

### Data Integrity
- **Checksum Verification**: Always verify Adler-32 checksums when reading files
- **Transaction Validation**: Validate all file operations through type script
- **History Verification**: Check file version history for tampering

### Storage Economics
- **Capacity Planning**: Account for witness storage costs in transaction fees
- **File Size Optimization**: Consider compression for large files
- **Version Management**: Balance versioning needs with storage costs

## Implementation Guidelines

### Contract Development
```rust
// CKBFS type script implementation
fn ckbfs_type_script_main() -> Result<(), Error> {
    let operation = determine_operation()?;
    
    match operation {
        CKBFSOperation::Publish => validate_file_publication()?,
        CKBFSOperation::Append => validate_file_append()?,
        CKBFSOperation::Transfer => validate_file_transfer()?,
        CKBFSOperation::Fork => validate_file_fork()?,
    }
    
    Ok(())
}
```

### Client Integration
```rust
// CKBFS client library usage
use ckbfs_sdk::CKBFSClient;

let client = CKBFSClient::new(ckb_rpc_url);

// Publish file
let file_hash = client.publish_file(
    file_data,
    "document.pdf",
    owner_lock_script,
).await?;

// Read file
let file_contents = client.read_file(file_hash).await?;

// List user files
let user_files = client.list_files_by_owner(user_lock_hash).await?;
```

## Current Status

### Deployment
- **Status**: Deployed and tested on CKB testnet
- **Implementation**: Open-source contract available on GitHub
- **Documentation**: Comprehensive protocol specification available

### Ecosystem Integration
- **SDK Support**: Client libraries available for Rust and JavaScript
- **Tooling**: Command-line tools for file operations
- **Indexing**: Specialized indexers for CKBFS file discovery

The CKBFS protocol provides a robust foundation for decentralized file storage on CKB, enabling new applications in content publishing, software distribution, and data permanence.