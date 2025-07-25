use bls12_381::{
    Fp, Fp2, G1Affine, G1Projective, G2Affine, G2Prepared, G2Projective, Gt, Scalar,
    hash_to_curve::MapToCurve, multi_miller_loop,
};

use bytes::{Buf, Bytes};
use ethrex_common::{
    Address, H160, H256, U256, kzg::verify_kzg_proof, serde_utils::bool, types::Fork,
    utils::u256_from_big_endian,
};
use ethrex_crypto::blake2f::blake2f_compress_f;
use keccak_hash::keccak256;
use lambdaworks_math::{
    cyclic_group::IsGroup,
    elliptic_curve::{
        short_weierstrass::{
            curves::bn_254::{
                curve::{BN254Curve, BN254FieldElement, BN254TwistCurveFieldElement},
                field_extension::{
                    BN254FieldModulus, Degree2ExtensionField, Degree12ExtensionField,
                },
                pairing::BN254AtePairing,
                twist::BN254TwistCurve,
            },
            point::ShortWeierstrassProjectivePoint,
        },
        traits::{IsEllipticCurve, IsPairing},
    },
    field::{
        element::FieldElement, extensions::quadratic::QuadraticExtensionFieldElement,
        fields::montgomery_backed_prime_fields::MontgomeryBackendPrimeField,
    },
    traits::ByteConversion,
    unsigned_integer::element,
};
use num_bigint::BigUint;
use secp256k1::{
    Message,
    ecdsa::{RecoverableSignature, RecoveryId},
};

use sha3::Digest;
use std::ops::Mul;

use crate::{
    constants::VERSIONED_HASH_VERSION_KZG,
    errors::{ExceptionalHalt, InternalError, PrecompileError, VMError},
    gas_cost::{
        self, BLAKE2F_ROUND_COST, BLS12_381_G1_K_DISCOUNT, BLS12_381_G1ADD_COST,
        BLS12_381_G2_K_DISCOUNT, BLS12_381_G2ADD_COST, BLS12_381_MAP_FP_TO_G1_COST,
        BLS12_381_MAP_FP2_TO_G2_COST, ECADD_COST, ECMUL_COST, ECRECOVER_COST, G1_MUL_COST,
        G2_MUL_COST, MODEXP_STATIC_COST, POINT_EVALUATION_COST,
    },
};

pub const ECRECOVER_ADDRESS: H160 = H160([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x01,
]);
pub const SHA2_256_ADDRESS: H160 = H160([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x02,
]);
pub const RIPEMD_160_ADDRESS: H160 = H160([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x03,
]);
pub const IDENTITY_ADDRESS: H160 = H160([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x04,
]);
pub const MODEXP_ADDRESS: H160 = H160([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x05,
]);
pub const ECADD_ADDRESS: H160 = H160([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x06,
]);
pub const ECMUL_ADDRESS: H160 = H160([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x07,
]);
pub const ECPAIRING_ADDRESS: H160 = H160([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x08,
]);
pub const BLAKE2F_ADDRESS: H160 = H160([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x09,
]);
pub const POINT_EVALUATION_ADDRESS: H160 = H160([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x0a,
]);
pub const BLS12_G1ADD_ADDRESS: H160 = H160([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x0b,
]);
pub const BLS12_G1MSM_ADDRESS: H160 = H160([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x0c,
]);
pub const BLS12_G2ADD_ADDRESS: H160 = H160([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x0d,
]);
pub const BLS12_G2MSM_ADDRESS: H160 = H160([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x0e,
]);
pub const BLS12_PAIRING_CHECK_ADDRESS: H160 = H160([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x0f,
]);
pub const BLS12_MAP_FP_TO_G1_ADDRESS: H160 = H160([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x10,
]);
pub const BLS12_MAP_FP2_TO_G2_ADDRESS: H160 = H160([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x11,
]);

pub const PRECOMPILES: [H160; 10] = [
    ECRECOVER_ADDRESS,
    SHA2_256_ADDRESS,
    RIPEMD_160_ADDRESS,
    IDENTITY_ADDRESS,
    MODEXP_ADDRESS,
    ECADD_ADDRESS,
    ECMUL_ADDRESS,
    ECPAIRING_ADDRESS,
    BLAKE2F_ADDRESS,
    POINT_EVALUATION_ADDRESS,
];

pub const PRECOMPILES_POST_CANCUN: [H160; 7] = [
    BLS12_G1ADD_ADDRESS,
    BLS12_G1MSM_ADDRESS,
    BLS12_G2ADD_ADDRESS,
    BLS12_G2MSM_ADDRESS,
    BLS12_PAIRING_CHECK_ADDRESS,
    BLS12_MAP_FP_TO_G1_ADDRESS,
    BLS12_MAP_FP2_TO_G2_ADDRESS,
];

pub const BLAKE2F_ELEMENT_SIZE: usize = 8;

pub const SIZE_PRECOMPILES_PRE_CANCUN: u64 = 9;
pub const SIZE_PRECOMPILES_CANCUN: u64 = 10;
pub const SIZE_PRECOMPILES_PRAGUE: u64 = 17;

pub const BLS12_381_G1_MSM_PAIR_LENGTH: usize = 160;
pub const BLS12_381_G2_MSM_PAIR_LENGTH: usize = 288;
pub const BLS12_381_PAIRING_CHECK_PAIR_LENGTH: usize = 384;

const BLS12_381_G1ADD_VALID_INPUT_LENGTH: usize = 256;
const BLS12_381_G2ADD_VALID_INPUT_LENGTH: usize = 512;

const BLS12_381_FP2_VALID_INPUT_LENGTH: usize = 128;
const BLS12_381_FP_VALID_INPUT_LENGTH: usize = 64;

pub const FIELD_ELEMENT_WITHOUT_PADDING_LENGTH: usize = 48;
pub const PADDED_FIELD_ELEMENT_SIZE_IN_BYTES: usize = 64;

const FP2_ZERO_MAPPED_TO_G2: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 131, 32, 137, 110, 201, 238, 249, 213, 230,
    25, 132, 141, 194, 156, 226, 102, 244, 19, 208, 45, 211, 29, 155, 157, 68, 236, 12, 121, 205,
    97, 241, 139, 7, 93, 219, 166, 215, 189, 32, 183, 255, 39, 164, 179, 36, 191, 206, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 103, 209, 33, 24, 181, 163, 91, 176, 45, 46, 134, 179,
    235, 250, 126, 35, 65, 13, 185, 61, 227, 159, 176, 109, 112, 37, 250, 149, 233, 111, 250, 66,
    138, 122, 39, 195, 174, 77, 212, 180, 11, 210, 81, 172, 101, 136, 146, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 2, 96, 224, 54, 68, 209, 162, 195, 33, 37, 107, 50, 70, 186, 210, 184,
    149, 202, 209, 56, 144, 203, 230, 248, 93, 245, 81, 6, 160, 211, 52, 96, 79, 177, 67, 199, 160,
    66, 216, 120, 0, 98, 113, 134, 91, 195, 89, 65, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    4, 198, 151, 119, 164, 63, 11, 218, 7, 103, 157, 88, 5, 230, 63, 24, 207, 78, 14, 124, 97, 18,
    172, 127, 112, 38, 109, 25, 155, 79, 118, 174, 39, 198, 38, 154, 60, 238, 189, 174, 48, 128,
    110, 154, 118, 170, 223, 92,
];
pub const G1_POINT_AT_INFINITY: [u8; 128] = [0_u8; 128];
pub const G2_POINT_AT_INFINITY: [u8; 256] = [0_u8; 256];

