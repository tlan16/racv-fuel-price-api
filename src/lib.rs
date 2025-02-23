use std::error::Error;
use serde_json::Value;

#[worker::event(fetch)]
async fn fetch(
    _req: worker::HttpRequest,
    _env: worker::Env,
    _ctx: worker::Context,
) -> Result<worker::Response, worker::Error> {
    console_error_panic_hook::set_once();
    let encrypted = fetch_encrypted().await.unwrap();
    let decrypted = decrypt(&encrypted);
    Ok(
        worker::Response::builder()
            .with_header("Content-Type", "application/json")
            ?.with_status(200)
            .from_json(&decrypted)?
    )
}

pub async fn fetch_encrypted() -> Result<String, Box<dyn Error>> {
    use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, USER_AGENT};
    use reqwest::Client;
    use fake_user_agent::get_chrome_rua;

    let url = "https://www.racv.com.au/bin/racv/fuelprice.2.json";

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static(get_chrome_rua()));
    headers.insert(ACCEPT, HeaderValue::from_static("text/plain, */*; q=0.01"));
    headers.insert("Cache-Control", HeaderValue::from_static("no-cache"));
    headers.insert("Dnt", HeaderValue::from_static("1"));
    headers.insert("Pragma", HeaderValue::from_static("no-cache"));
    headers.insert("Sec-Fetch-Dest", HeaderValue::from_static("empty"));
    headers.insert("Sec-Fetch-Mode", HeaderValue::from_static("cors"));
    headers.insert("Sec-Fetch-Site", HeaderValue::from_static("same-origin"));
    headers.insert("Te", HeaderValue::from_static("trailers"));

    let client = Client::builder()
        .default_headers(headers)
        .build()?;

    let response = client.get(url).send().await?;
    // don't process response as json, as it's encrypted
    let body = response.text().await?;
    Ok(body)
}

pub fn decrypt(encrypted: &str) -> Value {
    use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit};
    use generic_array::GenericArray;

    type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;
    let secret = b"gUkXp2s5v8y/B?E(";
    // as for aes128cbc, the IV must be 16 bytes
    let iv = secret.clone();

    let encrypted_bytes = base64::decode(encrypted).expect("Failed to decode base64");

    let mut buf = encrypted_bytes.clone();
    let decrypted_bytes = Aes128CbcDec::new(
        GenericArray::from_slice(secret),
        GenericArray::from_slice(&iv)
    )
        .decrypt_padded_mut::<Pkcs7>(&mut buf)
        .expect("Decryption failed");
    let decrypted_str = std::str::from_utf8(decrypted_bytes).expect("Failed to convert to UTF-8");
    let decrypted = serde_json::from_str::<serde_json::Value>(decrypted_str).expect("Failed to parse decoded string as JSON");

    decrypted
}
