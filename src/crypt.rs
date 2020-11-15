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

impl EncryptedSerializable {
  pub fn new(e: &Encrypted) -> Self {
    EncryptedSerializable {
      salt: base64::encode(&e.salt),
      nonce: base64::encode(&e.nonce),
      ciphertext: base64::encode(&e.ciphertext),
      tag: base64::encode(&e.tag),
    }
  }

  pub fn to_encrypted(&self) -> Result<Encrypted> {
    Ok(Encrypted {
      salt: base64::decode(&self.salt)?,
      nonce: base64::decode(&self.nonce)?,
      ciphertext: base64::decode(&self.ciphertext)?,
      tag: base64::decode(&self.tag)?,
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

  let params = ScryptParams::new(SCRYPT_LOG_N, SCRYPT_R, SCRYPT_P)
    .expect("scrpyt params to be created");
  scrypt(password.as_bytes(), &salt, &params, &mut key)
    .expect("scrypt should not fail");

  let mut nonce = vec![0; 8];
  thread_rng().fill_bytes(&mut nonce);

  let mut ciphertext = vec![0; plaintext.len()];
  let aad = vec![0; 0];
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
  let params = ScryptParams::new(SCRYPT_LOG_N, SCRYPT_R, SCRYPT_P)
    .expect("scrpyt params to be created");
  scrypt(password.as_bytes(), &encrypted.salt, &params, &mut key)
    .expect("scrypt should not fail");

  let aad = vec![0; 0];
  let mut chad =
    chacha20poly1305::ChaCha20Poly1305::new(&key, &encrypted.nonce, &aad);
  let mut plaintext = vec![0; encrypted.ciphertext.len()];

  let decrypt_succeeded = chad.decrypt(&encrypted.ciphertext, &mut plaintext, &encrypted.tag);
  ensure!(decrypt_succeeded, "could not decrypt contents");
  return Ok(plaintext);
}
