pub struct ArchiveEntry {
    pub name: String,
    pub data: Vec<u8>,
}

pub trait Compressor {
    fn compress(&self, entries: &[ArchiveEntry], password: Option<&str>)
    -> Result<Vec<u8>, String>;
    fn decompress(
        &self,
        archive: &[u8],
        password: Option<&str>,
    ) -> Result<Vec<ArchiveEntry>, String>;
}

pub enum GpuBackend {
    Cuda,
    Vulkan,
    None,
}

pub trait GpuAccelerator {
    fn name(&self) -> &str;
    fn run_kernel(&self, name: &str, data: &mut [u8]) -> Result<(), String>;
    /// Mixes probabilities from multiple models.
    ///
    /// # Layout Requirements
    /// For optimal GPU performance (coalesced memory access), both `model_probs` and `weights`
    /// must be provided in a `[num_models][num_bits]` layout (transposed).
    fn mix_probabilities(
        &self,
        model_probs: &[f32],
        weights: &[f32],
        num_bits: usize,
    ) -> Result<Vec<f32>, String>;
}

pub struct CompressionOptions {
    pub level: u32,
    pub backend: GpuBackend,
    pub password: Option<String>,
}

pub mod crypto {
    use aes_gcm::aead::{Aead, AeadInPlace};
    use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
    use pbkdf2::pbkdf2_hmac_array;
    use rand::Rng;
    use sha2::Sha256;

    pub fn encrypt(data: &[u8], password: &str) -> Result<Vec<u8>, String> {
        let mut rng = rand::thread_rng();

        // Bolt ⚡ Optimization: Pre-allocate result buffer and fill it directly with random
        // salt and nonce. This avoids a temporary stack array and an extra memcpy.
        let mut result = Vec::with_capacity(44 + data.len());
        result.resize(28, 0);
        rng.fill(&mut result[..28]);

        // Bolt ⚡ Optimization: Use pbkdf2_hmac_array for more efficient key derivation.
        // This avoids manual buffer initialization and slicing.
        let key = pbkdf2_hmac_array::<Sha256, 32>(password.as_bytes(), &result[..16], 100_000);
        let cipher = Aes256Gcm::new(&key.into());

        // Append plaintext data. Pre-allocated capacity ensures no reallocation.
        result.extend_from_slice(data);

        // Encrypt the data part in-place (starts at index 28).
        // Use split_at_mut to satisfy the borrow checker when passing both nonce and data.
        let (header, ciphertext) = result.split_at_mut(28);
        let nonce = &header[16..28];
        let tag = cipher
            .encrypt_in_place_detached(Nonce::from_slice(nonce), b"", ciphertext)
            .map_err(|e| e.to_string())?;

        // Append the authentication tag. Capacity is guaranteed to be sufficient.
        result.extend_from_slice(tag.as_slice());
        Ok(result)
    }

    pub fn decrypt(data: &[u8], password: &str) -> Result<Vec<u8>, String> {
        // Bolt ⚡ Optimization: Fail fast if data is too short to contain salt, nonce, and tag.
        // 16 (salt) + 12 (nonce) + 16 (tag) = 44 bytes
        if data.len() < 44 {
            return Err("Invalid encrypted data: too short".to_string());
        }

        // Bolt ⚡ Optimization: Consolidate slicing to reduce metadata updates.
        // Use split_at to partition the header into salt and nonce in one go.
        let (header, ciphertext_and_tag) = data.split_at(28);
        let (salt, nonce) = header.split_at(16);

        // Bolt ⚡ Optimization: Dual fail-fast check for zeroed salt or nonce with an
        // initial byte check to quickly skip non-zeroed slices (99.6% of random data).
        if (salt[0] == 0 && salt == [0u8; 16]) || (nonce[0] == 0 && nonce == [0u8; 12]) {
            return Err("Invalid encrypted data: possible zeroed or corrupted file".to_string());
        }

        // Bolt ⚡ Optimization: Use pbkdf2_hmac_array for more efficient key derivation.
        let key = pbkdf2_hmac_array::<Sha256, 32>(password.as_bytes(), salt, 100_000);
        let cipher = Aes256Gcm::new(&key.into());

        // Bolt ⚡ Optimization: Use Aead::decrypt to avoid an extra allocation and memcpy.
        // cipher.decrypt() reads directly from the ciphertext slice and writes to a new
        // plaintext Vec, saving the overhead of manually copying ciphertext into a buffer.
        let plaintext = cipher
            .decrypt(Nonce::from_slice(nonce), ciphertext_and_tag)
            .map_err(|e| e.to_string())?;

        Ok(plaintext)
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
