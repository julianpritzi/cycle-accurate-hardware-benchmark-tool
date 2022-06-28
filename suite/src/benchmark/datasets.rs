#![allow(dead_code)]
/// Contains predefined datasets with precomputed results

pub mod sha256 {
    /// Contains some data and a valid sha256 digest of the data
    pub struct Sha256Data {
        pub data: &'static [u32],
        pub digest: &'static [u32; 8],
    }

    pub const DATASETS: [Sha256Data; 2] = [
        Sha256Data {
            data: &DATA_1,
            digest: &DIGEST_1,
        },
        Sha256Data {
            data: &DATA_2,
            digest: &DIGEST_2,
        },
    ];

    const DATA_1: [u32; 64] = [
        0xdf3f6198, 0x04a92fdb, 0x4057192d, 0xc43dd748, 0xea778adc, 0x52bc498c, 0xe80524c0,
        0x14b81119, 0xdf3f6198, 0x04a92fdb, 0x4057192d, 0xc43dd748, 0xea778adc, 0x52bc498c,
        0xe80524c0, 0x14b81119, 0xdf3f6198, 0x04a92fdb, 0x4057192d, 0xc43dd748, 0xea778adc,
        0x52bc498c, 0xe80524c0, 0x14b81119, 0xdf3f6198, 0x04a92fdb, 0x4057192d, 0xc43dd748,
        0xea778adc, 0x52bc498c, 0xe80524c0, 0x14b81119, 0xdf3f6198, 0x04a92fdb, 0x4057192d,
        0xc43dd748, 0xea778adc, 0x52bc498c, 0xe80524c0, 0x14b81119, 0xdf3f6198, 0x04a92fdb,
        0x4057192d, 0xc43dd748, 0xea778adc, 0x52bc498c, 0xe80524c0, 0x14b81119, 0xdf3f6198,
        0x04a92fdb, 0x4057192d, 0xc43dd748, 0xea778adc, 0x52bc498c, 0xe80524c0, 0x14b81119,
        0xdf3f6198, 0x04a92fdb, 0x4057192d, 0xc43dd748, 0xea778adc, 0x52bc498c, 0xe80524c0,
        0x14b81119,
    ];
    const DATA_2: [u32; 1] = [0];

    // Precomputed value by sha2 crate from data_1
    const DIGEST_1: [u32; 8] = [
        0xa24ef743, 0xed238e92, 0x8f5fe495, 0x7959a1fa, 0x06b1d250, 0x147ed98d, 0xd817e3b2,
        0xb32854ae,
    ];
    // Precomputed value by sha2 crate from data_2
    const DIGEST_2: [u32; 8] = [
        0xdf3f6198, 0x04a92fdb, 0x4057192d, 0xc43dd748, 0xea778adc, 0x52bc498c, 0xe80524c0,
        0x14b81119,
    ];
}

pub mod aes {
    use crate::modules::{AESKeyLength, AESMode};

    pub struct AesData {
        pub key_share0: &'static [u32; 8],
        pub key_share1: &'static [u32; 8],
        pub key_length: AESKeyLength,
        pub mode: AESMode,
        pub plaintext: &'static [u128],
        pub ciphertext: &'static [u128],
    }

    pub const DATASETS: [AesData; 2] = [
        AesData {
            key_share0: &KEY_1,
            key_share1: &ZERO_KEY,
            key_length: AESKeyLength::Aes256,
            mode: MODE_1,
            plaintext: &PLAINTEXT_1,
            ciphertext: &CIPHERTEXT_1,
        },
        AesData {
            key_share0: &KEY_2,
            key_share1: &ZERO_KEY,
            key_length: AESKeyLength::Aes256,
            mode: MODE_2,
            plaintext: &PLAINTEXT_2,
            ciphertext: &CIPHERTEXT_2,
        },
    ];

