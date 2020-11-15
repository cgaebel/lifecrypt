//! encrpyt/decrypt text
use rand::thread_rng;
use rand::RngCore;

use anyhow::{ensure, Result};
use crypto::aead::AeadDecryptor;
use crypto::aead::AeadEncryptor;
use crypto::chacha20poly1305;
use scrypt::{scrypt, ScryptParams};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Encrypted {
  salt: Vec<u8>,
  nonce: Vec<u8>,
  ciphertext: Vec<u8>,
  tag: Vec<u8>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct EncryptedSerializable {
  salt: String,
  nonce: String,
  ciphertext: String,
  tag: String,
}

fn unbase64(s: &str) -> Result<Vec<u8>> {
  let r = base64::decode_config(s, base64::STANDARD_NO_PAD)?;
  Ok(r)
}

fn base64(bytes: &[u8]) -> String {
  base64::encode_config(bytes, base64::STANDARD_NO_PAD)
}

impl EncryptedSerializable {
  pub fn new(e: &Encrypted) -> Self {
    EncryptedSerializable {
      salt: base64(&e.salt),
      nonce: base64(&e.nonce),
      ciphertext: base64(&e.ciphertext),
      tag: base64(&e.tag),
    }
  }

  pub fn to_encrypted(&self) -> Result<Encrypted> {
    Ok(Encrypted {
      salt: unbase64(&self.salt)?,
      nonce: unbase64(&self.nonce)?,
      ciphertext: unbase64(&self.ciphertext)?,
      tag: unbase64(&self.tag)?,
    })
  }
}

const SCRYPT_LOG_N: u8 = 20;
const SCRYPT_R: u32 = 8;
const SCRYPT_P: u32 = 1;

pub fn encrypt(plaintext: &[u8], password: &str) -> Result<Encrypted> {
  let mut salt = vec![0; 32];
  thread_rng().fill_bytes(&mut salt);

  let mut key = vec![0; 32];

  let params = ScryptParams::new(SCRYPT_LOG_N, SCRYPT_R, SCRYPT_P)?;
  scrypt(password.as_bytes(), &salt, &params, &mut key)?;

  let mut nonce = vec![0; 8];
  thread_rng().fill_bytes(&mut nonce);

  let mut ciphertext = vec![0; plaintext.len()];
  let aad = vec![];
  let mut tag = vec![0; 16];
  let mut cha = chacha20poly1305::ChaCha20Poly1305::new(&key, &nonce, &aad);
  cha.encrypt(plaintext, &mut ciphertext, &mut tag);

  return Ok(Encrypted {
    salt: salt,
    nonce: nonce,
    ciphertext: ciphertext,
    tag: tag,
  });
}

pub fn decrypt(encrypted: Encrypted, password: &str) -> Result<Vec<u8>> {
  let mut key = vec![0; 32];
  let params = ScryptParams::new(SCRYPT_LOG_N, SCRYPT_R, SCRYPT_P)?;
  scrypt(password.as_bytes(), &encrypted.salt, &params, &mut key)?;

  let aad = vec![];
  let mut chad =
    chacha20poly1305::ChaCha20Poly1305::new(&key, &encrypted.nonce, &aad);
  let mut plaintext = vec![0; encrypted.ciphertext.len()];

  let decrypt_succeeded =
    chad.decrypt(&encrypted.ciphertext, &mut plaintext, &encrypted.tag);
  ensure!(decrypt_succeeded, "could not decrypt contents (is your password correct?)");
  return Ok(plaintext);
}
