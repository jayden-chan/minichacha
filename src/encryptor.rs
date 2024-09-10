use anyhow::Result;
use chacha20poly1305::{
    aead::{Aead, OsRng},
    AeadCore, ChaCha20Poly1305, KeyInit,
};
use pbkdf2::pbkdf2_hmac_array;
use sha2::{digest::generic_array::GenericArray, Sha256};
use std::{
    fs::File,
    io::{BufWriter, Read, Write},
};

pub fn encrypt(input: &[u8], output: File, passphrase: &str) -> Result<()> {
    let salt = ChaCha20Poly1305::generate_nonce(&mut OsRng).to_vec();
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    let key =
        pbkdf2_hmac_array::<Sha256, 32>(passphrase.as_bytes(), &salt, 600_000);
    let cipher = ChaCha20Poly1305::new(GenericArray::from_slice(&key));

    let ciphertext = cipher.encrypt(&nonce, input)?;

    let mut writer = BufWriter::new(&output);
    writer.write_all(
        format!(
            "minichacha {:03}\n",
            env!("CARGO_PKG_VERSION_MAJOR").parse::<u32>()?
        )
        .as_bytes(),
    )?;

    writer.write_all(&salt)?;
    writer.write_all(&nonce)?;
    writer.write_all(&ciphertext)?;
    writer.flush()?;

    Ok(())
}

pub fn decrypt(
    mut input: File,
    output: std::path::PathBuf,
    passphrase: &str,
) -> Result<()> {
    let mut header = [0u8; 15];
    input.read_exact(&mut header[..])?;

    if header != "minichacha 001\n".as_bytes() {
        anyhow::bail!("Incorrect minichacha header")
    }

    let mut salt = [0u8; 12];
    input.read_exact(&mut salt[..])?;

    let mut nonce = [0u8; 12];
    input.read_exact(&mut nonce[..])?;
    let nonce = GenericArray::from_slice(&nonce);

    let mut ciphertext: Vec<u8> = Vec::new();
    input.read_to_end(&mut ciphertext)?;

    let key =
        pbkdf2_hmac_array::<Sha256, 32>(passphrase.as_bytes(), &salt, 600_000);
    let cipher = ChaCha20Poly1305::new(GenericArray::from_slice(&key));

    let plaintext = cipher.decrypt(nonce, ciphertext.as_ref())?;
    std::fs::write(output, plaintext)?;

    Ok(())
}
