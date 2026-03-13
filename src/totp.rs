// TOTP generation module
// Wraps totp-lite for generating 6-digit codes with 30-second intervals

use totp_lite::{totp_custom, Sha1};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct Account {
    pub name: String,
    secret_bytes: Vec<u8>,
}

impl Account {
    pub fn new(name: String, secret: String) -> Self {
        Self { name, secret_bytes: base32_decode(&secret).unwrap_or_default() }
    }

    pub(crate) fn is_valid(&self) -> bool { !self.secret_bytes.is_empty() }

    pub fn current_code(&self) -> Result<String, String> {
        if self.secret_bytes.is_empty() { return Err("Invalid secret".to_string()); }
        let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        Ok(format!("{:06}", totp_custom::<Sha1>(30, 6, &self.secret_bytes, ts)))
    }

    pub fn time_remaining(&self) -> u32 {
        let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        (30 - ts % 30) as u32
    }
}

fn base32_decode(input: &str) -> Result<Vec<u8>, String> {
    let input = input.trim_end_matches('=');
    
    let mut result = Vec::new();
    let mut buffer: u64 = 0;
    let mut bits_in_buffer = 0;
    
    for c in input.chars() {
        let value = match c {
            'A'..='Z' => (c as u8 - b'A') as u64,
            '2'..='7' => (c as u8 - b'2' + 26) as u64,
            _ => return Err(format!("Invalid character in base32: {}", c)),
        };
        
        buffer = (buffer << 5) | value;
        bits_in_buffer += 5;
        
        if bits_in_buffer >= 8 {
            bits_in_buffer -= 8;
            result.push((buffer >> bits_in_buffer) as u8);
            buffer &= (1 << bits_in_buffer) - 1;
        }
    }
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base32_decode() {
        // Test vector from RFC 4648
        let decoded = base32_decode("JBSWY3DPEHPK3PXP").unwrap();
        assert_eq!(decoded, b"Hello!");
        
        let decoded = base32_decode("MFRGGZDFMZTWQ2LK").unwrap();
        assert_eq!(decoded, b"foobar");
    }

    #[test]
    fn test_time_remaining() {
        let account = Account::new("Test".to_string(), "JBSWY3DPEHPK3PXP".to_string());
        let remaining = account.time_remaining();
        
        // Should be between 0 and 30
        assert!(remaining <= 30);
    }

    #[test]
    fn test_current_code() {
        let account = Account::new("Test".to_string(), "JBSWY3DPEHPK3PXP".to_string());
        let code = account.current_code().unwrap();
        
        // Should be 6 digits
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
    }
}
