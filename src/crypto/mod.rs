use ed25519_dalek::{SigningKey, VerifyingKey, Signature as EdSignature, Verifier};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PublicKey(pub VerifyingKey);

impl PublicKey {
    pub fn as_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }

    pub fn verify_signature(&self, signature: &Signature, message: &[u8]) -> bool {
        self.0.verify(message, &signature.0).is_ok()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature(pub EdSignature);

impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.to_bytes().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes = <[u8; 32]>::deserialize(deserializer)?;
        VerifyingKey::from_bytes(&bytes)
            .map(PublicKey)
            .map_err(serde::de::Error::custom)
    }
}

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.to_bytes().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes: Vec<u8> = Deserialize::deserialize(deserializer)?;
        if bytes.len() != 64 {
            return Err(serde::de::Error::custom("invalid signature length"));
        }
        let mut arr = [0u8; 64];
        arr.copy_from_slice(&bytes);
        Ok(Signature(EdSignature::from_bytes(&arr)))
    }
}

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Invalid signature format")]
    InvalidSignature,
    #[error("Public key parse error")]
    KeyParseError,
    #[error("Signature verification failed")]
    VerificationFailed,
}

#[derive(Clone)]
pub struct KeyPair {
    pub signing_key: SigningKey,
    pub verifying_key: PublicKey,
}

impl KeyPair {
    pub fn public_key(&self) -> PublicKey {
        self.verifying_key.clone()
    }
}

pub fn generate_keypair() -> KeyPair {
    let mut rng = OsRng;
    let mut seed = [0u8; 32];
    rng.fill_bytes(&mut seed);
    let signing_key = SigningKey::from_bytes(&seed);
    let verifying_key = PublicKey(signing_key.verifying_key());
    KeyPair {
        signing_key,
        verifying_key,
    }
}

pub fn verify_multisig(
    signatures: &[Signature],
    participants: &[PublicKey],
    message: &[u8],
) -> Result<(), CryptoError> {
    if signatures.len() != participants.len() {
        return Err(CryptoError::VerificationFailed);
    }

    for (pk, sig) in participants.iter().zip(signatures) {
        pk.0.verify(message, &sig.0)
            .map_err(|_| CryptoError::VerificationFailed)?;
    }
    
    Ok(())
}

/// Verifies signatures from a subset of participants.
/// Each signature must correspond to a participant in the affected_participants list.
/// The signatures can be provided in any order, as long as they match the affected participants.
pub fn verify_partial_multisig(
    signatures: &[Signature],
    all_participants: &[PublicKey],
    affected_participants: &[PublicKey],
    message: &[u8],
) -> Result<(), CryptoError> {
    println!("Verifying {} signatures for {} affected participants", 
        signatures.len(), affected_participants.len());

    // Verify we have the right number of signatures
    if signatures.len() != affected_participants.len() {
        println!("Signature count mismatch: {} signatures != {} affected participants",
            signatures.len(), affected_participants.len());
        return Err(CryptoError::VerificationFailed);
    }

    // Verify that all affected participants are in the main participants list
    for affected in affected_participants {
        if !all_participants.contains(affected) {
            println!("Affected participant not found in participants list");
            return Err(CryptoError::VerificationFailed);
        }
    }

    // Create pairs of signatures and affected participants
    let mut sig_pairs: Vec<_> = signatures.iter()
        .zip(affected_participants.iter())
        .collect();

    // Sort pairs by public key bytes for consistent ordering
    sig_pairs.sort_by(|(_, pk1), (_, pk2)| pk1.0.to_bytes().cmp(&pk2.0.to_bytes()));

    // Verify each signature against its corresponding participant
    for (sig, pk) in sig_pairs {
        if pk.0.verify(message, &sig.0).is_err() {
            println!("Signature verification failed for participant");
            return Err(CryptoError::VerificationFailed);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::Signer;

    #[test]
    fn test_multisig_verification() {
        let msg = b"channel state update";
        let kp1 = generate_keypair();
        let kp2 = generate_keypair();

        let sig1 = Signature(kp1.signing_key.try_sign(msg).unwrap());
        let sig2 = Signature(kp2.signing_key.try_sign(msg).unwrap());

        let participants = vec![
            kp1.public_key(),
            kp2.public_key(),
        ];

        let signatures = vec![sig1, sig2];

        assert!(verify_multisig(&signatures[..], &participants[..], msg).is_ok());
    }
}
