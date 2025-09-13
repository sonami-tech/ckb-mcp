## Description

Spore protocol error debugging guide covering cluster validation failures, NFT minting errors, immortal constraint violations, content type mismatches, extension conflicts, and digital object backing (DOB) issues with solutions and code examples.

## Related Resources

- Protocol Spec: ckb-dev-context://protocols/spore-protocol
- Development Guide: ckb-dev-context://patterns/spore-development  
- SDK Examples: ckb-dev-context://api-reference/spore-sdk-examples
- Digital Objects: ckb-dev-context://protocols/spore-digital-objects

## Common Spore Errors

### ERROR: Cluster Validation Failed

**Error Code**: `0x60` (ClusterError: -96)

**Cause**: Spore cell references invalid or non-existent cluster

**Debug Steps**:
```typescript
// Validate cluster reference
async function validateCluster(sporeData: SporeData) {
  if (!sporeData.clusterId) return; // No cluster is valid
  
  // Find cluster cell
  const clusterCell = await findClusterCell(sporeData.clusterId);
  
  if (!clusterCell) {
    throw new Error(`Cluster ${sporeData.clusterId} not found`);
  }
  
  // Verify cluster allows this spore
  const clusterData = parseClusterData(clusterCell.data);
  
  if (clusterData.maxSupply && clusterData.currentCount >= clusterData.maxSupply) {
    throw new Error("Cluster max supply reached");
  }
}
```

### ERROR: Immortal Constraint Violated

**Error Code**: `0x61` (ImmutabilityError: -97)

**Cause**: Attempting to modify or destroy an immortal Spore

**Solution**:
```typescript
// Check immortal flag before operations
function checkImmortality(sporeCell: Cell): boolean {
  const sporeData = parseSporeData(sporeCell.data);
  
  // Immortal flag in content_type field
  const isImmortal = (sporeData.contentType & 0x80000000) !== 0;
  
  if (isImmortal) {
    // Cannot be in inputs (destroyed) or modified
    throw new Error("Immortal Spore cannot be modified or destroyed");
  }
  
  return isImmortal;
}

// Creating immortal Spore
const immortalSpore = {
  contentType: "image/png" | 0x80000000, // Set immortal bit
  content: imageData,
  clusterId: null
};
```

### ERROR: Invalid Content Type

**Error Code**: `0x62` (ContentTypeError: -98)

**Cause**: Content type string invalid or encoding error

**Valid Content Types**:
```typescript
const VALID_CONTENT_TYPES = [
  "image/png",
  "image/jpeg", 
  "image/gif",
  "image/webp",
  "text/plain",
  "text/markdown",
  "application/json",
  "audio/mpeg",
  "video/mp4",
  "model/gltf-binary"
];

function validateContentType(contentType: string): void {
  // Remove immortal flag if present
  const cleanType = contentType.replace(/\|0x80000000$/, "");
  
  if (!VALID_CONTENT_TYPES.includes(cleanType)) {
    throw new Error(`Invalid content type: ${cleanType}`);
  }
  
  // Check encoding
  if (contentType.length > 64) {
    throw new Error("Content type too long (max 64 bytes)");
  }
}
```

### ERROR: Content Size Exceeded

**Error Code**: `0x63` (SizeError: -99)

**Cause**: Spore content exceeds maximum cell size

**Size Management**:
```typescript
// Maximum practical content size
const MAX_CONTENT_SIZE = 500_000; // ~500KB recommended max

async function createSporeWithContent(content: Uint8Array) {
  if (content.length > MAX_CONTENT_SIZE) {
    // Use content reference instead
    const contentHash = blake256(content);
    const storageCell = await storeContentSeparately(content);
    
    return {
      contentType: "reference/blake256",
      content: contentHash,
      extension: {
        storageCell: storageCell.outPoint
      }
    };
  }
  
  // Direct storage for small content
  return {
    contentType: detectContentType(content),
    content: content
  };
}
```

### ERROR: Cluster Permission Denied

**Error Code**: `0x64` (PermissionError: -100)

**Cause**: Caller lacks permission to mint in cluster

**Permission Check**:
```typescript
interface ClusterData {
  name: string;
  description: string;
  mutantId?: string; // If set, only mutant owner can mint
  maxSupply?: number;
}

async function checkClusterPermission(
  cluster: Cell,
  signer: Address
): Promise<boolean> {
  const data = parseClusterData(cluster.data);
  
  if (data.mutantId) {
    // Only mutant NFT owner can mint
    const mutantOwner = await getMutantOwner(data.mutantId);
    
    if (mutantOwner !== signer) {
      throw new Error("Only mutant owner can mint in this cluster");
    }
  }
  
  return true;
}
```

### ERROR: DOB Transfer Constraint

**Error Code**: `0x65` (DOBError: -101)

**Cause**: Digital Object Backing rules violated

