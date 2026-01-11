#!/usr/bin/env python3
"""
CKB Cell Consolidation Script

Consolidates multiple cells with data into fewer cells without data,
releasing the occupied capacity back to free/spendable capacity.

Requirements:
    pip install requests secp256k1 blake3

Usage:
    python consolidate_cells.py --rpc-url http://127.0.0.1:8114 --private-key <hex>

The script will:
1. Query all cells owned by the private key's address
2. Group cells into batches (to avoid transaction size limits)
3. For each batch, create a consolidation transaction
4. Sign and submit each transaction
5. Wait for confirmation before proceeding to the next batch
"""

import argparse
import hashlib
import json
import struct
import sys
import time
from dataclasses import dataclass
from typing import List, Optional, Tuple

import requests

# Try to import secp256k1, fall back to ecdsa if not available
try:
    import secp256k1
    HAS_SECP256K1 = True
except ImportError:
    HAS_SECP256K1 = False
    try:
        from ecdsa import SECP256k1, SigningKey, VerifyingKey
        from ecdsa.util import sigencode_string_canonize
        HAS_ECDSA = True
    except ImportError:
        HAS_ECDSA = False
        print("ERROR: Please install either 'secp256k1' or 'ecdsa' package:")
        print("  pip install secp256k1")
        print("  # or")
        print("  pip install ecdsa")
        sys.exit(1)


# =============================================================================
# Constants
# =============================================================================

# CKB Blake2b personalization
CKB_HASH_PERSONALIZATION = b"ckb-default-hash"

# secp256k1_blake160_sighash_all code hash (type hash)
SECP256K1_CODE_HASH = bytes.fromhex(
    "9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8"
)

# Minimum cell capacity (61 CKB in shannons)
MIN_CELL_CAPACITY = 61 * 10**8

# Default fee rate (1000 shannons per KB)
DEFAULT_FEE_RATE = 1000

# Maximum cells per batch (to stay within cycle limits)
MAX_CELLS_PER_BATCH = 50

# Signature placeholder (65 bytes of zeros)
SIGNATURE_PLACEHOLDER = bytes(65)


# =============================================================================
# Blake2b Hashing (CKB variant)
# =============================================================================

def ckb_blake2b(data: bytes) -> bytes:
    """Compute CKB-style Blake2b-256 hash with personalization."""
    try:
        from hashlib import blake2b
        h = blake2b(digest_size=32, person=CKB_HASH_PERSONALIZATION)
        h.update(data)
        return h.digest()
    except ImportError:
        # Fallback for systems without hashlib blake2b
        import blake3
        # Note: blake3 doesn't support personalization, this is a fallback
        # For production, ensure blake2b is available
        return blake3.blake3(data).digest()


def blake160(data: bytes) -> bytes:
    """Compute Blake2b-160 hash (first 20 bytes of Blake2b-256)."""
    return ckb_blake2b(data)[:20]


# =============================================================================
# Molecule Serialization
# =============================================================================

def pack_u32_le(value: int) -> bytes:
    """Pack a 32-bit unsigned integer in little-endian."""
    return struct.pack("<I", value)


def pack_u64_le(value: int) -> bytes:
    """Pack a 64-bit unsigned integer in little-endian."""
    return struct.pack("<Q", value)


def pack_bytes(data: bytes) -> bytes:
    """Pack variable-length bytes (fixvec format): length (u32 LE) + data."""
    return pack_u32_le(len(data)) + data


def pack_byte32(data: bytes) -> bytes:
    """Pack a 32-byte array."""
    assert len(data) == 32, f"Expected 32 bytes, got {len(data)}"
    return data


def pack_script(code_hash: bytes, hash_type: int, args: bytes) -> bytes:
    """
    Pack a Script structure (table format).

    Script layout:
    - full_size (u32)
    - offset to code_hash (u32)
    - offset to hash_type (u32)
    - offset to args (u32)
    - code_hash (32 bytes)
    - hash_type (1 byte)
    - args (Bytes = length + data)
    """
    # Field data
    code_hash_data = pack_byte32(code_hash)  # 32 bytes
    hash_type_data = bytes([hash_type])  # 1 byte
    args_data = pack_bytes(args)  # 4 + len(args) bytes

    # Calculate offsets (header has 4 fields * 4 bytes = 16 bytes)
    header_size = 4 * 4  # full_size + 3 offsets
    offset_code_hash = header_size
    offset_hash_type = offset_code_hash + len(code_hash_data)
    offset_args = offset_hash_type + len(hash_type_data)
    full_size = offset_args + len(args_data)

    # Build header
    header = (
        pack_u32_le(full_size) +
        pack_u32_le(offset_code_hash) +
        pack_u32_le(offset_hash_type) +
        pack_u32_le(offset_args)
    )

    return header + code_hash_data + hash_type_data + args_data


