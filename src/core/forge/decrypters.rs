use super::krb_cred::new_krb_cred_info;
use crate::core::Cipher;
use crate::core::TicketCredInfo;
use crate::error::Result;
use kerberos_asn1::{AsRep, Asn1Object, EncAsRepPart, EncryptedData};
use kerberos_constants;
use kerberos_constants::key_usages;
use kerberos_crypto::{new_kerberos_cipher};

pub fn extract_krb_cred_from_as_rep(
    as_rep: AsRep,
    cipher: &Cipher,
) -> Result<TicketCredInfo> {
    let raw_enc_as_rep_part =
        decrypt_as_rep_enc_part(cipher, &as_rep.enc_part)?;

    let (_, enc_as_rep_part) = EncAsRepPart::parse(&raw_enc_as_rep_part)
        .map_err(|_| format!("Error decoding AS-REP"))?;

    let krb_cred_info =
        new_krb_cred_info(enc_as_rep_part.into(), as_rep.crealm, as_rep.cname);

    return Ok(TicketCredInfo::new(as_rep.ticket, krb_cred_info));
}

/// Decrypts the AS-REP enc-part by using the use credentials
fn decrypt_as_rep_enc_part(
    cipher: &Cipher,
    enc_part: &EncryptedData,
) -> Result<Vec<u8>> {
    if cipher.etype() != enc_part.etype {
        return Err("Unable to decrypt KDC response AS-REP: mistmach etypes")?;
    }

    let raw_enc_as_req_part = cipher
        .decrypt(key_usages::KEY_USAGE_AS_REP_ENC_PART, &enc_part.cipher)
        .map_err(|error| {
            format!("Error decrypting KDC response AS-REP: {}", error)
        })?;

    return Ok(raw_enc_as_req_part);
}

/// Decrypts the TGS-REP enc-part by using the session key
pub fn decrypt_tgs_rep_enc_part(
    session_key: &[u8],
    enc_part: &EncryptedData,
) -> Result<Vec<u8>> {
    let cipher = new_kerberos_cipher(enc_part.etype)
        .map_err(|_| format!("Not supported etype: '{}'", enc_part.etype))?;

    let raw_enc_as_req_part = cipher
        .decrypt(
            session_key,
            key_usages::KEY_USAGE_TGS_REP_ENC_PART_SESSION_KEY,
            &enc_part.cipher,
        )
        .map_err(|error| format!("Error decrypting TGS-REP: {}", error))?;

    return Ok(raw_enc_as_req_part);
}