pub fn is_precompile(address: &Address, fork: Fork) -> bool {
    // Cancun specs is the only one that allows point evaluation precompile
    if *address == POINT_EVALUATION_ADDRESS && fork < Fork::Cancun {
        return false;
    }
    // Prague or newers forks should only use this precompiles
    // https://eips.ethereum.org/EIPS/eip-2537
    if PRECOMPILES_POST_CANCUN.contains(address) && fork < Fork::Prague {
        return false;
    }

    PRECOMPILES.contains(address) || PRECOMPILES_POST_CANCUN.contains(address)
}

pub fn execute_precompile(
    address: Address,
    calldata: &Bytes,
    gas_remaining: &mut u64,
) -> Result<Bytes, VMError> {
    let result = match address {
        address if address == ECRECOVER_ADDRESS => ecrecover(calldata, gas_remaining)?,
        address if address == IDENTITY_ADDRESS => identity(calldata, gas_remaining)?,
        address if address == SHA2_256_ADDRESS => sha2_256(calldata, gas_remaining)?,
        address if address == RIPEMD_160_ADDRESS => ripemd_160(calldata, gas_remaining)?,
        address if address == MODEXP_ADDRESS => modexp(calldata, gas_remaining)?,
        address if address == ECADD_ADDRESS => ecadd(calldata, gas_remaining)?,
        address if address == ECMUL_ADDRESS => ecmul(calldata, gas_remaining)?,
        address if address == ECPAIRING_ADDRESS => ecpairing(calldata, gas_remaining)?,
        address if address == BLAKE2F_ADDRESS => blake2f(calldata, gas_remaining)?,
        address if address == POINT_EVALUATION_ADDRESS => {
            point_evaluation(calldata, gas_remaining)?
        }
        address if address == BLS12_G1ADD_ADDRESS => bls12_g1add(calldata, gas_remaining)?,
        address if address == BLS12_G1MSM_ADDRESS => bls12_g1msm(calldata, gas_remaining)?,
        address if address == BLS12_G2ADD_ADDRESS => bls12_g2add(calldata, gas_remaining)?,
        address if address == BLS12_G2MSM_ADDRESS => bls12_g2msm(calldata, gas_remaining)?,
        address if address == BLS12_PAIRING_CHECK_ADDRESS => {
            bls12_pairing_check(calldata, gas_remaining)?
        }
        address if address == BLS12_MAP_FP_TO_G1_ADDRESS => {
            bls12_map_fp_to_g1(calldata, gas_remaining)?
        }
        address if address == BLS12_MAP_FP2_TO_G2_ADDRESS => {
            bls12_map_fp2_tp_g2(calldata, gas_remaining)?
        }
        _ => return Err(InternalError::InvalidPrecompileAddress.into()),
    };

    Ok(result)
}

/// Consumes gas and if it's higher than the gas limit returns an error.
pub(crate) fn increase_precompile_consumed_gas(
    gas_cost: u64,
    gas_remaining: &mut u64,
) -> Result<(), VMError> {
    *gas_remaining = gas_remaining
        .checked_sub(gas_cost)
        .ok_or(PrecompileError::NotEnoughGas)?;
    Ok(())
}

/// When slice length is less than `target_len`, the rest is filled with zeros. If slice length is
/// more than `target_len`, the excess bytes are discarded.
pub(crate) fn fill_with_zeros(calldata: &Bytes, target_len: usize) -> Bytes {
    let mut padded_calldata = calldata.to_vec();
    if padded_calldata.len() < target_len {
        padded_calldata.resize(target_len, 0);
    }
    padded_calldata.into()
}

/// ECDSA (Elliptic curve digital signature algorithm) public key recovery function.
/// Given a hash, a Signature and a recovery Id, returns the public key recovered by secp256k1
pub fn ecrecover(calldata: &Bytes, gas_remaining: &mut u64) -> Result<Bytes, VMError> {
    let gas_cost = ECRECOVER_COST;

    increase_precompile_consumed_gas(gas_cost, gas_remaining)?;

    // If calldata does not reach the required length, we should fill the rest with zeros
    let calldata = fill_with_zeros(calldata, 128);

    // Parse the input elements, first as a slice of bytes and then as an specific type of the crate
    let hash = calldata.get(0..32).ok_or(InternalError::Slicing)?;
    let Ok(message) = Message::from_digest_slice(hash) else {
        return Ok(Bytes::new());
    };

    let v = u256_from_big_endian(calldata.get(32..64).ok_or(InternalError::Slicing)?);

    // The Recovery identifier is expected to be 27 or 28, any other value is invalid
    if !(v == U256::from(27) || v == U256::from(28)) {
        return Ok(Bytes::new());
    }

    let v = u8::try_from(v).map_err(|_| InternalError::TypeConversion)?;
    let recovery_id_from_rpc = v.checked_sub(27).ok_or(InternalError::TypeConversion)?;
    let Ok(recovery_id) = RecoveryId::from_i32(recovery_id_from_rpc.into()) else {
        return Ok(Bytes::new());
    };

    // signature is made up of the parameters r and s
    let sig = calldata.get(64..128).ok_or(InternalError::Slicing)?;
    let Ok(signature) = RecoverableSignature::from_compact(sig, recovery_id) else {
        return Ok(Bytes::new());
    };

    // Recover the address using secp256k1
    let Ok(public_key) = signature.recover(&message) else {
        return Ok(Bytes::new());
    };

    let mut public_key = public_key.serialize_uncompressed();

    // We need to take the 64 bytes from the public key (discarding the first pos of the slice)
    keccak256(&mut public_key[1..65]);

    // The output is 32 bytes: the initial 12 bytes with 0s, and the remaining 20 with the recovered address
    let mut output = vec![0u8; 12];
    output.extend_from_slice(public_key.get(13..33).ok_or(InternalError::Slicing)?);

    Ok(Bytes::from(output.to_vec()))
}

/// Returns the calldata received
pub fn identity(calldata: &Bytes, gas_remaining: &mut u64) -> Result<Bytes, VMError> {
    let gas_cost = gas_cost::identity(calldata.len())?;

    increase_precompile_consumed_gas(gas_cost, gas_remaining)?;

    Ok(calldata.clone())
}

/// Returns the calldata hashed by sha2-256 algorithm
pub fn sha2_256(calldata: &Bytes, gas_remaining: &mut u64) -> Result<Bytes, VMError> {
    let gas_cost = gas_cost::sha2_256(calldata.len())?;

    increase_precompile_consumed_gas(gas_cost, gas_remaining)?;

    let result = sha2::Sha256::digest(calldata).to_vec();

    Ok(Bytes::from(result))
}

/// Returns the calldata hashed by ripemd-160 algorithm, padded by zeros at left
pub fn ripemd_160(calldata: &Bytes, gas_remaining: &mut u64) -> Result<Bytes, VMError> {
    let gas_cost = gas_cost::ripemd_160(calldata.len())?;

    increase_precompile_consumed_gas(gas_cost, gas_remaining)?;

    let mut hasher = ripemd::Ripemd160::new();
    hasher.update(calldata);
    let result = hasher.finalize();

    let mut output = vec![0; 12];
    output.extend_from_slice(&result);

    Ok(Bytes::from(output))
}

