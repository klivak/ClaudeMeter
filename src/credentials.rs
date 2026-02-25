use windows::core::PCWSTR;
use windows::Win32::Security::Credentials::{
    CredFree, CredReadW, CREDENTIALW, CRED_TYPE_GENERIC,
};

#[derive(Debug)]
pub enum CredentialError {
    NotFound,
    WindowsError(String),
    ParseError(String),
}

impl std::fmt::Display for CredentialError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "Claude Code credentials not found in Windows Credential Manager"),
            Self::WindowsError(e) => write!(f, "Windows API error: {e}"),
            Self::ParseError(e) => write!(f, "Failed to parse credential JSON: {e}"),
        }
    }
}

/// Read the Claude OAuth access token from Windows Credential Manager.
///
/// Claude Code stores credentials under the target name "Claude Code-credentials"
/// as a JSON blob: `{"claudeAiOauth": {"accessToken": "sk-ant-oat01-..."}}`
pub fn read_claude_token() -> Result<String, CredentialError> {
    const TARGET: &str = "Claude Code-credentials";

    // Encode target as wide string
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
                // ERROR_NOT_FOUND
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
            return Err(CredentialError::ParseError("Empty credential blob".to_string()));
        }

        // The blob may be UTF-16 (Windows stores as wide) or UTF-8
        let json_string = if blob_size % 2 == 0 {
            // Try UTF-16 first
            let wide_len = blob_size / 2;
            let wide_slice = std::slice::from_raw_parts(blob_ptr as *const u16, wide_len);
            // Remove null terminator if present
            let wide_slice = if wide_slice.last() == Some(&0) {
                &wide_slice[..wide_len - 1]
            } else {
                wide_slice
            };
            match String::from_utf16(wide_slice) {
                Ok(s) if s.starts_with('{') => s,
                _ => {
                    // Fall back to UTF-8
                    let bytes = std::slice::from_raw_parts(blob_ptr, blob_size);
                    String::from_utf8_lossy(bytes).trim_end_matches('\0').to_string()
                }
            }
        } else {
            let bytes = std::slice::from_raw_parts(blob_ptr, blob_size);
            String::from_utf8_lossy(bytes).trim_end_matches('\0').to_string()
        };

        CredFree(pcredential as *mut _);

        extract_access_token(&json_string)
    }
}

fn extract_access_token(json: &str) -> Result<String, CredentialError> {
    let v: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| CredentialError::ParseError(format!("Invalid JSON: {e}")))?;

    // Try nested: {"claudeAiOauth": {"accessToken": "..."}}
    if let Some(token) = v
        .get("claudeAiOauth")
        .and_then(|o| o.get("accessToken"))
        .and_then(|t| t.as_str())
    {
        return Ok(token.to_string());
    }

    // Try flat: {"accessToken": "..."}
    if let Some(token) = v.get("accessToken").and_then(|t| t.as_str()) {
        return Ok(token.to_string());
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
        let json = r#"{"claudeAiOauth": {"accessToken": "sk-ant-oat01-test"}}"#;
        assert_eq!(
            extract_access_token(json).unwrap(),
            "sk-ant-oat01-test"
        );
    }

    #[test]
    fn test_extract_flat() {
        let json = r#"{"accessToken": "sk-ant-oat01-flat"}"#;
        assert_eq!(
            extract_access_token(json).unwrap(),
            "sk-ant-oat01-flat"
        );
    }

    #[test]
    fn test_extract_missing() {
        let json = r#"{"other": "data"}"#;
        assert!(matches!(
            extract_access_token(json),
            Err(CredentialError::ParseError(_))
        ));
    }
}