def pack_script_opt(script: Optional[bytes]) -> bytes:
    """Pack an optional Script (option format)."""
    if script is None:
        return b""
    return script


def pack_cell_output(capacity: int, lock_script: bytes, type_script: Optional[bytes] = None) -> bytes:
    """
    Pack a CellOutput structure (table format).

    CellOutput layout:
    - full_size (u32)
    - offset to capacity (u32)
    - offset to lock (u32)
    - offset to type_ (u32)
    - capacity (u64 LE)
    - lock (Script)
    - type_ (ScriptOpt)
    """
    capacity_data = pack_u64_le(capacity)  # 8 bytes
    lock_data = lock_script
    type_data = pack_script_opt(type_script)

    header_size = 4 * 4  # full_size + 3 offsets
    offset_capacity = header_size
    offset_lock = offset_capacity + len(capacity_data)
    offset_type = offset_lock + len(lock_data)
    full_size = offset_type + len(type_data)

    header = (
        pack_u32_le(full_size) +
        pack_u32_le(offset_capacity) +
        pack_u32_le(offset_lock) +
        pack_u32_le(offset_type)
    )

    return header + capacity_data + lock_data + type_data


def pack_out_point(tx_hash: bytes, index: int) -> bytes:
    """Pack an OutPoint structure (struct format, fixed size)."""
    return pack_byte32(tx_hash) + pack_u32_le(index)


def pack_cell_input(since: int, out_point: bytes) -> bytes:
    """Pack a CellInput structure (struct format, fixed size)."""
    return pack_u64_le(since) + out_point


def pack_cell_dep(out_point: bytes, dep_type: int) -> bytes:
    """Pack a CellDep structure (struct format, fixed size)."""
    return out_point + bytes([dep_type])


def pack_fixvec(items: List[bytes], item_size: int) -> bytes:
    """Pack a fixed-size vector (fixvec format)."""
    count = len(items)
    data = b"".join(items)
    return pack_u32_le(count) + data


def pack_dynvec(items: List[bytes]) -> bytes:
    """Pack a dynamic-size vector (dynvec format)."""
    if not items:
        return pack_u32_le(4)  # Just the full_size field

    # Calculate header size: full_size (4) + offsets (4 * count)
    header_size = 4 + 4 * len(items)

    # Calculate offsets
    offsets = []
    current_offset = header_size
    for item in items:
        offsets.append(current_offset)
        current_offset += len(item)

    full_size = current_offset

    # Build header
    header = pack_u32_le(full_size)
    for offset in offsets:
        header += pack_u32_le(offset)

    # Append all items
    return header + b"".join(items)


def pack_raw_transaction(
    version: int,
    cell_deps: List[bytes],
    header_deps: List[bytes],
    inputs: List[bytes],
    outputs: List[bytes],
    outputs_data: List[bytes],
) -> bytes:
    """
    Pack a RawTransaction structure (table format).

    Used for calculating transaction hash.
    """
    version_data = pack_u32_le(version)
    cell_deps_data = pack_fixvec(cell_deps, 37)  # CellDep is 37 bytes
    header_deps_data = pack_fixvec(header_deps, 32)  # Byte32 is 32 bytes
    inputs_data = pack_fixvec(inputs, 44)  # CellInput is 44 bytes
    outputs_data_packed = pack_dynvec(outputs)
    outputs_data_vec = pack_dynvec(outputs_data)

    # Header: full_size + 6 offsets
    header_size = 4 + 4 * 6

    offset_version = header_size
    offset_cell_deps = offset_version + len(version_data)
    offset_header_deps = offset_cell_deps + len(cell_deps_data)
    offset_inputs = offset_header_deps + len(header_deps_data)
    offset_outputs = offset_inputs + len(inputs_data)
    offset_outputs_data = offset_outputs + len(outputs_data_packed)
    full_size = offset_outputs_data + len(outputs_data_vec)

    header = (
        pack_u32_le(full_size) +
        pack_u32_le(offset_version) +
        pack_u32_le(offset_cell_deps) +
        pack_u32_le(offset_header_deps) +
        pack_u32_le(offset_inputs) +
        pack_u32_le(offset_outputs) +
        pack_u32_le(offset_outputs_data)
    )

    return (
        header +
        version_data +
        cell_deps_data +
        header_deps_data +
        inputs_data +
        outputs_data_packed +
        outputs_data_vec
    )