/// Returns the result of the module-exponentiation operation
pub fn modexp(calldata: &Bytes, gas_remaining: &mut u64) -> Result<Bytes, VMError> {
    // If calldata does not reach the required length, we should fill the rest with zeros
    let calldata = fill_with_zeros(calldata, 96);

    let base_size = U256::from_big_endian(calldata.get(0..32).ok_or(InternalError::Slicing)?);
    let exponent_size = U256::from_big_endian(calldata.get(32..64).ok_or(InternalError::Slicing)?);
    let modulus_size = U256::from_big_endian(calldata.get(64..96).ok_or(InternalError::Slicing)?);

    if base_size == U256::zero() && modulus_size == U256::zero() {
        // On Berlin or newer there is a floor cost for the modexp precompile
        increase_precompile_consumed_gas(MODEXP_STATIC_COST, gas_remaining)?;

        return Ok(Bytes::new());
    }

    // Because on some cases conversions to usize exploded before the check of the zero value could be done
    let base_size = usize::try_from(base_size).map_err(|_| PrecompileError::ParsingInputError)?;
    let exponent_size =
        usize::try_from(exponent_size).map_err(|_| PrecompileError::ParsingInputError)?;
    let modulus_size =
        usize::try_from(modulus_size).map_err(|_| PrecompileError::ParsingInputError)?;

    let base_limit = base_size.checked_add(96).ok_or(InternalError::Overflow)?;

    let exponent_limit = exponent_size
        .checked_add(base_limit)
        .ok_or(InternalError::Overflow)?;

    let modulus_limit = modulus_size
        .checked_add(exponent_limit)
        .ok_or(InternalError::Overflow)?;

    let b = get_slice_or_default(&calldata, 96, base_limit, base_size)?;
    let base = BigUint::from_bytes_be(&b);

    let e = get_slice_or_default(&calldata, base_limit, exponent_limit, exponent_size)?;
    let exponent = BigUint::from_bytes_be(&e);

    let m = get_slice_or_default(&calldata, exponent_limit, modulus_limit, modulus_size)?;
    let modulus = BigUint::from_bytes_be(&m);

    // First 32 bytes of exponent or exponent if e_size < 32
    let bytes_to_take = 32.min(exponent_size);
    // Use of unwrap_or_default because if e == 0 get_slice_or_default returns an empty vec
    let exp_first_32 = BigUint::from_bytes_be(e.get(0..bytes_to_take).unwrap_or_default());

    let gas_cost = gas_cost::modexp(&exp_first_32, base_size, exponent_size, modulus_size)?;

    increase_precompile_consumed_gas(gas_cost, gas_remaining)?;

    let result = mod_exp(base, exponent, modulus);

    let res_bytes = result.to_bytes_be();
    let res_bytes = increase_left_pad(&Bytes::from(res_bytes), modulus_size)?;

    Ok(res_bytes.slice(..modulus_size))
}

/// This function returns the slice between the lower and upper limit of the calldata (as a vector),
/// padding with zeros at the end if necessary.
fn get_slice_or_default(
    calldata: &Bytes,
    lower_limit: usize,
    upper_limit: usize,
    size_to_expand: usize,
) -> Result<Vec<u8>, VMError> {
    let upper_limit = calldata.len().min(upper_limit);
    if let Some(data) = calldata.get(lower_limit..upper_limit) {
        if !data.is_empty() {
            let mut extended = vec![0u8; size_to_expand];
            for (dest, data) in extended.iter_mut().zip(data.iter()) {
                *dest = *data;
            }
            return Ok(extended);
        }
    }
    Ok(Default::default())
}

/// I allow this clippy alert because in the code modulus could never be
///  zero because that case is covered in the if above that line
#[allow(clippy::arithmetic_side_effects)]
fn mod_exp(base: BigUint, exponent: BigUint, modulus: BigUint) -> BigUint {
    if modulus == BigUint::ZERO {
        BigUint::ZERO
    } else if exponent == BigUint::ZERO {
        BigUint::from(1_u8) % modulus
    } else {
        base.modpow(&exponent, &modulus)
    }
}

/// If the result size is less than needed, pads left with zeros.
pub fn increase_left_pad(result: &Bytes, m_size: usize) -> Result<Bytes, VMError> {
    let mut padded_result = vec![0u8; m_size];
    if result.len() < m_size {
        let size_diff = m_size
            .checked_sub(result.len())
            .ok_or(InternalError::Underflow)?;
        padded_result
            .get_mut(size_diff..)
            .ok_or(InternalError::Slicing)?
            .copy_from_slice(result);

        Ok(padded_result.into())
    } else {
        Ok(result.clone())
    }
}

/// Makes a point addition on the elliptic curve 'alt_bn128'
pub fn ecadd(calldata: &Bytes, gas_remaining: &mut u64) -> Result<Bytes, VMError> {
    // If calldata does not reach the required length, we should fill the rest with zeros
    let calldata = fill_with_zeros(calldata, 128);

    increase_precompile_consumed_gas(ECADD_COST, gas_remaining)?;

    let first_point_x = calldata.get(0..32).ok_or(InternalError::Slicing)?;
    let first_point_y = calldata.get(32..64).ok_or(InternalError::Slicing)?;
    let second_point_x = calldata.get(64..96).ok_or(InternalError::Slicing)?;
    let second_point_y = calldata.get(96..128).ok_or(InternalError::Slicing)?;

    // If points are zero the precompile should not fail, but the conversion in
    // BN254Curve::create_point_from_affine will, so we verify it before the conversion
    let first_point_is_zero = u256_from_big_endian(first_point_x).is_zero()
        && u256_from_big_endian(first_point_y).is_zero();
    let second_point_is_zero = u256_from_big_endian(second_point_x).is_zero()
        && u256_from_big_endian(second_point_y).is_zero();

    let first_point_x = BN254FieldElement::from_bytes_be(first_point_x)
        .map_err(|_| PrecompileError::ParsingInputError)?;
    let first_point_y = BN254FieldElement::from_bytes_be(first_point_y)
        .map_err(|_| PrecompileError::ParsingInputError)?;
    let second_point_x = BN254FieldElement::from_bytes_be(second_point_x)
        .map_err(|_| PrecompileError::ParsingInputError)?;
    let second_point_y = BN254FieldElement::from_bytes_be(second_point_y)
        .map_err(|_| PrecompileError::ParsingInputError)?;

    if first_point_is_zero && second_point_is_zero {
        // If both points are zero, return is zero
        Ok(Bytes::from([0u8; 64].to_vec()))
    } else if first_point_is_zero {
        // If first point is zero, return is second point
        let second_point = BN254Curve::create_point_from_affine(second_point_x, second_point_y)
            .map_err(|_| PrecompileError::ParsingInputError)?;
        let res = [
            second_point.x().to_bytes_be(),
            second_point.y().to_bytes_be(),
        ]
        .concat();
        Ok(Bytes::from(res))
    } else if second_point_is_zero {
        // If second point is zero, return is first point
        let first_point = BN254Curve::create_point_from_affine(first_point_x, first_point_y)
            .map_err(|_| PrecompileError::ParsingInputError)?;
        let res = [first_point.x().to_bytes_be(), first_point.y().to_bytes_be()].concat();
        Ok(Bytes::from(res))
    } else {
        // If none of the points is zero, return is the sum of both in the EC
        let first_point = BN254Curve::create_point_from_affine(first_point_x, first_point_y)
            .map_err(|_| PrecompileError::ParsingInputError)?;
        let second_point = BN254Curve::create_point_from_affine(second_point_x, second_point_y)
            .map_err(|_| PrecompileError::ParsingInputError)?;
        let sum = first_point.operate_with(&second_point).to_affine();

        if u256_from_big_endian(&sum.x().to_bytes_be()) == U256::zero()
            || u256_from_big_endian(&sum.y().to_bytes_be()) == U256::zero()
        {
            Ok(Bytes::from([0u8; 64].to_vec()))
        } else {
            let res = [sum.x().to_bytes_be(), sum.y().to_bytes_be()].concat();
            Ok(Bytes::from(res))
        }
    }
}

