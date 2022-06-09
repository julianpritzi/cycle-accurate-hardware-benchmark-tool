//! FFI Code for the opentitan ecdsa library
#![allow(dead_code)]

use core::mem;

use super::otbn::otbn_error_t;

/// This is a boolean type for use in hardened contexts.
/// The intention is that this is used instead of a bool, where a
/// higher hamming distance is required between the truthy and the falsey value.
/// The values below were chosen at random, with some specific restrictions. They
/// have a Hamming Distance of 8, and they are 11-bit values so they can be
/// materialized with a single instruction on RISC-V. They are also specifically
/// not the complement of each other.
#[repr(C)]
pub enum hardened_bool_t {
    /// The truthy value, expected to be used like #true.
    HardenedBoolTrue = 0x739,
    /// The falsey value, expected to be used like #false.
    HardenedBoolFalse = 0x1d4,
    /// Custom invalid value for initialization.
    HardenedBoolInvalid = 0,
}

/// Length of a P-256 curve point coordinate in bits (integer modulo the "p"
/// parameter, see FIPS 186-4 section D.1.2.3)
const K_P256_COORD_NUM_BITS: usize = 256;

/// Length of a P-256 curve point coordinate in words
const K_P256_COORD_NUM_WORDS: usize = K_P256_COORD_NUM_BITS / (mem::size_of::<u32>() * 8);

/// Length of a number modulo the P-256 "n" parameter (see FIPS 186-4 section D.1.2.3) in bits
const K_P256_SCALAR_NUM_BITS: usize = 256;

/// Length of a number modulo the P-256 "n" parameter in words
const K_P256_SCALAR_NUM_WORDS: usize = K_P256_SCALAR_NUM_BITS / (mem::size_of::<u32>() * 8);

/// Length of the message digest in bits
const K_P256_MESSAGE_DIGEST_NUM_BITS: usize = 256;

/// A type that holds an ECDSA/P-256 signature.
///
/// The signature consists of two integers r and s, computed modulo n.
#[repr(C)]
pub struct ecdsa_p256_signature_t {
    pub r: [u32; K_P256_SCALAR_NUM_WORDS],
    pub s: [u32; K_P256_SCALAR_NUM_WORDS],
}

/// A type that holds an ECDSA/P-256 private key.
///
/// The private key consists of a single integer d, computed modulo n.
#[repr(C)]
pub struct ecdsa_p256_private_key_t {
    pub d: [u32; K_P256_SCALAR_NUM_WORDS],
}

/// A type that holds an ECDSA/P-256 public key.
///
/// The public key is a point Q on the p256 curve, consisting of two coordinates
/// x and y computed modulo p.
#[repr(C)]
pub struct ecdsa_p256_public_key_t {
    pub x: [u32; K_P256_COORD_NUM_WORDS],
    pub y: [u32; K_P256_COORD_NUM_WORDS],
}

/// A type that holds an ECDSA/P-256 message digest.
///
/// The message digest is expected to already be transformed into an integer
/// h = H(msg) mod n, where H is the hash function.
#[repr(C)]
pub struct ecdsa_p256_message_digest_t {
    pub h: [u32; K_P256_SCALAR_NUM_WORDS],
}

#[link(name = "sw_lib_crypto_ecdsa_p256")]
extern "C" {

    /// Generates an ECDSA/P-256 signature.
    ///
    /// # Arguments
    ///
    /// * `message_digest` - Digest of the message to sign.
    /// * `private_key` - Key to sign the message with.
    /// * `result` - Buffer in which to store the generated signature.
    pub fn ecdsa_p256_sign(
        digest: *const ecdsa_p256_message_digest_t,
        private_key: *const ecdsa_p256_private_key_t,
        result: *mut ecdsa_p256_signature_t,
    ) -> otbn_error_t;

    /// Verifies an ECDSA/P-256 signature.
    ///
    /// # Arguments
    ///
    /// * `signature` - Signature to be verified.
    /// * `message_digest` - Digest of the message to check the signature against.
    /// * `public_key` - Key to check the signature against.
    /// * `result` - Buffer in which to store output (true iff signature is valid)
    pub fn ecdsa_p256_verify(
        signature: *const ecdsa_p256_signature_t,
        digest: *const ecdsa_p256_message_digest_t,
        public_key: *const ecdsa_p256_public_key_t,
        result: *mut hardened_bool_t,
    ) -> otbn_error_t;
}

#[link(name = "p256_ecdsa")]
extern "C" {}