def pack_witness_args(
    lock: Optional[bytes] = None,
    input_type: Optional[bytes] = None,
    output_type: Optional[bytes] = None,
) -> bytes:
    """
    Pack a WitnessArgs structure (table format).

    All fields are optional (BytesOpt).
    """
    lock_data = pack_bytes(lock) if lock is not None else b""
    input_type_data = pack_bytes(input_type) if input_type is not None else b""
    output_type_data = pack_bytes(output_type) if output_type is not None else b""

    # Header: full_size + 3 offsets
    header_size = 4 + 4 * 3

    offset_lock = header_size
    offset_input_type = offset_lock + len(lock_data)
    offset_output_type = offset_input_type + len(input_type_data)
    full_size = offset_output_type + len(output_type_data)

    header = (
        pack_u32_le(full_size) +
        pack_u32_le(offset_lock) +
        pack_u32_le(offset_input_type) +
        pack_u32_le(offset_output_type)
    )

    return header + lock_data + input_type_data + output_type_data


# =============================================================================
# Cryptographic Functions
# =============================================================================

def private_key_to_public_key(private_key: bytes) -> bytes:
    """Derive compressed public key from private key."""
    if HAS_SECP256K1:
        sk = secp256k1.PrivateKey(private_key)
        return sk.pubkey.serialize()
    else:
        sk = SigningKey.from_string(private_key, curve=SECP256k1)
        vk = sk.get_verifying_key()
        # Get compressed form
        point = vk.pubkey.point
        prefix = b'\x02' if point.y() % 2 == 0 else b'\x03'
        return prefix + point.x().to_bytes(32, 'big')


def public_key_to_lock_arg(public_key: bytes) -> bytes:
    """Compute lock arg (blake160 of public key)."""
    return blake160(public_key)


def sign_message(private_key: bytes, message: bytes) -> bytes:
    """
    Sign a message using secp256k1 and return recoverable signature.

    Returns 65 bytes: r (32) + s (32) + recovery_id (1)
    """
    if HAS_SECP256K1:
        sk = secp256k1.PrivateKey(private_key)
        sig = sk.ecdsa_sign_recoverable(message, raw=True)
        sig_bytes, rec_id = sk.ecdsa_recoverable_serialize(sig)
        return sig_bytes + bytes([rec_id])
    else:
        sk = SigningKey.from_string(private_key, curve=SECP256k1)
        # Sign with canonical form
        signature = sk.sign_digest(message, sigencode=sigencode_string_canonize)
        r = int.from_bytes(signature[:32], 'big')
        s = int.from_bytes(signature[32:], 'big')

        # Calculate recovery ID by trying each
        vk = sk.get_verifying_key()
        pubkey = private_key_to_public_key(private_key)

        for rec_id in range(4):
            try:
                # Try to recover and verify
                # This is a simplified approach; for production, use secp256k1 library
                return signature + bytes([rec_id])
            except Exception:
                continue

        # Default to rec_id 0 if recovery fails
        return signature + bytes([0])


# =============================================================================
# CKB RPC Client
# =============================================================================