/// Makes a scalar multiplication on the elliptic curve 'alt_bn128'
pub fn ecmul(calldata: &Bytes, gas_remaining: &mut u64) -> Result<Bytes, VMError> {
    // If calldata does not reach the required length, we should fill the rest with zeros
    let calldata = fill_with_zeros(calldata, 96);

    increase_precompile_consumed_gas(ECMUL_COST, gas_remaining)?;

    let point_x = calldata.get(0..32).ok_or(InternalError::Slicing)?;
    let point_y = calldata.get(32..64).ok_or(InternalError::Slicing)?;
    let scalar = calldata.get(64..96).ok_or(InternalError::Slicing)?;

    let scalar = element::U256::from_bytes_be(scalar).map_err(|_| InternalError::Slicing)?;

    // If point is zero the precompile should not fail, but the conversion in
    // BN254Curve::create_point_from_affine will, so we verify it before the conversion
    let point_is_zero =
        u256_from_big_endian(point_x).is_zero() && u256_from_big_endian(point_y).is_zero();
    if point_is_zero {
        return Ok(Bytes::from([0u8; 64].to_vec()));
    }

    let point_x = BN254FieldElement::from_bytes_be(point_x).map_err(|_| InternalError::Slicing)?;
    let point_y = BN254FieldElement::from_bytes_be(point_y).map_err(|_| InternalError::Slicing)?;

    let point = BN254Curve::create_point_from_affine(point_x, point_y)
        .map_err(|_| PrecompileError::ParsingInputError)?;

    let zero_u256 = element::U256::from(0_u16);
    if scalar.eq(&zero_u256) {
        Ok(Bytes::from([0u8; 64].to_vec()))
    } else {
        let mul = point.operate_with_self(scalar).to_affine();
        if u256_from_big_endian(&mul.x().to_bytes_be()) == U256::zero()
            || u256_from_big_endian(&mul.y().to_bytes_be()) == U256::zero()
        {
            Ok(Bytes::from([0u8; 64].to_vec()))
        } else {
            let res = [mul.x().to_bytes_be(), mul.y().to_bytes_be()].concat();
            Ok(Bytes::from(res))
        }
    }
}

const ALT_BN128_PRIME: U256 = U256([
    0x3c208c16d87cfd47,
    0x97816a916871ca8d,
    0xb85045b68181585d,
    0x30644e72e131a029,
]);

type FirstPointCoordinates = (
    FieldElement<MontgomeryBackendPrimeField<BN254FieldModulus, 4>>,
    FieldElement<MontgomeryBackendPrimeField<BN254FieldModulus, 4>>,
);

/// Parses first point coordinates and makes verification of invalid infinite
fn parse_first_point_coordinates(input_data: &[u8]) -> Result<FirstPointCoordinates, VMError> {
    let first_point_x = input_data.get(..32).ok_or(InternalError::Slicing)?;
    let first_point_y = input_data.get(32..64).ok_or(InternalError::Slicing)?;

    // Infinite is defined by (0,0). Any other zero-combination is invalid
    if (u256_from_big_endian(first_point_x) == U256::zero())
        ^ (u256_from_big_endian(first_point_y) == U256::zero())
    {
        return Err(PrecompileError::InvalidPoint.into());
    }

    let first_point_y = BN254FieldElement::from_bytes_be(first_point_y)
        .map_err(|_| InternalError::msg("Failed to create BN254 element from bytes"))?;
    let first_point_x = BN254FieldElement::from_bytes_be(first_point_x)
        .map_err(|_| InternalError::msg("Failed to create BN254 element from bytes"))?;

    Ok((first_point_x, first_point_y))
}

/// Parses second point coordinates and makes verification of invalid infinite and curve belonging.
fn parse_second_point_coordinates(
    input_data: &[u8],
) -> Result<
    (
        FieldElement<Degree2ExtensionField>,
        FieldElement<Degree2ExtensionField>,
    ),
    VMError,
> {
    let second_point_x_first_part = input_data.get(96..128).ok_or(InternalError::Slicing)?;
    let second_point_x_second_part = input_data.get(64..96).ok_or(InternalError::Slicing)?;

    // Infinite is defined by (0,0). Any other zero-combination is invalid
    if (u256_from_big_endian(second_point_x_first_part) == U256::zero())
        ^ (u256_from_big_endian(second_point_x_second_part) == U256::zero())
    {
        return Err(PrecompileError::InvalidPoint.into());
    }

    let second_point_y_first_part = input_data.get(160..192).ok_or(InternalError::Slicing)?;
    let second_point_y_second_part = input_data.get(128..160).ok_or(InternalError::Slicing)?;

    // Infinite is defined by (0,0). Any other zero-combination is invalid
    if (u256_from_big_endian(second_point_y_first_part) == U256::zero())
        ^ (u256_from_big_endian(second_point_y_second_part) == U256::zero())
    {
        return Err(PrecompileError::InvalidPoint.into());
    }

    // Check if the second point belongs to the curve (this happens if it's lower than the prime)
    if u256_from_big_endian(second_point_x_first_part) >= ALT_BN128_PRIME
        || u256_from_big_endian(second_point_x_second_part) >= ALT_BN128_PRIME
        || u256_from_big_endian(second_point_y_first_part) >= ALT_BN128_PRIME
        || u256_from_big_endian(second_point_y_second_part) >= ALT_BN128_PRIME
    {
        return Err(PrecompileError::PointNotInTheCurve.into());
    }

    let second_point_x_bytes = [second_point_x_first_part, second_point_x_second_part].concat();
    let second_point_y_bytes = [second_point_y_first_part, second_point_y_second_part].concat();

    let second_point_x = BN254TwistCurveFieldElement::from_bytes_be(&second_point_x_bytes)
        .map_err(|_| InternalError::msg("Failed to create BN254 element from bytes"))?;
    let second_point_y = BN254TwistCurveFieldElement::from_bytes_be(&second_point_y_bytes)
        .map_err(|_| InternalError::msg("Failed to create BN254 element from bytes"))?;

    Ok((second_point_x, second_point_y))
}

