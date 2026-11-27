use crate::common::error::AppError;

/// Memvalidasi dan menormalisasi nomor telepon mitra (Supplier & Customer).
/// Sesuai PRD Section 3.2:
/// - None atau whitespace string disimpan sebagai None (NULL).
/// - Jika terisi: panjang <= 25 karakter, hanya memuat karakter {0..9, '+', '-', ' ', '(', ')'},
///   serta jumlah digit murni antara 6 sampai 20 digit.
pub fn validate_and_normalize_phone(phone: Option<&str>) -> Result<Option<String>, AppError> {
    let p = match phone {
        Some(val) => {
            let trimmed = val.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            trimmed
        }
        None => return Ok(None),
    };

    if p.len() > 25 {
        return Err(AppError::BadRequest(
            "Nomor telepon maksimal 25 karakter".to_string(),
        ));
    }

    let mut digit_count = 0;
    for c in p.chars() {
        if c.is_ascii_digit() {
            digit_count += 1;
        } else if c != '+' && c != '-' && c != ' ' && c != '(' && c != ')' {
            return Err(AppError::BadRequest(
                "Format nomor telepon tidak valid".to_string(),
            ));
        }
    }

    if !(6..=20).contains(&digit_count) {
        return Err(AppError::BadRequest(
            "Nomor telepon harus memuat antara 6 hingga 20 digit angka".to_string(),
        ));
    }

    Ok(Some(p.to_string()))
}

/// Memvalidasi format username pengguna sesuai PRD MJ-08:
/// - Panjang 3..=50 karakter setelah .trim()
/// - Hanya memuat karakter alfanumerik, underscore (_), titik (.), atau strip (-)
/// - Dilarang mengandung spasi
pub fn validate_username(username: &str) -> Result<String, AppError> {
    let trimmed = username.trim();
    if trimmed.len() < 3 || trimmed.len() > 50 {
        return Err(AppError::BadRequest(
            "Username harus terdiri dari 3 hingga 50 karakter".to_string(),
        ));
    }

    let is_valid = trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '-');
    if !is_valid {
        return Err(AppError::BadRequest(
            "Username hanya boleh terdiri dari karakter alfanumerik, garis bawah (_), titik (.), atau tanda hubung (-), dan tidak boleh mengandung spasi".to_string(),
        ));
    }

    Ok(trimmed.to_string())
}

/// Memvalidasi input string agar tidak kosong atau hanya berisi spasi (MJ-06, MJ-07, BL-07)
pub fn validate_trimmed_non_empty(
    val: &str,
    field_name: &str,
    max_len: usize,
) -> Result<String, AppError> {
    let trimmed = val.trim();
    if trimmed.is_empty() {
        return Err(AppError::BadRequest(format!(
            "{field_name} tidak boleh kosong atau hanya berisi spasi"
        )));
    }
    if trimmed.len() > max_len {
        return Err(AppError::BadRequest(format!(
            "{field_name} melebihi batas maksimum {max_len} karakter"
        )));
    }
    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_username_validation() {
        // Valid
        assert!(validate_username("ardi").is_ok());
        assert!(validate_username("admin_123").is_ok());
        assert!(validate_username("user.name-test").is_ok());

        // Invalid: contains space
        assert!(matches!(
            validate_username("bad user"),
            Err(AppError::BadRequest(_))
        ));

        // Invalid: too short (< 3)
        assert!(matches!(
            validate_username("ab"),
            Err(AppError::BadRequest(_))
        ));

        // Invalid: whitespace only
        assert!(matches!(
            validate_username("   "),
            Err(AppError::BadRequest(_))
        ));
    }

    #[test]
    fn test_trimmed_non_empty_validation() {
        assert!(validate_trimmed_non_empty("Selada", "Nama tanaman", 100).is_ok());
        assert!(matches!(
            validate_trimmed_non_empty("   ", "Nama tanaman", 100),
            Err(AppError::BadRequest(_))
        ));
    }

    #[test]
    fn test_phone_validation_sim_2_case_a() {
        // Simulasi 2 Kasus A dari PRD Section 3.3:
        // Valid: "+62 812-3456-7890" (13 digit)
        let valid =
            validate_and_normalize_phone(Some("+62 812-3456-7890")).expect("Phone should be valid");
        assert_eq!(valid, Some("+62 812-3456-7890".to_string()));

        // Invalid: mengandung alfabet "0812-ABCD-999"
        let invalid_alpha = validate_and_normalize_phone(Some("0812-ABCD-999"));
        assert!(matches!(invalid_alpha, Err(AppError::BadRequest(_))));

        // Invalid: terlalu pendek "081" (hanya 3 digit, syarat >= 6)
        let invalid_short = validate_and_normalize_phone(Some("081"));
        assert!(matches!(invalid_short, Err(AppError::BadRequest(_))));

        // None & whitespace -> Ok(None)
        assert_eq!(validate_and_normalize_phone(None).unwrap(), None);
        assert_eq!(validate_and_normalize_phone(Some("   ")).unwrap(), None);
    }
}