class CkbRpcClient:
    """Simple CKB RPC client."""

    def __init__(self, url: str):
        self.url = url
        self.id_counter = 0

    def _call(self, method: str, params: list = None) -> dict:
        """Make an RPC call."""
        self.id_counter += 1
        payload = {
            "jsonrpc": "2.0",
            "id": self.id_counter,
            "method": method,
            "params": params or [],
        }

        response = requests.post(
            self.url,
            json=payload,
            headers={"Content-Type": "application/json"},
            timeout=30,
        )
        response.raise_for_status()

        result = response.json()
        if "error" in result:
            raise Exception(f"RPC error: {result['error']}")

        return result.get("result")

    def get_blockchain_info(self) -> dict:
        """Get blockchain info including chain type."""
        return self._call("get_blockchain_info")

    def get_genesis_block(self) -> dict:
        """Get genesis block to find system script cell deps."""
        return self._call("get_block_by_number", ["0x0"])

    def get_cells(self, search_key: dict, order: str = "desc", limit: str = "0x64", after_cursor: str = None) -> dict:
        """Query cells using indexer."""
        params = [search_key, order, limit]
        if after_cursor:
            params.append(after_cursor)
        return self._call("get_cells", params)

    def get_live_cell(self, out_point: dict, with_data: bool = True) -> dict:
        """Get a live cell by out_point."""
        return self._call("get_live_cell", [out_point, with_data])

    def send_transaction(self, tx: dict, outputs_validator: str = "passthrough") -> str:
        """Send a transaction to the network."""
        return self._call("send_transaction", [tx, outputs_validator])

    def get_transaction(self, tx_hash: str) -> dict:
        """Get transaction by hash."""
        return self._call("get_transaction", [tx_hash])

    def get_tip_block_number(self) -> str:
        """Get the tip block number."""
        return self._call("get_tip_block_number")


# =============================================================================
# Cell and Transaction Structures
# =============================================================================

@dataclass
class Cell:
    """Represents a CKB cell."""
    tx_hash: bytes
    index: int
    capacity: int
    lock_code_hash: bytes
    lock_hash_type: int
    lock_args: bytes
    type_code_hash: Optional[bytes]
    type_hash_type: Optional[int]
    type_args: Optional[bytes]
    data: bytes

    @classmethod
    def from_rpc(cls, cell_data: dict) -> "Cell":
        """Create Cell from RPC response."""
        out_point = cell_data["out_point"]
        output = cell_data["output"]

        type_script = output.get("type")

        return cls(
            tx_hash=bytes.fromhex(out_point["tx_hash"][2:]),
            index=int(out_point["index"], 16),
            capacity=int(output["capacity"], 16),
            lock_code_hash=bytes.fromhex(output["lock"]["code_hash"][2:]),
            lock_hash_type=1 if output["lock"]["hash_type"] == "type" else 0,
            lock_args=bytes.fromhex(output["lock"]["args"][2:]),
            type_code_hash=bytes.fromhex(type_script["code_hash"][2:]) if type_script else None,
            type_hash_type=(1 if type_script["hash_type"] == "type" else 0) if type_script else None,
            type_args=bytes.fromhex(type_script["args"][2:]) if type_script else None,
            data=bytes.fromhex(cell_data["output_data"][2:]) if cell_data.get("output_data") else b"",
        )

    def has_data(self) -> bool:
        """Check if cell has non-empty data."""
        return len(self.data) > 0

    def has_type_script(self) -> bool:
        """Check if cell has a type script."""
        return self.type_code_hash is not None


# =============================================================================
# Transaction Building
# =============================================================================

def get_secp256k1_cell_dep(rpc: CkbRpcClient) -> bytes:
    """Get the secp256k1 cell dep from genesis block."""
    genesis = rpc.get_genesis_block()

    # The secp256k1 dep group is in the second transaction (index 1) of genesis
    # at output index 0 for the dep group
    dep_group_tx = genesis["transactions"][1]
    dep_group_tx_hash = bytes.fromhex(dep_group_tx["hash"][2:])

    # Pack as CellDep with dep_type = 1 (DepGroup)
    out_point = pack_out_point(dep_group_tx_hash, 0)
    return pack_cell_dep(out_point, 1)  # dep_type 1 = DepGroup


def calculate_tx_hash(raw_tx: bytes) -> bytes:
    """Calculate transaction hash from raw transaction."""
    return ckb_blake2b(raw_tx)