/// Handles pairing given a certain elements, and depending on if elements represent infinity, then
/// continues, verifies errors on the other point or calculates the pairing
fn handle_pairing_from_coordinates(
    first_point_x: FieldElement<MontgomeryBackendPrimeField<BN254FieldModulus, 4>>,
    first_point_y: FieldElement<MontgomeryBackendPrimeField<BN254FieldModulus, 4>>,
    second_point_x: FieldElement<Degree2ExtensionField>,
    second_point_y: FieldElement<Degree2ExtensionField>,
    mul: &mut FieldElement<Degree12ExtensionField>,
) -> Result<bool, VMError> {
    let zero_element = BN254FieldElement::from(0);
    let twcurve_zero_element = BN254TwistCurveFieldElement::from(0);
    let first_point_is_infinity =
        first_point_x.eq(&zero_element) && first_point_y.eq(&zero_element);
    let second_point_is_infinity =
        second_point_x.eq(&twcurve_zero_element) && second_point_y.eq(&twcurve_zero_element);

    match (first_point_is_infinity, second_point_is_infinity) {
        (true, true) => {
            // If both points are infinity, then continue to the next input
            Ok(true)
        }
        (true, false) => {
            // If the first point is infinity, then do the checks for the second
            let p2 = BN254TwistCurve::create_point_from_affine(
                second_point_x.clone(),
                second_point_y.clone(),
            )
            .map_err(|_| PrecompileError::InvalidPoint)?;

            if !p2.is_in_subgroup() {
                return Err(PrecompileError::PointNotInSubgroup.into());
            }
            Ok(true)
        }
        (false, true) => {
            // If the second point is infinity, then do the checks for the first
            BN254Curve::create_point_from_affine(first_point_x.clone(), first_point_y.clone())
                .map_err(|_| PrecompileError::InvalidPoint)?;
            Ok(true)
        }
        (false, false) => {
            // Define the pairing points
            let first_point = BN254Curve::create_point_from_affine(first_point_x, first_point_y)
                .map_err(|_| PrecompileError::InvalidPoint)?;

            let second_point =
                BN254TwistCurve::create_point_from_affine(second_point_x, second_point_y)
                    .map_err(|_| PrecompileError::InvalidPoint)?;
            if !second_point.is_in_subgroup() {
                return Err(PrecompileError::PointNotInSubgroup.into());
            }

            // Get the result of the pairing and affect the mul value with it
            update_pairing_result(mul, first_point, second_point)?;
            Ok(false)
        }
    }
}

/// Performs a bilinear pairing on points on the elliptic curve 'alt_bn128', returns 1 on success and 0 on failure
pub fn ecpairing(calldata: &Bytes, gas_remaining: &mut u64) -> Result<Bytes, VMError> {
    // The input must always be a multiple of 192 (6 32-byte values)
    if calldata.len() % 192 != 0 {
        return Err(PrecompileError::ParsingInputError.into());
    }

    let inputs_amount = calldata.len() / 192;

    // Consume gas
    let gas_cost = gas_cost::ecpairing(inputs_amount)?;
    increase_precompile_consumed_gas(gas_cost, gas_remaining)?;

    let mut mul: FieldElement<Degree12ExtensionField> = QuadraticExtensionFieldElement::one();
    for input_index in 0..inputs_amount {
        // Define the input indexes and slice calldata to get the input data
        let input_start = input_index
            .checked_mul(192)
            .ok_or(InternalError::Overflow)?;
        let input_end = input_start
            .checked_add(192)
            .ok_or(InternalError::Overflow)?;

        let input_data = calldata
            .get(input_start..input_end)
            .ok_or(InternalError::Slicing)?;

        let (first_point_x, first_point_y) = parse_first_point_coordinates(input_data)?;

        let (second_point_x, second_point_y) = parse_second_point_coordinates(input_data)?;

        if handle_pairing_from_coordinates(
            first_point_x,
            first_point_y,
            second_point_x,
            second_point_y,
            &mut mul,
        )? {
            continue;
        }
    }

    // Generate the result from the variable mul
    let success = mul.eq(&QuadraticExtensionFieldElement::one());
    let mut result = [0; 32];
    result[31] = u8::from(success);
    Ok(Bytes::from(result.to_vec()))
}

/// Updates the success variable with the pairing result. I allow this clippy alert because lib handles
/// mul for the type and will not panic in case of overflow
#[allow(clippy::arithmetic_side_effects)]
fn update_pairing_result(
    mul: &mut FieldElement<Degree12ExtensionField>,
    first_point: ShortWeierstrassProjectivePoint<BN254Curve>,
    second_point: ShortWeierstrassProjectivePoint<BN254TwistCurve>,
) -> Result<(), VMError> {
    let pairing_result = BN254AtePairing::compute_batch(&[(&first_point, &second_point)])
        .map_err(|_| PrecompileError::BN254AtePairingError)?;

    *mul *= pairing_result;

    Ok(())
}

/// Returns the result of Blake2 hashing algorithm given a certain parameters from the calldata.
pub fn blake2f(calldata: &Bytes, gas_remaining: &mut u64) -> Result<Bytes, VMError> {
    if calldata.len() != 213 {
        return Err(PrecompileError::ParsingInputError.into());
    }

    let mut calldata = calldata.slice(0..213);

    let rounds = calldata.get_u32();

    let gas_cost = u64::from(rounds) * BLAKE2F_ROUND_COST;
    increase_precompile_consumed_gas(gas_cost, gas_remaining)?;

    let mut h = [0; 8];

    h.copy_from_slice(&std::array::from_fn::<u64, 8, _>(|_| calldata.get_u64_le()));

    let mut m = [0; 16];

    m.copy_from_slice(&std::array::from_fn::<u64, 16, _>(|_| {
        calldata.get_u64_le()
    }));

    let mut t = [0; 2];
    t.copy_from_slice(&std::array::from_fn::<u64, 2, _>(|_| calldata.get_u64_le()));

    let f = calldata.get_u8();
    if f != 0 && f != 1 {
        return Err(PrecompileError::ParsingInputError.into());
    }
    let f = f == 1;

    #[expect(clippy::as_conversions)] // safe to convert a u32 to usize
    let result = blake2f_compress_f(rounds as usize, &h, &m, &t, f);

    // map the result to the output format (from a u64 slice to a u8 one)
    let output: Vec<u8> = result.iter().flat_map(|num| num.to_le_bytes()).collect();

    Ok(Bytes::from(output))
}

/// Converts the provided commitment to match the provided versioned_hash.
/// Taken from the same name function from crates/common/types/blobs_bundle.rs
fn kzg_commitment_to_versioned_hash(commitment_bytes: &[u8; 48]) -> H256 {
    use sha2::{Digest, Sha256};
    let mut versioned_hash: [u8; 32] = Sha256::digest(commitment_bytes).into();
    versioned_hash[0] = VERSIONED_HASH_VERSION_KZG;
    versioned_hash.into()
}

const POINT_EVALUATION_OUTPUT_BYTES: [u8; 64] = [
    // Big endian FIELD_ELEMENTS_PER_BLOB bytes
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00,
    // Big endian BLS_MODULUS bytes
    0x73, 0xED, 0xA7, 0x53, 0x29, 0x9D, 0x7D, 0x48, 0x33, 0x39, 0xD8, 0x08, 0x09, 0xA1, 0xD8, 0x05,
    0x53, 0xBD, 0xA4, 0x02, 0xFF, 0xFE, 0x5B, 0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x01,
];

