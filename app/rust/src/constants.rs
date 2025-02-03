pub const ADDR_INDEX_LEN: usize = 16;

pub const SIGNATURE_LEN: usize = 64;

// Diversifier: 16 bytes
// Transmission Key: 32 bytes
// Clue Key: 32 bytes
// Total: 16 + 32 + 32 = 80 bytes
// this len is before F4Jumble is applied
// and bech32 encoding is appliend
pub const ADDRESS_LEN: usize = 80;

pub const KEY_LEN: usize = 32;
pub const FVK_LEN: usize = 64;
pub const DIVERSIFIER_KEY_LEN: usize = 16;
pub const OUTGOING_VIEWING_KEY_LEN: usize = KEY_LEN;
pub const NULLIFIER_KEY_LEN: usize = KEY_LEN; // Assuming decaf377 curve parameters
pub const SPEND_AUTHORIZATION_KEY_LEN: usize = KEY_LEN; // Assuming encoded size
pub const SPEND_VERIFICATION_KEY_LEN: usize = KEY_LEN; // Assuming encoded size
pub const INCOMING_VIEWING_KEY_LEN: usize = KEY_LEN; //
/// The maximum detection precision, chosen so that the message bits fit in 3 bytes.
pub const MAX_PRECISION: u8 = 24;
pub const PAYLOAD_KEY_LEN_BYTES: usize = 32;
pub const RSEED_LEN_BYTES: usize = 32;
pub const ID_LEN_BYTES: usize = 32;
pub const AMOUNT_LEN_BYTES: usize = 16;

pub const DETECTION_DATA_QTY: usize = 16;
pub const ACTION_DATA_QTY: usize = 16;
pub const MAX_CLUE_SUBKEYS: usize = 10;
pub const MAX_REWARDS: usize = 5;

pub const EFFECT_HASH_LEN: usize = 64;
pub const UI_ADDRESS_LEN: usize = 37;

// Nonces:
pub const NONCE_LEN: usize = 12;
pub const NONCE_NOTE: &[u8; NONCE_LEN] = &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
pub const NONCE_MEMO_KEYS: &[u8; NONCE_LEN] = &[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
pub const NONCE_SWAP: &[u8; NONCE_LEN] = &[2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
pub const NONCE_MEMO: &[u8; NONCE_LEN] = &[3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

pub const MEMO_CIPHERTEXT_LEN_BYTES: usize = 528;
// This is the `MEMO_CIPHERTEXT_LEN_BYTES` - MAC size (16 bytes).
pub const MEMO_LEN_BYTES: usize = 512;
// This is the largest text length we can support
pub const MAX_TEXT_LEN: usize = MEMO_LEN_BYTES - ADDRESS_LEN;

// Swap ciphertext byte length.
pub const SWAP_CIPHERTEXT_BYTES: usize = 272;
// Swap plaintext byte length.
pub const SWAP_LEN_BYTES: usize = 256;