**DOB Validation**:
```typescript
// DOB (Digital Object Backing) enforces specific transfer rules
function validateDOBTransfer(spore: SporeCell, tx: Transaction) {
  const dobConfig = spore.data.extension?.dob;
  
  if (!dobConfig) return; // No DOB constraints
  
  // Check transfer conditions
  if (dobConfig.requiresProof) {
    const proof = tx.witnesses[0].outputType;
    if (!verifyDOBProof(proof, dobConfig)) {
      throw new Error("DOB proof validation failed");
    }
  }
  
  // Verify receiver capability
  if (dobConfig.restrictedTransfer) {
    const receiver = tx.outputs[0].lock;
    if (!isApprovedReceiver(receiver, dobConfig)) {
      throw new Error("Receiver not approved for DOB transfer");
    }
  }
}
```

### ERROR: Extension Conflict

**Error Code**: `0x66` (ExtensionError: -102)

**Cause**: Multiple incompatible extensions in Spore data

**Extension Management**:
```typescript
interface SporeExtensions {
  immortal?: boolean;
  dob?: DOBConfig;
  proxy?: ProxyConfig;
  social?: SocialConfig;
}

function validateExtensions(extensions: SporeExtensions): void {
  // Check for conflicts
  if (extensions.immortal && extensions.proxy) {
    throw new Error("Immortal Spores cannot have proxy extension");
  }
  
  if (extensions.dob && extensions.social) {
    // Some DOB configs conflict with social features
    if (extensions.dob.restrictedTransfer && extensions.social.allowSharing) {
      throw new Error("DOB restricted transfer conflicts with social sharing");
    }
  }
}
```

### ERROR: Invalid Spore ID

**Error Code**: `0x67` (IDError: -103)

**Cause**: Spore ID doesn't match expected format or derivation

**ID Calculation**:
```typescript
// Spore ID is first input's outpoint hash
function calculateSporeId(firstInput: Input): string {
  const outpoint = firstInput.previousOutput;
  const data = serializeOutpoint(outpoint);
  return "0x" + blake256(data);
}

// Validate Spore ID matches
function validateSporeId(sporeCell: Cell, expectedId: string) {
  const actualId = sporeCell.type.args;
  
  if (actualId !== expectedId) {
    throw new Error(`Spore ID mismatch: ${actualId} != ${expectedId}`);
  }
  
  // ID must be 32 bytes
  if (actualId.length !== 66) { // "0x" + 64 hex chars
    throw new Error("Invalid Spore ID length");
  }
}
```

## Debugging Utilities

### Spore Data Parser

```typescript
function parseSporeData(data: Uint8Array): SporeData {
  let offset = 0;
  
  // Read content type (variable length string)
  const contentTypeLen = data[offset];
  offset += 1;
  const contentType = new TextDecoder().decode(
    data.slice(offset, offset + contentTypeLen)
  );
  offset += contentTypeLen;
  
  // Read content length
  const contentLen = readUInt32LE(data.slice(offset, offset + 4));
  offset += 4;
  
  // Read content
  const content = data.slice(offset, offset + contentLen);
  offset += contentLen;
  
  // Read cluster ID (optional, 32 bytes)
  let clusterId = null;
  if (offset < data.length) {
    clusterId = "0x" + bytesToHex(data.slice(offset, offset + 32));
    offset += 32;
  }
  
  // Parse extensions if present
  const extensions = offset < data.length ? 
    parseExtensions(data.slice(offset)) : {};
  
  return {
    contentType,
    content,
    clusterId,
    extensions
  };
}
```

### Transaction Validator

```typescript
async function validateSporeTransaction(tx: Transaction) {
  // Check for Spore type script
  const sporeOutputs = tx.outputs.filter(output => 
    output.type?.codeHash === SPORE_TYPE_HASH
  );
  
  for (const output of sporeOutputs) {
    try {
      // Validate data structure
      const data = parseSporeData(output.data);
      validateContentType(data.contentType);
      
      // Check cluster if referenced
      if (data.clusterId) {
        await validateCluster(data);
      }
      
      // Validate extensions
      if (data.extensions) {
        validateExtensions(data.extensions);
      }
      
      console.log("✓ Valid Spore:", {
        id: output.type.args,
        contentType: data.contentType,
        size: data.content.length
      });
    } catch (e) {
      console.error("✗ Invalid Spore:", e.message);
    }
  }
}
```

## Common Mistakes

1. **Wrong content type format**: Must be valid MIME type string
2. **Missing cluster cell dep**: Include cluster as cell dependency
3. **Immortal bit in wrong position**: Use highest bit of content_type
4. **Invalid ID calculation**: Must use first input's outpoint
5. **Content too large**: Consider external storage for large files
6. **Extension encoding errors**: Follow Molecule schema strictly
7. **Missing type script**: Spore requires type script, not lock script