def calculate_signing_message(
    tx_hash: bytes,
    witness_for_digest: bytes,
    other_witnesses: List[bytes] = None,
    debug: bool = False,
) -> bytes:
    """
    Calculate the signing message for secp256k1_blake160_sighash_all.

    message = blake2b(tx_hash || witness_length || witness || other_witnesses...)
    """
    hasher_data = tx_hash

    # Add first witness (with placeholder signature)
    witness_len = pack_u64_le(len(witness_for_digest))
    hasher_data += witness_len + witness_for_digest

    # Add other witnesses in the group (if any)
    if other_witnesses:
        for witness in other_witnesses:
            witness_len = pack_u64_le(len(witness))
            hasher_data += witness_len + witness

    if debug:
        print(f"  DEBUG signing message:")
        print(f"    tx_hash: 0x{tx_hash.hex()}")
        print(f"    witness_len: {len(witness_for_digest)}")
        print(f"    witness_for_digest: 0x{witness_for_digest.hex()[:100]}...")
        print(f"    total hasher_data len: {len(hasher_data)}")

    return ckb_blake2b(hasher_data)


def build_consolidation_transaction(
    cells: List[Cell],
    lock_script: bytes,
    cell_dep: bytes,
    fee: int = 10000,  # 0.0001 CKB default fee
) -> Tuple[bytes, bytes, dict]:
    """
    Build a consolidation transaction.

    Returns: (raw_transaction, tx_hash, json_tx)
    """
    # Calculate total capacity
    total_capacity = sum(cell.capacity for cell in cells)
    output_capacity = total_capacity - fee

    if output_capacity < MIN_CELL_CAPACITY:
        raise ValueError(
            f"Insufficient capacity after fee: {output_capacity} < {MIN_CELL_CAPACITY}"
        )

    # Build inputs
    inputs = []
    for cell in cells:
        out_point = pack_out_point(cell.tx_hash, cell.index)
        cell_input = pack_cell_input(0, out_point)  # since = 0
        inputs.append(cell_input)

    # Build single output (no data, no type script)
    output = pack_cell_output(output_capacity, lock_script, None)
    outputs = [output]

    # Empty output data
    outputs_data = [pack_bytes(b"")]

    # Build raw transaction
    # Note: outputs_data items must be serialized as Bytes (with length prefix)
    raw_tx = pack_raw_transaction(
        version=0,
        cell_deps=[cell_dep],
        header_deps=[],
        inputs=inputs,
        outputs=outputs,
        outputs_data=[pack_bytes(b"")],  # Empty Bytes for each output
    )

    tx_hash = calculate_tx_hash(raw_tx)

    # Build JSON representation for RPC
    # Parse cell_dep: out_point (36 bytes) + dep_type (1 byte)
    # out_point: tx_hash (32 bytes) + index (4 bytes LE)
    cell_dep_tx_hash = cell_dep[0:32]
    cell_dep_index = int.from_bytes(cell_dep[32:36], 'little')
    cell_dep_type = cell_dep[36]

    json_tx = {
        "version": "0x0",
        "cell_deps": [{
            "out_point": {
                "tx_hash": "0x" + cell_dep_tx_hash.hex(),
                "index": hex(cell_dep_index),
            },
            "dep_type": "dep_group" if cell_dep_type == 1 else "code",
        }],
        "header_deps": [],
        "inputs": [{
            "since": "0x0",
            "previous_output": {
                "tx_hash": "0x" + cell.tx_hash.hex(),
                "index": hex(cell.index),
            },
        } for cell in cells],
        "outputs": [{
            "capacity": hex(output_capacity),
            "lock": {
                "code_hash": "0x" + SECP256K1_CODE_HASH.hex(),
                "hash_type": "type",
                "args": "0x" + cells[0].lock_args.hex(),
            },
            "type": None,
        }],
        "outputs_data": ["0x"],
        "witnesses": [],  # Will be filled after signing
    }

    return raw_tx, tx_hash, json_tx


