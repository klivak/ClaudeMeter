use windows::core::PCWSTR;
use windows::Win32::Security::Credentials::{CredFree, CredReadW, CREDENTIALW, CRED_TYPE_GENERIC};

#[derive(Debug, Clone)]
pub struct CredentialInfo {
    pub access_token: String,
    /// e.g. "max", "pro" — from credentials file, if available
    pub subscription_type: Option<String>,
    /// e.g. "default_claude_max_5x" — rate limit tier from credentials
    pub rate_limit_tier: Option<String>,
}

#[derive(Debug)]
pub enum CredentialError {
    NotFound,
    WindowsError(String),
    ParseError(String),
}

impl std::fmt::Display for CredentialError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(
                f,
                "Claude Code credentials not found in Windows Credential Manager"
            ),
            Self::WindowsError(e) => write!(f, "Windows API error: {e}"),
            Self::ParseError(e) => write!(f, "Failed to parse credential JSON: {e}"),
        }
    }
}

/// Read the Claude OAuth access token.
///
/// Checks two locations in order:
/// 1. File-based: `~/.claude/.credentials.json` (Claude Code v2.x)
/// 2. Windows Credential Manager: target "Claude Code-credentials" (legacy)
pub fn read_claude_token() -> Result<CredentialInfo, CredentialError> {
    // Try file-based credentials first (Claude Code v2.x stores here)
    if let Ok(info) = read_token_from_file() {
        return Ok(info);
    }

    // Fall back to Windows Credential Manager
    read_token_from_credential_manager()
}

/// Read token from ~/.claude/.credentials.json
fn read_token_from_file() -> Result<CredentialInfo, CredentialError> {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map_err(|_| CredentialError::NotFound)?;

    let cred_path = std::path::Path::new(&home)
        .join(".claude")
        .join(".credentials.json");

    let contents = std::fs::read_to_string(&cred_path).map_err(|_| CredentialError::NotFound)?;

    extract_credential_info(&contents)
}

/// Read token from Windows Credential Manager (legacy path)
fn read_token_from_credential_manager() -> Result<CredentialInfo, CredentialError> {
    const TARGET: &str = "Claude Code-credentials";

    let target_wide: Vec<u16> = TARGET.encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        let mut pcredential: *mut CREDENTIALW = std::ptr::null_mut();
        let result = CredReadW(
            PCWSTR(target_wide.as_ptr()),
            CRED_TYPE_GENERIC,
            0,
            &mut pcredential,
        );

        if result.is_err() {
            let err = windows::core::Error::from_win32();
            if err.code().0 == 0x80070490u32 as i32 {
                return Err(CredentialError::NotFound);
            }
            return Err(CredentialError::WindowsError(err.to_string()));
        }

        if pcredential.is_null() {
            return Err(CredentialError::NotFound);
        }

        let cred = &*pcredential;
        let blob_size = cred.CredentialBlobSize as usize;
        let blob_ptr = cred.CredentialBlob;

        if blob_ptr.is_null() || blob_size == 0 {
            CredFree(pcredential as *mut _);
            return Err(CredentialError::ParseError(
                "Empty credential blob".to_string(),
            ));
        }

        let json_string = if blob_size % 2 == 0 {
            let wide_len = blob_size / 2;
            let wide_slice = std::slice::from_raw_parts(blob_ptr as *const u16, wide_len);
            let wide_slice = if wide_slice.last() == Some(&0) {
                &wide_slice[..wide_len - 1]
            } else {
                wide_slice
            };
            match String::from_utf16(wide_slice) {
                Ok(s) if s.starts_with('{') => s,
                _ => {
                    let bytes = std::slice::from_raw_parts(blob_ptr, blob_size);
                    String::from_utf8_lossy(bytes)
                        .trim_end_matches('\0')
                        .to_string()
                }
            }
        } else {
            let bytes = std::slice::from_raw_parts(blob_ptr, blob_size);
            String::from_utf8_lossy(bytes)
                .trim_end_matches('\0')
                .to_string()
        };

        CredFree(pcredential as *mut _);

        extract_credential_info(&json_string)
    }
}

fn extract_credential_info(json: &str) -> Result<CredentialInfo, CredentialError> {
    let v: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| CredentialError::ParseError(format!("Invalid JSON: {e}")))?;

    let oauth = v.get("claudeAiOauth");

    // Try nested: {"claudeAiOauth": {"accessToken": "..."}}
    if let Some(token) = oauth
        .and_then(|o| o.get("accessToken"))
        .and_then(|t| t.as_str())
    {
        let subscription_type = oauth
            .and_then(|o| o.get("subscriptionType"))
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());
        let rate_limit_tier = oauth
            .and_then(|o| o.get("rateLimitTier"))
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());
        return Ok(CredentialInfo {
            access_token: token.to_string(),
            subscription_type,
            rate_limit_tier,
        });
    }

    // Try flat: {"accessToken": "..."}
    if let Some(token) = v.get("accessToken").and_then(|t| t.as_str()) {
        let subscription_type = v
            .get("subscriptionType")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());
        let rate_limit_tier = v
            .get("rateLimitTier")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());
        return Ok(CredentialInfo {
            access_token: token.to_string(),
            subscription_type,
            rate_limit_tier,
        });
    }

    Err(CredentialError::ParseError(
        "accessToken field not found in credential JSON".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_nested() {
        let json = r#"{"claudeAiOauth": {"accessToken": "test-token-nested"}}"#;
        let info = extract_credential_info(json).unwrap();
        assert_eq!(info.access_token, "test-token-nested");
        assert_eq!(info.subscription_type, None);
    }

    #[test]
    fn test_extract_nested_with_subscription() {
        let json = r#"{"claudeAiOauth": {"accessToken": "tok", "subscriptionType": "max"}}"#;
        let info = extract_credential_info(json).unwrap();
        assert_eq!(info.access_token, "tok");
        assert_eq!(info.subscription_type, Some("max".to_string()));
    }

    #[test]
    fn test_extract_flat() {
        let json = r#"{"accessToken": "test-token-flat"}"#;
        let info = extract_credential_info(json).unwrap();
        assert_eq!(info.access_token, "test-token-flat");
    }

    #[test]
    fn test_extract_missing() {
        let json = r#"{"other": "data"}"#;
        assert!(matches!(
            extract_credential_info(json),
            Err(CredentialError::ParseError(_))
        ));
    }
}
