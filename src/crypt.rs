//! encrpyt/decrypt text
use rand::thread_rng;
use rand::RngCore;

use anyhow::{ensure, Result};
use crypto::aead::AeadDecryptor;
use crypto::aead::AeadEncryptor;
use crypto::chacha20poly1305::ChaCha20Poly1305;
use lazy_static::lazy_static;
use scrypt::{scrypt, ScryptParams};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct EncryptedSerializable {
  salt: String,
  nonce: String,
  ciphertext: String,
  tag: String,
}

fn unbase64(s: &str) -> Result<Vec<u8>> {
  Ok(base64::decode_config(s, base64::STANDARD_NO_PAD)?)
}

fn base64(bytes: &[u8]) -> String {
  base64::encode_config(bytes, base64::STANDARD_NO_PAD)
}

pub struct Encrypted {
  salt: Vec<u8>,
  nonce: Vec<u8>,
  ciphertext: Vec<u8>,
  tag: Vec<u8>,
}

impl Encrypted {
  pub fn to_json(&self) -> Result<String> {
    let jsonable = EncryptedSerializable {
      salt: base64(&self.salt),
      nonce: base64(&self.nonce),
      ciphertext: base64(&self.ciphertext),
      tag: base64(&self.tag),
    };
    Ok(serde_json::to_string_pretty(&jsonable)?)
  }

  pub fn from_json(json: &str) -> Result<Self> {
    let as_json: EncryptedSerializable = serde_json::from_str(json)?;
    Ok(Encrypted {
      salt: unbase64(&as_json.salt)?,
      nonce: unbase64(&as_json.nonce)?,
      ciphertext: unbase64(&as_json.ciphertext)?,
      tag: unbase64(&as_json.tag)?,
    })
  }
}

lazy_static! {
  static ref SCRYPT_PARAMS: ScryptParams = ScryptParams::new(14, 8, 1).unwrap();
}

pub fn encrypt(plaintext: &[u8], password: &str) -> Result<Encrypted> {
  let mut salt = vec![0; 32];
  thread_rng().fill_bytes(&mut salt);

  let mut key = vec![0; 32];

  scrypt(password.as_bytes(), &salt, &SCRYPT_PARAMS, &mut key)?;

  let mut nonce = vec![0; 8];
  thread_rng().fill_bytes(&mut nonce);

  let mut ciphertext = vec![0; plaintext.len()];
  let aad = vec![];
  let mut tag = vec![0; 16];
  ChaCha20Poly1305::new(&key, &nonce, &aad).encrypt(
    plaintext,
    &mut ciphertext,
    &mut tag,
  );

  Ok(Encrypted {
    salt: salt,
    nonce: nonce,
    ciphertext: ciphertext,
    tag: tag,
  })
}

pub fn decrypt(encrypted: Encrypted, password: &str) -> Result<Vec<u8>> {
  let mut key = vec![0; 32];
  scrypt(
    password.as_bytes(),
    &encrypted.salt,
    &SCRYPT_PARAMS,
    &mut key,
  )?;

  let mut plaintext = vec![0; encrypted.ciphertext.len()];
  let decrypt_succeeded = ChaCha20Poly1305::new(&key, &encrypted.nonce, &[])
    .decrypt(&encrypted.ciphertext, &mut plaintext, &encrypted.tag);
  ensure!(
    decrypt_succeeded,
    "could not decrypt contents (is your password correct?)"
  );
  Ok(plaintext)
}