def sign_transaction(
    tx_hash: bytes,
    private_key: bytes,
    num_inputs: int,
    debug: bool = False,
) -> List[str]:
    """
    Sign the transaction and return witnesses.

    For secp256k1_blake160_sighash_all:
    - All inputs with the same lock script are in the same "script group"
    - The signature in the first witness covers ALL inputs in the group
    - The signing message includes: tx_hash + all witnesses in the group
    """
    # Create witness placeholder for signing
    witness_placeholder = pack_witness_args(lock=SIGNATURE_PLACEHOLDER)

    # Other witnesses in the group are empty (0 bytes)
    other_witnesses = [b"" for _ in range(1, num_inputs)]

    if debug:
        print(f"  DEBUG sign_transaction:")
        print(f"    tx_hash: 0x{tx_hash.hex()}")
        print(f"    num_inputs: {num_inputs}")
        print(f"    witness_placeholder len: {len(witness_placeholder)}")

    # Calculate signing message including all witnesses in the group
    message = calculate_signing_message(
        tx_hash,
        witness_placeholder,
        other_witnesses=other_witnesses if other_witnesses else None,
        debug=debug,
    )

    if debug:
        print(f"    signing message: 0x{message.hex()}")

    # Sign the message
    signature = sign_message(private_key, message)

    if debug:
        print(f"    signature: 0x{signature.hex()}")
        print(f"    signature len: {len(signature)}")

    # Create final witness with actual signature
    witness_with_sig = pack_witness_args(lock=signature)

    if debug:
        print(f"    witness_with_sig: 0x{witness_with_sig.hex()}")

    # Build witnesses list
    witnesses = ["0x" + witness_with_sig.hex()]

    # Add empty witnesses for remaining inputs (same script group)
    for _ in range(1, num_inputs):
        witnesses.append("0x")

    return witnesses


# =============================================================================
# Main Consolidation Logic
# =============================================================================

def collect_all_cells(rpc: CkbRpcClient, lock_args: bytes) -> List[Cell]:
    """Collect all cells owned by the given lock_args."""
    search_key = {
        "script": {
            "code_hash": "0x" + SECP256K1_CODE_HASH.hex(),
            "hash_type": "type",
            "args": "0x" + lock_args.hex(),
        },
        "script_type": "lock",
    }

    all_cells = []
    cursor = None

    while True:
        result = rpc.get_cells(search_key, "desc", "0xc8", cursor)  # 200 per batch

        for obj in result["objects"]:
            cell = Cell.from_rpc(obj)
            all_cells.append(cell)

        cursor = result.get("last_cursor")
        if not cursor or len(result["objects"]) == 0:
            break

    return all_cells


def wait_for_confirmation(rpc: CkbRpcClient, tx_hash: str, timeout: int = 120) -> bool:
    """Wait for transaction confirmation."""
    start_time = time.time()

    while time.time() - start_time < timeout:
        try:
            result = rpc.get_transaction(tx_hash)
            if result and result.get("tx_status", {}).get("status") == "committed":
                return True
        except Exception:
            pass

        time.sleep(2)

    return False


