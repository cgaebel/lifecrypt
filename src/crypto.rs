//! encrpyt/decrypt text
use rand::thread_rng;
use rand::RngCore;

use crypto::aead::AeadDecryptor;
use crypto::aead::AeadEncryptor;
use crypto::chacha20poly1305;
use scrypt::{scrypt, ScryptParams};

#[derive(Debug)]
pub struct Encrypted {
  salt: Vec<u8>,
  nonce: Vec<u8>,
  ciphertext: Vec<u8>,
  tag: Vec<u8>,
}

// TODO this panics on errors, make it return Result?
pub fn encrypt(plaintext: &str, password: &str) -> Encrypted {
  let mut salt = vec![0; 32];
  thread_rng().fill_bytes(&mut salt);

  let mut key = vec![0; 32];

  let params = ScryptParams::recommended(); // TODO use the params from the readme
  scrypt(password.as_bytes(), &salt, &params, &mut key)
    .expect("scrypt should not fail");

  let mut nonce = vec![0; 8];
  thread_rng().fill_bytes(&mut nonce);

  let mut ciphertext = vec![0; plaintext.as_bytes().len()];
  let aad = vec![0; 0];
  let mut tag = vec![0; 16];
  let mut cha = chacha20poly1305::ChaCha20Poly1305::new(&key, &nonce, &aad);
  cha.encrypt(plaintext.as_bytes(), &mut ciphertext, &mut tag);

  return Encrypted {
    salt: salt,
    nonce: nonce,
    ciphertext: ciphertext,
    tag: tag,
  };
}

// TODO error handling
pub fn decrypt(encrypted: Encrypted, password: &str) -> Vec<u8> {
  let mut key = vec![0; 32];
  let params = ScryptParams::recommended(); // TODO use the params from the readme
  scrypt(password.as_bytes(), &encrypted.salt, &params, &mut key)
    .expect("scrypt should not fail");

  let aad = vec![0; 0];
  let mut chad =
    chacha20poly1305::ChaCha20Poly1305::new(&key, &encrypted.nonce, &aad);
  let mut plaintext = vec![0; encrypted.ciphertext.len()];

  assert!(chad.decrypt(&encrypted.ciphertext, &mut plaintext, &encrypted.tag));
  return plaintext;
}