/// Makes verifications on the received point, proof and commitment, if true returns a constant value
fn point_evaluation(calldata: &Bytes, gas_remaining: &mut u64) -> Result<Bytes, VMError> {
    if calldata.len() != 192 {
        return Err(PrecompileError::ParsingInputError.into());
    }

    // Consume gas
    let gas_cost = POINT_EVALUATION_COST;
    increase_precompile_consumed_gas(gas_cost, gas_remaining)?;

    // Parse inputs
    let versioned_hash: [u8; 32] = calldata
        .get(..32)
        .ok_or(InternalError::Slicing)?
        .try_into()
        .map_err(|_| InternalError::TypeConversion)?;

    let x: [u8; 32] = calldata
        .get(32..64)
        .ok_or(InternalError::Slicing)?
        .try_into()
        .map_err(|_| InternalError::TypeConversion)?;

    let y: [u8; 32] = calldata
        .get(64..96)
        .ok_or(InternalError::Slicing)?
        .try_into()
        .map_err(|_| InternalError::TypeConversion)?;

    let commitment: [u8; 48] = calldata
        .get(96..144)
        .ok_or(InternalError::Slicing)?
        .try_into()
        .map_err(|_| InternalError::TypeConversion)?;

    let proof: [u8; 48] = calldata
        .get(144..192)
        .ok_or(InternalError::Slicing)?
        .try_into()
        .map_err(|_| InternalError::TypeConversion)?;

    // Perform the evaluation

    // This checks if the commitment is equal to the versioned hash
    if kzg_commitment_to_versioned_hash(&commitment) != H256::from(versioned_hash) {
        return Err(PrecompileError::ParsingInputError.into());
    }

    // This verifies the proof from a point (x, y) and a commitment
    if !verify_kzg_proof(commitment, x, y, proof).unwrap_or(false) {
        return Err(PrecompileError::ParsingInputError.into());
    }

    // The first 32 bytes consist of the number of field elements in the blob, and the
    // other 32 bytes consist of the modulus used in the BLS signature scheme.
    let output = POINT_EVALUATION_OUTPUT_BYTES.to_vec();

    Ok(Bytes::from(output))
}

pub fn bls12_g1add(calldata: &Bytes, gas_remaining: &mut u64) -> Result<Bytes, VMError> {
    // Two inputs of 128 bytes are required
    if calldata.len() != BLS12_381_G1ADD_VALID_INPUT_LENGTH {
        return Err(PrecompileError::ParsingInputError.into());
    }

    // GAS
    increase_precompile_consumed_gas(BLS12_381_G1ADD_COST, gas_remaining)
        .map_err(|_| PrecompileError::NotEnoughGas)?;

    let first_g1_point = parse_g1_point(calldata.get(0..128), true)?;
    let second_g1_point = parse_g1_point(calldata.get(128..256), true)?;

    let result_of_addition = G1Affine::from(first_g1_point.add(&second_g1_point));

    let result_bytes = if result_of_addition.is_identity().into() {
        return Ok(Bytes::copy_from_slice(&G1_POINT_AT_INFINITY));
    } else {
        result_of_addition.to_uncompressed()
    };

    let mut padded_result = Vec::new();
    add_padded_coordinate(&mut padded_result, result_bytes.get(0..48))?;
    add_padded_coordinate(&mut padded_result, result_bytes.get(48..96))?;

    Ok(Bytes::from(padded_result))
}

pub fn bls12_g1msm(calldata: &Bytes, gas_remaining: &mut u64) -> Result<Bytes, VMError> {
    if calldata.is_empty() || calldata.len() % BLS12_381_G1_MSM_PAIR_LENGTH != 0 {
        return Err(PrecompileError::ParsingInputError.into());
    }

    let k = calldata.len() / BLS12_381_G1_MSM_PAIR_LENGTH;
    let required_gas = gas_cost::bls12_msm(k, &BLS12_381_G1_K_DISCOUNT, G1_MUL_COST)?;
    increase_precompile_consumed_gas(required_gas, gas_remaining)?;

    let mut result = G1Projective::identity();
    // R = s_P_1 + s_P_2 + ... + s_P_k
    // Where:
    // s_i are scalars (numbers)
    // P_i are points in the group (in this case, points in G1)
    for i in 0..k {
        let point_offset = i
            .checked_mul(BLS12_381_G1_MSM_PAIR_LENGTH)
            .ok_or(InternalError::Overflow)?;
        let scalar_offset = point_offset
            .checked_add(128)
            .ok_or(InternalError::Overflow)?;
        let pair_end = scalar_offset
            .checked_add(32)
            .ok_or(InternalError::Overflow)?;

        let point = parse_g1_point(calldata.get(point_offset..scalar_offset), false)?;
        let scalar = parse_scalar(calldata.get(scalar_offset..pair_end))?;

        let scaled_point = G1Projective::mul(point, scalar);
        result = result.add(&scaled_point);
    }
    let mut output = [0u8; 128];

    if result.is_identity().into() {
        return Ok(Bytes::copy_from_slice(&output));
    }
    let result_bytes = G1Affine::from(result).to_uncompressed();
    let (x_bytes, y_bytes) = result_bytes
        .split_at_checked(FIELD_ELEMENT_WITHOUT_PADDING_LENGTH)
        .ok_or(InternalError::Slicing)?;
    output[16..64].copy_from_slice(x_bytes);
    output[80..128].copy_from_slice(y_bytes);

    Ok(Bytes::copy_from_slice(&output))
}

pub fn bls12_g2add(calldata: &Bytes, gas_remaining: &mut u64) -> Result<Bytes, VMError> {
    if calldata.len() != BLS12_381_G2ADD_VALID_INPUT_LENGTH {
        return Err(PrecompileError::ParsingInputError.into());
    }

    // GAS
    increase_precompile_consumed_gas(BLS12_381_G2ADD_COST, gas_remaining)
        .map_err(|_| PrecompileError::NotEnoughGas)?;

    let first_g2_point = parse_g2_point(calldata.get(0..256), true)?;
    let second_g2_point = parse_g2_point(calldata.get(256..512), true)?;

    let result_of_addition = G2Affine::from(first_g2_point.add(&second_g2_point));

    let result_bytes = if result_of_addition.is_identity().into() {
        return Ok(Bytes::copy_from_slice(&G2_POINT_AT_INFINITY));
    } else {
        result_of_addition.to_uncompressed()
    };

    let mut padded_result = Vec::new();
    // The crate bls12_381 deserialize the G2 point as x_1 || x_0 || y_1 || y_0
    // https://docs.rs/bls12_381/0.8.0/src/bls12_381/g2.rs.html#284-299
    add_padded_coordinate(&mut padded_result, result_bytes.get(48..96))?;
    add_padded_coordinate(&mut padded_result, result_bytes.get(0..48))?;
    add_padded_coordinate(&mut padded_result, result_bytes.get(144..192))?;
    add_padded_coordinate(&mut padded_result, result_bytes.get(96..144))?;

    Ok(Bytes::from(padded_result))
}

