pub struct ArchiveEntry {
    pub name: String,
    pub data: Vec<u8>,
}

pub trait Compressor {
    fn compress(&self, entries: &[ArchiveEntry], password: Option<&str>) -> Result<Vec<u8>, String>;
    fn decompress(&self, archive: &[u8], password: Option<&str>) -> Result<Vec<ArchiveEntry>, String>;
}

pub enum GpuBackend {
    Cuda,
    Vulkan,
    None,
}

pub trait GpuAccelerator {
    fn name(&self) -> &str;
    fn run_kernel(&self, name: &str, data: &mut [u8]) -> Result<(), String>;
    fn mix_probabilities(&self, model_probs: &[f32], weights: &[f32], num_bits: usize) -> Result<Vec<f32>, String>;
}

pub struct CompressionOptions {
    pub level: u32,
    pub backend: GpuBackend,
    pub password: Option<String>,
}

pub mod crypto {
    use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit, Tag};
    use aes_gcm::aead::AeadInPlace;
    use pbkdf2::pbkdf2_hmac;
    use sha2::Sha256;
    use rand::Rng;

    pub fn encrypt(data: &[u8], password: &str) -> Result<Vec<u8>, String> {
        let mut rng = rand::thread_rng();
        let mut salt = [0u8; 16];
        rng.fill(&mut salt);

        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, 100_000, &mut key);

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
        let mut nonce = [0u8; 12];
        rng.fill(&mut nonce);

        // Optimization: Pre-allocate result buffer and encrypt in-place to avoid
        // redundant allocations and copies.
        // Format: [16-byte salt][12-byte nonce][ciphertext][16-byte tag]
        let mut result = Vec::with_capacity(16 + 12 + data.len() + 16);
        result.extend_from_slice(&salt);
        result.extend_from_slice(&nonce);
        result.extend_from_slice(data);

        // Encrypt the data part in-place (starts at index 28)
        let tag = cipher.encrypt_in_place_detached(Nonce::from_slice(&nonce), b"", &mut result[28..])
            .map_err(|e| e.to_string())?;

        // Append the authentication tag
        result.extend_from_slice(tag.as_slice());
        Ok(result)
    }

    pub fn decrypt(data: &[u8], password: &str) -> Result<Vec<u8>, String> {
        // Optimization: Fail fast if data is too short to contain salt, nonce, and tag.
        // 16 (salt) + 12 (nonce) + 16 (tag) = 44 bytes
        if data.len() < 44 {
            return Err("Invalid encrypted data: too short".to_string());
        }

        let salt = &data[0..16];
        let nonce = &data[16..28];
        let ciphertext = &data[28..];

        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, 100_000, &mut key);

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));

        // Optimization: Avoid redundant allocations and copying by decrypting in-place.
        // cipher.decrypt() would allocate a new Vec and copy the ciphertext+tag into it.
        // Format: [ciphertext][16-byte tag]
        let tag_pos = ciphertext.len() - 16;
        let mut buffer = ciphertext[..tag_pos].to_vec();
        let tag = Tag::from_slice(&ciphertext[tag_pos..]);

        cipher.decrypt_in_place_detached(Nonce::from_slice(nonce), b"", &mut buffer, tag)
            .map_err(|e| e.to_string())?;

        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::crypto;

    #[test]
    fn test_encryption_decryption() {
        let password = "super_secret_password";
        let data = b"Hello, GPU-accelerated world!";

        let encrypted = crypto::encrypt(data, password).expect("Encryption failed");
        let decrypted = crypto::decrypt(&encrypted, password).expect("Decryption failed");

        assert_eq!(data.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_wrong_password() {
        let password = "correct_password";
        let wrong_password = "wrong_password";
        let data = b"Secret data";

        let encrypted = crypto::encrypt(data, password).expect("Encryption failed");
        let result = crypto::decrypt(&encrypted, wrong_password);

        assert!(result.is_err());
    }
}