    const ZERO_KEY: [u32; 8] = [0; 8];
    const KEY_1: [u32; 8] = [
        0x0000_1111,
        0x2222_3333,
        0x4444_5555,
        0x6666_7777,
        0x0000_1111,
        0x2222_3333,
        0x4444_5555,
        0x6666_7777,
    ];
    const KEY_2: [u32; 8] = [
        0x0000_1111,
        0x2222_3333,
        0x4444_5555,
        0x6666_7777,
        0x0000_1111,
        0x2222_3333,
        0x4444_5555,
        0x6666_7777,
    ];
    const KEY_3: [u32; 8] = [
        0x0000_1111,
        0x2222_3333,
        0x4444_5555,
        0x6666_7777,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
    ];
    const MODE_1: AESMode = AESMode::CTR {
        iv: 0xcccc_cccc_cccc_cccc_cccc_cccc_cccc_cccc,
    };
    const MODE_2: AESMode = AESMode::CTR {
        iv: 0xfb12_60c5_8b69_93a7_b8c7_7c6e_464a_a903u128,
    };
    const PLAINTEXT_1: [u128; 4] = [
        0xffff_ffff_ffff_ffff_ffff_ffff_ffff_ffff,
        0x0000_0000_0000_0000_0000_0000_0000_0000,
        0x0000_1111_2222_3333_4444_5555_6666_7777,
        0x1234_4321_abcd_dcba_affa_afaf_0100_0010,
    ];
    const PLAINTEXT_2: [u128; 5] = [
        0x12bb_b300_8e5d_392b_eeab_2332_be17_833e,
        0xa1f0_6916_0d57_f83a_a0ba_1311_1e98_709f,
        0x3d05_7c8c_6f2a_1b6e_bf50_2dcb_38cd_60d8,
        0x7c6d_2b00_3232_d98b_a452_627a_fe2f_23dc,
        0xb491_10e4_8ad8_3e04_20e4_348a_82ce_cf15,
    ];
    /// Precomputed using the openssl crate
    /// with the following configuration:
    /// - key_1 & zero_key
    /// - aes256
    /// - mode_1
    /// - plaintext_1
    const CIPHERTEXT_1: [u128; 4] = [
        0xfd0dcbcab0d253425800853d7c871aa4,
        0xce1192022849ba635a02a1b9efabe045,
        0x477120db31cf4dfd849c565ff4f8e932,
        0x1ac8141b6d63a496c015988d5ac71596,
    ];

    /// Precomputed using the openssl crate
    /// with the following configuration:
    /// - key_2 & zero_key
    /// - aes256
    /// - mode_2
    /// - plaintext_2
    const CIPHERTEXT_2: [u128; 5] = [
        0x7b436a4b7d3f339be5e7177bd8921e2f,
        0x1eefd500fd21234297170d075150b292,
        0xbaedb76067736877ac26e465251f1c3a,
        0xfb7511bf323f8851ee66e9c253a07f02,
        0x9255ff0a9b062e8759bd262ee56526bd,
    ];
}

pub mod rng {
    pub struct RngData {
        pub seed: &'static [u32],
        pub values: &'static [u128],
    }

    pub const DATASETS: [RngData; 1] = [RngData {
        seed: &SEED_1,
        values: &VALUES_1,
    }];

    const SEED_1: [u32; 12] = [
        651981, 19191, 165996, 215151, 816547, 20, 0, 1616, 1616651651, 8546, 999, 1561,
    ];
    const VALUES_1: [u128; 3] = [
        153684701634699060983499893045240912715u128,
        301721415404207314724546574610589213438u128,
        64250610127905256792585182264175928463u128,
    ];
}

pub mod ecdsa {
    use crate::libs::ecdsa::{
        ecdsa_p256_message_digest_t, ecdsa_p256_private_key_t, ecdsa_p256_public_key_t,
        ecdsa_p256_signature_t,
    };

    pub struct EcdsaData {
        pub priv_key: &'static ecdsa_p256_private_key_t,
        pub pub_key: &'static ecdsa_p256_public_key_t,
        pub digest: &'static ecdsa_p256_message_digest_t,
        pub signature: &'static ecdsa_p256_signature_t,
    }

    pub const DATASETS: [EcdsaData; 0] = [];

    const PRIV_KEY_1: ecdsa_p256_private_key_t = ecdsa_p256_private_key_t {
        d: [
            0xe32ae325, 0xba720dd6, 0x7a61c7bf, 0x042a9ce2, 0x1caf1e98, 0xdada301d, 0x209ab209,
            0x69d57c5c,
        ],
    };
    const PUB_KEY_1: ecdsa_p256_public_key_t = ecdsa_p256_public_key_t {
        x: [
            0x2119818f, 0x4bf23e33, 0xa6730cc3, 0x7f88c59f, 0xd73e9dab, 0x0e28969b, 0x4560410e,
            0xda6152c2,
        ],
        y: [
            0x9dccc8a7, 0xf2f07fac, 0xb22c083e, 0xf519656d, 0x86ed498a, 0x9eceefab, 0x82219250,
            0x54b75d6a,
        ],
    };
}