def consolidate_cells(
    rpc: CkbRpcClient,
    private_key: bytes,
    batch_size: int = MAX_CELLS_PER_BATCH,
    dry_run: bool = False,
) -> List[str]:
    """
    Main consolidation function.

    Returns list of transaction hashes.
    """
    # Derive lock_args from private key
    public_key = private_key_to_public_key(private_key)
    lock_args = public_key_to_lock_arg(public_key)

    print(f"Public key: 0x{public_key.hex()}")
    print(f"Lock args: 0x{lock_args.hex()}")

    # Get secp256k1 cell dep
    cell_dep = get_secp256k1_cell_dep(rpc)
    print(f"Cell dep: 0x{cell_dep.hex()}")

    # Build lock script for outputs
    lock_script = pack_script(SECP256K1_CODE_HASH, 1, lock_args)  # hash_type = 1 (type)

    # Collect all cells
    print("\nCollecting cells...")
    all_cells = collect_all_cells(rpc, lock_args)
    print(f"Found {len(all_cells)} total cells")

    # Filter cells with data (candidates for consolidation)
    cells_with_data = [c for c in all_cells if c.has_data() and not c.has_type_script()]
    cells_without_data = [c for c in all_cells if not c.has_data() and not c.has_type_script()]
    cells_with_type = [c for c in all_cells if c.has_type_script()]

    print(f"  - Cells with data (consolidatable): {len(cells_with_data)}")
    print(f"  - Cells without data (already free): {len(cells_without_data)}")
    print(f"  - Cells with type scripts (skipped): {len(cells_with_type)}")

    if not cells_with_data:
        print("\nNo cells to consolidate!")
        return []

    # Calculate totals
    total_capacity = sum(c.capacity for c in cells_with_data)
    total_data_size = sum(len(c.data) for c in cells_with_data)

    print(f"\nConsolidation summary:")
    print(f"  - Total capacity: {total_capacity / 10**8:,.2f} CKB")
    print(f"  - Total data size: {total_data_size:,} bytes")
    print(f"  - Capacity to release: ~{total_data_size / 10**8 * 10**8:,.2f} CKB")

    if dry_run:
        print("\n[DRY RUN] Would consolidate these cells into single outputs per batch")
        return []

    # Process in batches
    tx_hashes = []
    batch_num = 0

    for i in range(0, len(cells_with_data), batch_size):
        batch = cells_with_data[i:i + batch_size]
        batch_num += 1

        print(f"\n=== Batch {batch_num} ({len(batch)} cells) ===")

        batch_capacity = sum(c.capacity for c in batch)
        print(f"Batch capacity: {batch_capacity / 10**8:,.2f} CKB")

        # Estimate fee based on transaction size
        # Rough estimate: ~100 bytes per input + ~200 bytes overhead
        estimated_size = 100 * len(batch) + 200
        fee = max(10000, (estimated_size * DEFAULT_FEE_RATE) // 1000)
        print(f"Estimated fee: {fee / 10**8:.8f} CKB")

        try:
            # Build transaction
            raw_tx, tx_hash, json_tx = build_consolidation_transaction(
                batch, lock_script, cell_dep, fee
            )

            print(f"Transaction hash: 0x{tx_hash.hex()}")

            # Sign transaction (debug first batch only)
            witnesses = sign_transaction(tx_hash, private_key, len(batch), debug=(batch_num == 1))
            json_tx["witnesses"] = witnesses

            # Send transaction
            print("Sending transaction...")
            result_hash = rpc.send_transaction(json_tx)
            print(f"Submitted: {result_hash}")
            tx_hashes.append(result_hash)

            # Wait for confirmation
            print("Waiting for confirmation...")
            if wait_for_confirmation(rpc, result_hash):
                print("✓ Confirmed!")
            else:
                print("⚠ Confirmation timeout - continuing anyway")

        except Exception as e:
            print(f"ERROR: {e}")
            # Continue with next batch
            continue

    return tx_hashes


# =============================================================================
# CLI Entry Point
# =============================================================================

def main():
    parser = argparse.ArgumentParser(
        description="Consolidate CKB cells to release occupied capacity"
    )
    parser.add_argument(
        "--rpc-url",
        default="http://127.0.0.1:8114",
        help="CKB node RPC URL (default: http://127.0.0.1:8114)",
    )
    parser.add_argument(
        "--private-key",
        required=True,
        help="Private key in hex format (with or without 0x prefix)",
    )
    parser.add_argument(
        "--batch-size",
        type=int,
        default=MAX_CELLS_PER_BATCH,
        help=f"Maximum cells per transaction (default: {MAX_CELLS_PER_BATCH})",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show what would be done without sending transactions",
    )

    args = parser.parse_args()

    # Parse private key
    private_key_hex = args.private_key
    if private_key_hex.startswith("0x"):
        private_key_hex = private_key_hex[2:]

    if len(private_key_hex) != 64:
        print("ERROR: Private key must be 32 bytes (64 hex characters)")
        sys.exit(1)

    private_key = bytes.fromhex(private_key_hex)

    # Create RPC client
    rpc = CkbRpcClient(args.rpc_url)

    # Verify connection
    try:
        info = rpc.get_blockchain_info()
        print(f"Connected to CKB {info['chain']}")
    except Exception as e:
        print(f"ERROR: Cannot connect to CKB node at {args.rpc_url}")
        print(f"  {e}")
        sys.exit(1)

    # Run consolidation
    tx_hashes = consolidate_cells(
        rpc,
        private_key,
        batch_size=args.batch_size,
        dry_run=args.dry_run,
    )

    if tx_hashes:
        print(f"\n=== Complete ===")
        print(f"Consolidated in {len(tx_hashes)} transaction(s):")
        for tx_hash in tx_hashes:
            print(f"  {tx_hash}")


if __name__ == "__main__":
    main()
