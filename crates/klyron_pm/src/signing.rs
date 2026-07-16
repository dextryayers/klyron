use ed25519_dalek::{SigningKey as DalekSigningKey, VerifyingKey as DalekVerifyingKey};
use ed25519_dalek::{Signature, Signer, Verifier};
use rand::rngs::OsRng;
use rand::RngCore;
use std::path::Path;

pub struct SigningKey(pub DalekSigningKey);

pub struct VerifyKey(pub DalekVerifyingKey);

pub fn generate_keypair() -> (Vec<u8>, Vec<u8>) {
    let mut secret = [0u8; 32];
    OsRng.fill_bytes(&mut secret);
    let signing_key = DalekSigningKey::from_bytes(&secret);
    let verifying_key = signing_key.verifying_key();
    (signing_key.to_bytes().to_vec(), verifying_key.to_bytes().to_vec())
}

pub fn sign(data: &[u8], secret: &[u8]) -> Vec<u8> {
    let secret_bytes: [u8; 32] = match secret.try_into() {
        Ok(b) => b,
        Err(_) => return Vec::new(),
    };
    let signing_key = DalekSigningKey::from_bytes(&secret_bytes);
    let signature = signing_key.sign(data);
    signature.to_bytes().to_vec()
}

pub fn verify(data: &[u8], signature: &[u8], public: &[u8]) -> bool {
    if signature.len() != 64 {
        return false;
    }
    let pub_bytes: [u8; 32] = match public.try_into() {
        Ok(b) => b,
        Err(_) => return false,
    };
    let verifying_key = match DalekVerifyingKey::from_bytes(&pub_bytes) {
        Ok(k) => k,
        Err(_) => return false,
    };
    let mut sig_bytes = [0u8; 64];
    sig_bytes.copy_from_slice(signature);
    let sig = Signature::from_bytes(&sig_bytes);
    verifying_key.verify(data, &sig).is_ok()
}

pub fn keypair_to_files(pub_path: &Path, sec_path: &Path) {
    let mut secret = [0u8; 32];
    OsRng.fill_bytes(&mut secret);
    let signing_key = DalekSigningKey::from_bytes(&secret);
    let verifying_key = signing_key.verifying_key();

    if let Some(parent) = pub_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Some(parent) = sec_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let _ = std::fs::write(pub_path, verifying_key.to_bytes());
    let _ = std::fs::write(sec_path, signing_key.to_bytes());
}

pub fn keypair_from_files(_pub_path: &Path, sec_path: &Path) -> Option<(Vec<u8>, Vec<u8>)> {
    let sec_bytes = std::fs::read(sec_path).ok()?;
    let sec_arr: [u8; 32] = sec_bytes.as_slice().try_into().ok()?;
    let signing_key = DalekSigningKey::from_bytes(&sec_arr);
    let verifying_key = signing_key.verifying_key();
    Some((signing_key.to_bytes().to_vec(), verifying_key.to_bytes().to_vec()))
}

pub fn save_keypair(name: &str, secret: &[u8], public: &[u8]) -> Result<(), String> {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("~/.config"))
        .join("klyron")
        .join("keys");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    std::fs::write(dir.join(format!("{name}.sec")), secret).map_err(|e| e.to_string())?;
    std::fs::write(dir.join(format!("{name}.pub")), public).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn load_keypair(name: &str) -> Option<(Vec<u8>, Vec<u8>)> {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("~/.config"))
        .join("klyron")
        .join("keys");
    let pub_path = dir.join(format!("{name}.pub"));
    let sec_path = dir.join(format!("{name}.sec"));
    keypair_from_files(&pub_path, &sec_path)
}