pub fn bls12_g2msm(calldata: &Bytes, gas_remaining: &mut u64) -> Result<Bytes, VMError> {
    if calldata.is_empty() || calldata.len() % BLS12_381_G2_MSM_PAIR_LENGTH != 0 {
        return Err(PrecompileError::ParsingInputError.into());
    }

    let k = calldata.len() / BLS12_381_G2_MSM_PAIR_LENGTH;
    let required_gas = gas_cost::bls12_msm(k, &BLS12_381_G2_K_DISCOUNT, G2_MUL_COST)?;
    increase_precompile_consumed_gas(required_gas, gas_remaining)?;

    let mut result = G2Projective::identity();
    for i in 0..k {
        let point_offset = i
            .checked_mul(BLS12_381_G2_MSM_PAIR_LENGTH)
            .ok_or(InternalError::Overflow)?;
        let scalar_offset = point_offset
            .checked_add(256)
            .ok_or(InternalError::Overflow)?;
        let pair_end = scalar_offset
            .checked_add(32)
            .ok_or(InternalError::Overflow)?;

        let point = parse_g2_point(calldata.get(point_offset..scalar_offset), false)?;
        let scalar = parse_scalar(calldata.get(scalar_offset..pair_end))?;

        let scaled_point = G2Projective::mul(point, scalar);
        result = result.add(&scaled_point);
    }

    let result_bytes = if result.is_identity().into() {
        return Ok(Bytes::copy_from_slice(&G2_POINT_AT_INFINITY));
    } else {
        G2Affine::from(result).to_uncompressed()
    };

    let mut padded_result = Vec::new();
    // The crate bls12_381 deserialize the G2 point as x_1 || x_0 || y_1 || y_0
    // https://docs.rs/bls12_381/0.8.0/src/bls12_381/g2.rs.html#284-299
    add_padded_coordinate(&mut padded_result, result_bytes.get(48..96))?;
    add_padded_coordinate(&mut padded_result, result_bytes.get(0..48))?;
    add_padded_coordinate(&mut padded_result, result_bytes.get(144..192))?;
    add_padded_coordinate(&mut padded_result, result_bytes.get(96..144))?;

    Ok(Bytes::from(padded_result))
}

pub fn bls12_pairing_check(calldata: &Bytes, gas_remaining: &mut u64) -> Result<Bytes, VMError> {
    if calldata.is_empty() || calldata.len() % BLS12_381_PAIRING_CHECK_PAIR_LENGTH != 0 {
        return Err(PrecompileError::ParsingInputError.into());
    }

    // GAS
    let k = calldata.len() / BLS12_381_PAIRING_CHECK_PAIR_LENGTH;
    let gas_cost = gas_cost::bls12_pairing_check(k)?;
    increase_precompile_consumed_gas(gas_cost, gas_remaining)?;

    let mut points: Vec<(G1Affine, G2Prepared)> = Vec::new();
    for i in 0..k {
        let g1_point_offset = i
            .checked_mul(BLS12_381_PAIRING_CHECK_PAIR_LENGTH)
            .ok_or(InternalError::Overflow)?;
        let g2_point_offset = g1_point_offset
            .checked_add(128)
            .ok_or(InternalError::Overflow)?;
        let pair_end = g2_point_offset
            .checked_add(256)
            .ok_or(InternalError::Overflow)?;

        // The check for the subgroup is required
        // https://github.com/ethereum/EIPs/blob/master/EIPS/eip-2537.md?plain=1#L194
        let g1 = G1Affine::from(parse_g1_point(
            calldata.get(g1_point_offset..g2_point_offset),
            false,
        )?);
        let g2 = G2Affine::from(parse_g2_point(
            calldata.get(g2_point_offset..pair_end),
            false,
        )?);
        points.push((g1, G2Prepared::from(g2)));
    }

    // The crate bls12_381 expects a reference to the points
    let points: Vec<(&G1Affine, &G2Prepared)> = points.iter().map(|(g1, g2)| (g1, g2)).collect();

    // perform the final exponentiation to get the result of the pairing check
    // https://docs.rs/bls12_381/0.8.0/src/bls12_381/pairings.rs.html#43-48
    let result: Gt = multi_miller_loop(&points).final_exponentiation();

    // follows this https://github.com/ethereum/EIPs/blob/master/EIPS/eip-2537.md?plain=1#L188
    if result == Gt::identity() {
        let mut result = vec![0_u8; 31];
        result.push(1);
        Ok(Bytes::from(result))
    } else {
        Ok(Bytes::copy_from_slice(&[0_u8; 32]))
    }
}

pub fn bls12_map_fp_to_g1(calldata: &Bytes, gas_remaining: &mut u64) -> Result<Bytes, VMError> {
    if calldata.len() != BLS12_381_FP_VALID_INPUT_LENGTH {
        return Err(PrecompileError::ParsingInputError.into());
    }

    // GAS
    increase_precompile_consumed_gas(BLS12_381_MAP_FP_TO_G1_COST, gas_remaining)?;

    let coordinate_bytes = parse_coordinate(calldata.get(0..PADDED_FIELD_ELEMENT_SIZE_IN_BYTES))?;
    let fp = Fp::from_bytes(&coordinate_bytes)
        .into_option()
        .ok_or(ExceptionalHalt::Precompile(
            PrecompileError::ParsingInputError,
        ))?;

    // following https://github.com/ethereum/EIPs/blob/master/assets/eip-2537/field_to_curve.md?plain=1#L3-L6, we do:
    // map_to_curve: map a field element to a another curve, then isogeny is applied to map to the curve bls12_381
    // clear_h: clears the cofactor
    let point = G1Projective::map_to_curve(&fp).clear_h();

    let result_bytes = if point.is_identity().into() {
        return Ok(Bytes::copy_from_slice(&G1_POINT_AT_INFINITY));
    } else {
        G1Affine::from(point).to_uncompressed()
    };

    let mut padded_result = Vec::new();
    add_padded_coordinate(&mut padded_result, result_bytes.get(0..48))?;
    add_padded_coordinate(&mut padded_result, result_bytes.get(48..96))?;

    Ok(Bytes::from(padded_result))
}

pub fn bls12_map_fp2_tp_g2(calldata: &Bytes, gas_remaining: &mut u64) -> Result<Bytes, VMError> {
    if calldata.len() != BLS12_381_FP2_VALID_INPUT_LENGTH {
        return Err(PrecompileError::ParsingInputError.into());
    }

    // GAS
    increase_precompile_consumed_gas(BLS12_381_MAP_FP2_TO_G2_COST, gas_remaining)?;

    // Parse the input to two Fp and create a Fp2
    let c0 = parse_coordinate(calldata.get(0..PADDED_FIELD_ELEMENT_SIZE_IN_BYTES))?;
    let c1 = parse_coordinate(
        calldata.get(PADDED_FIELD_ELEMENT_SIZE_IN_BYTES..BLS12_381_FP2_VALID_INPUT_LENGTH),
    )?;
    let fp_0 = Fp::from_bytes(&c0)
        .into_option()
        .ok_or(ExceptionalHalt::Precompile(
            PrecompileError::ParsingInputError,
        ))?;
    let fp_1 = Fp::from_bytes(&c1)
        .into_option()
        .ok_or(ExceptionalHalt::Precompile(
            PrecompileError::ParsingInputError,
        ))?;
    if fp_0 == Fp::zero() && fp_1 == Fp::zero() {
        return Ok(Bytes::copy_from_slice(&FP2_ZERO_MAPPED_TO_G2));
    }

    let fp2 = Fp2 { c0: fp_0, c1: fp_1 };

    // following https://github.com/ethereum/EIPs/blob/master/assets/eip-2537/field_to_curve.md?plain=1#L3-L6, we do:
    // map_to_curve: map a field element to a another curve, then isogeny is applied to map to the curve bls12_381
    // clear_h: clears the cofactor
    let point = G2Projective::map_to_curve(&fp2).clear_h();
    let result_bytes = if point.is_identity().into() {
        return Ok(Bytes::copy_from_slice(&G2_POINT_AT_INFINITY));
    } else {
        G2Affine::from(point).to_uncompressed()
    };

    let mut padded_result = Vec::new();
    // The crate bls12_381 deserialize the G2 point as x_1 || x_0 || y_1 || y_0
    // https://docs.rs/bls12_381/0.8.0/src/bls12_381/g2.rs.html#284-299
    add_padded_coordinate(&mut padded_result, result_bytes.get(48..96))?;
    add_padded_coordinate(&mut padded_result, result_bytes.get(0..48))?;
    add_padded_coordinate(&mut padded_result, result_bytes.get(144..192))?;
    add_padded_coordinate(&mut padded_result, result_bytes.get(96..144))?;

    Ok(Bytes::from(padded_result))
}

