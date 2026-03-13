use std::{fs, path::Path};
use crate::totp::Account;

pub fn load_secrets<P: AsRef<Path>>(path: P) -> Result<(Vec<Account>, usize), std::io::Error> {
    let content = fs::read_to_string(path)?;
    let mut accounts = Vec::new();
    let mut invalid = 0usize;
    for line in content.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') { continue; }
        match t.split_once('=') {
            Some((name, secret)) if !name.trim().is_empty() => {
                let a = Account::new(name.trim().to_string(), secret.trim().to_string());
                if a.is_valid() { accounts.push(a); } else { invalid += 1; }
            }
            _ => invalid += 1,
        }
    }
    Ok((accounts, invalid))
}
