use chacha20poly1305::{AeadInPlace, ChaCha20Poly1305, KeyInit, Nonce};
use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret};
use rand::thread_rng;
use hkdf::Hkdf;
use sha2::Sha256;

pub struct EphemeralKeys {
    pub secret: EphemeralSecret,
    pub public: PublicKey,
}

impl EphemeralKeys {
    pub fn generate() -> Self {
        let mut rng = thread_rng();
        let secret = EphemeralSecret::random_from_rng(&mut rng);
        let public = PublicKey::from(&secret);
        EphemeralKeys { secret, public }
    }

    pub fn diffie_hellman(self, remote_public: &PublicKey) -> SharedSecret {
        self.secret.diffie_hellman(remote_public)
    }
}

pub fn derive_session_key(secret: SharedSecret) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(Some(b"penum-v1"), secret.as_bytes());
    let mut okm = [0u8; 32];
    hk.expand(&[], &mut okm).expect("HKDF expand failed");
    okm
}

pub fn encrypt_in_place(
    key: &[u8; 32],
    aad: &[u8],
    buffer: &mut [u8],
) -> anyhow::Result<[u8; 16]> {
    let cipher = ChaCha20Poly1305::new(key.into());
    let nonce = Nonce::from_slice(&[0u8; 12]);
    let tag = cipher
        .encrypt_in_place_detached(nonce, aad, buffer)
        .map_err(|_| anyhow::anyhow!("Encryption failed"))?;
    Ok(tag.into())
}

pub fn decrypt_in_place(
    key: &[u8; 32],
    aad: &[u8],
    buffer: &mut [u8],
    tag: &[u8; 16],
) -> anyhow::Result<()> {
    let cipher = ChaCha20Poly1305::new(key.into());
    let nonce = Nonce::from_slice(&[0u8; 12]);
    cipher
        .decrypt_in_place_detached(nonce, aad, buffer, tag.into())
        .map_err(|_| anyhow::anyhow!("Decryption failed"))?;
    Ok(())
}