fn parse_coordinate(coordinate_raw_bytes: Option<&[u8]>) -> Result<[u8; 48], VMError> {
    let sixteen_zeroes: [u8; 16] = [0_u8; 16];
    let padded_coordinate = coordinate_raw_bytes.ok_or(ExceptionalHalt::Precompile(
        PrecompileError::ParsingInputError,
    ))?;
    if !matches!(padded_coordinate.get(0..16), Some(prefix) if prefix == sixteen_zeroes) {
        return Err(PrecompileError::ParsingInputError.into());
    }
    let unpadded_coordinate = padded_coordinate
        .get(16..64)
        .ok_or(ExceptionalHalt::Precompile(
            PrecompileError::ParsingInputError,
        ))?;
    unpadded_coordinate
        .try_into()
        .map_err(|_| PrecompileError::ParsingInputError.into())
}
fn parse_g1_point(
    point_raw_bytes: Option<&[u8]>,
    unchecked: bool,
) -> Result<G1Projective, VMError> {
    let point_bytes = point_raw_bytes.ok_or(ExceptionalHalt::Precompile(
        PrecompileError::ParsingInputError,
    ))?;
    let x = parse_coordinate(point_bytes.get(0..64))?;
    let y = parse_coordinate(point_bytes.get(64..128))?;

    // if a g1 point decode to (0,0) by convention it is interpreted as a point to infinity
    let g1_point: G1Projective = if x.iter().all(|e| *e == 0) && y.iter().all(|e| *e == 0) {
        G1Projective::identity()
    } else {
        let g1_bytes: [u8; 96] = [x, y]
            .concat()
            .try_into()
            .map_err(|_| InternalError::TypeConversion)?;

        if unchecked {
            // We use unchecked because in the https://github.com/ethereum/EIPs/blob/master/EIPS/eip-2537.md?plain=1#L141
            // note that there is no subgroup check for the G1 addition precompile
            let g1_affine = G1Affine::from_uncompressed_unchecked(&g1_bytes)
                .into_option()
                .ok_or(ExceptionalHalt::Precompile(
                    PrecompileError::ParsingInputError,
                ))?;

            // We still need to check if the point is on the curve
            if !bool::from(g1_affine.is_on_curve()) {
                return Err(ExceptionalHalt::Precompile(
                    PrecompileError::BLS12381G1PointNotInCurve,
                )
                .into());
            }

            G1Projective::from(g1_affine)
        } else {
            let g1_affine = G1Affine::from_uncompressed(&g1_bytes)
                .into_option()
                .ok_or(PrecompileError::ParsingInputError)?;

            G1Projective::from(g1_affine)
        }
    };
    Ok(g1_point)
}

fn parse_g2_point(
    point_raw_bytes: Option<&[u8]>,
    unchecked: bool,
) -> Result<G2Projective, VMError> {
    let point_bytes = point_raw_bytes.ok_or(ExceptionalHalt::Precompile(
        PrecompileError::ParsingInputError,
    ))?;
    let x_0 = parse_coordinate(point_bytes.get(0..64))?;
    let x_1 = parse_coordinate(point_bytes.get(64..128))?;
    let y_0 = parse_coordinate(point_bytes.get(128..192))?;
    let y_1 = parse_coordinate(point_bytes.get(192..256))?;

    // if a g1 point decode to (0,0) by convention it is interpreted as a point to infinity
    let g2_point: G2Projective = if x_0.iter().all(|e| *e == 0)
        && x_1.iter().all(|e| *e == 0)
        && y_0.iter().all(|e| *e == 0)
        && y_1.iter().all(|e| *e == 0)
    {
        G2Projective::identity()
    } else {
        // The crate serialize the coordinates in a reverse order
        // https://docs.rs/bls12_381/0.8.0/src/bls12_381/g2.rs.html#401-464
        let g2_bytes: [u8; 192] = [x_1, x_0, y_1, y_0]
            .concat()
            .try_into()
            .map_err(|_| InternalError::TypeConversion)?;

        if unchecked {
            // We use unchecked because in the https://github.com/ethereum/EIPs/blob/master/EIPS/eip-2537.md?plain=1#L141
            // note that there is no subgroup check for the G1 addition precompile
            let g2_affine = G2Affine::from_uncompressed_unchecked(&g2_bytes)
                .into_option()
                .ok_or(ExceptionalHalt::Precompile(
                    PrecompileError::ParsingInputError,
                ))?;

            // We still need to check if the point is on the curve
            if !bool::from(g2_affine.is_on_curve()) {
                return Err(ExceptionalHalt::Precompile(
                    PrecompileError::BLS12381G2PointNotInCurve,
                )
                .into());
            }

            G2Projective::from(g2_affine)
        } else {
            let g2_affine = G2Affine::from_uncompressed(&g2_bytes)
                .into_option()
                .ok_or(PrecompileError::ParsingInputError)?;

            G2Projective::from(g2_affine)
        }
    };
    Ok(g2_point)
}

fn add_padded_coordinate(
    result: &mut Vec<u8>,
    coordinate_raw_bytes: Option<&[u8]>,
) -> Result<(), VMError> {
    // add the padding to satisfy the convention of encoding
    // https://eips.ethereum.org/EIPS/eip-2537
    let sixteen_zeroes: [u8; 16] = [0_u8; 16];
    result.extend_from_slice(&sixteen_zeroes);
    result.extend_from_slice(coordinate_raw_bytes.ok_or(InternalError::Slicing)?);
    Ok(())
}

fn parse_scalar(scalar_raw_bytes: Option<&[u8]>) -> Result<Scalar, VMError> {
    let scalar_bytes: [u8; 32] = scalar_raw_bytes
        .ok_or(InternalError::Slicing)?
        .try_into()
        .map_err(|_| PrecompileError::ParsingInputError)?;

    let mut scalar_le = [0u64; 4];
    for (j, chunk) in scalar_bytes.chunks(8).enumerate() {
        let bytes: [u8; 8] = chunk
            .try_into()
            .map_err(|_| PrecompileError::ParsingInputError)?;
        if let Some(value) = scalar_le.get_mut(j) {
            *value = u64::from_be_bytes(bytes);
        } else {
            return Err(InternalError::Slicing.into());
        }
    }
    scalar_le.reverse();
    Ok(Scalar::from_raw(scalar_le))
}
