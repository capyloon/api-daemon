pub mod deviceinfo;
pub mod downloader;

// Provide the implementations to get_access_token and get_hawk_header.
use crate::deviceinfo::{get_char_pref, DeviceInfo};
use hawk::{Credentials, Key, PayloadHasher, RequestBuilder, SHA256};
use log::{debug, error};
use reqwest::header::AUTHORIZATION;
use reqwest::StatusCode;
use serde::Deserialize;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub enum Method {
    GET,
    POST,
}

// The server returns a response such as:
// {
//     "access_token":"eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJhaWQiOiJaVzhzdkdTbGF3MVpMQ3hXWlBRQSIsImFvayI6dHJ1ZSwiYXRoIjoiaGF3ayIsImF1ZCI6ImFwaS5zdGFnZS5rYWlvc3RlY2guY29tIiwiYnJhbmQiOiJTUFJEIiwiY3UiOiI0MDQ0Ty0yQUFRVVMwIiwiZGlkIjoiNU9tX1pVR2xCZ2lzMzRiNEFwdWwiLCJkdHlwZSI6IjEwMDAiLCJlb2siOmZhbHNlLCJleHAiOjE1NTMxODkwNDksImZuIjoiNU9tX1pVR2xCZ2lzMzRiNEFwdWwiLCJpYXQiOjE1NTI1ODQyNDksImljY19tY2MiOjAsImljY19tY2MyIjowLCJpY2NfbW5jIjowLCJpY2NfbW5jMiI6MCwiaXNzIjoiYXV0aC5zdGFnZS5rYWlvc3RlY2guY29tIiwianRpIjoibkZVenVRNFRheTFlaHA5ZlI3M0siLCJsb2ciOiI1T21fWlVHbEJnaXMzNGI0QXB1bCIsIm1vZGVsIjoic3A3NzMxZWZfMThjMTBfbmF0aXZlIiwibW9rIjpmYWxzZSwibmV0X21jYyI6MCwibmV0X21jYzIiOjAsIm5ldF9tbmMiOjAsIm5ldF9tbmMyIjowLCJvcyI6IkthaU9TIiwib3N2IjoiMi41Iiwic2NwIjoidXxjb3JlOmNydWRzIHNjI2FwcHM6cnMgc2MjbWV0cmljczpjIiwidHRsIjo2MDQ4MDAsInR5cCI6ImFjY2Vzc190b2tlbiIsInVpZCI6Im5vbmUifQ.lDX3O4jzrryeO97putfImh2XP-BPuxt107JwfJSOtROGiZw0mxdXJb9O0soOeAFgWYTPWfefESNvf6o9snQW7aoTgpObAkofdlAZBUgk2-GdYK24J3XSPTilRr19tQrjB9c2SNC0GTepZnttuyqhXJOF2dCpbM2ipUzT6WNzm9YHb4bDUcNAt7z_RnyuV0Vv1UMNMSqYy-5VHLV1ZxpdKZOnO5fJ4LGqH2VLEcYxTGT-NB3tHyBFJOMnab_CYeQqYd9RGpfifUU4sXs49QhvMZnXVqaIBQH922ZkQ_sExk8YjqqygbIkUii68Uo-8DjfxH7yjUHRUB-ZwKSQB19x1g",
//     "token_type":"hawk",
//     "scope":"u|core:cruds sc#apps:rs sc#metrics:c",
//     "expires_in":604800,
//     "kid":"0OaiTEPTEsL1I1C/wvlMCj/x8qc=",
//     "mac_key":"YwYvMXDCkOAswfx9ptDllvQveZTrSc7b2FIQ+gpXsHE=",
//     "mac_algorithm":"hmac-sha-256"
// }

#[derive(Deserialize, Debug, Clone)]
pub struct AccessTokenInfo {
    pub expires_in: u64,
    pub kid: String,
    pub mac_key: String,
}

#[derive(Debug, Default, Clone)]
pub struct ServerInfo {
    pub token_uri: String, // from config file and composed with TOKEN_ID
    pub api_key: String,   // TOKEN_KEY from build time
    pub api_uri: String,   // from config
}

impl ServerInfo {
    pub fn try_get_token_uri(&mut self) {
        if let Some(uri) = get_char_pref("service.token.uri") {
            self.token_uri = uri;
        }
    }
}

#[derive(Debug, Clone)]
pub struct Hawk {
    pub token_info: Option<AccessTokenInfo>,
    pub valid_until: Instant,
    pub is_external: bool,     //Denotes if the token_info is set by external user
}

impl Default for Hawk {
    fn default() -> Self {
        Self {
            token_info: None,
            valid_until: Instant::now(),
            is_external: false,
        }
    }
}

impl Hawk {
    // Checks that we have a token and if so whether it has expired or not.
    pub fn has_valid_token(&self) -> bool {
        debug!(
            "has_valid_token server_info={:?} valid_until={:?}",
            self.token_info, self.valid_until
        );
        match self.token_info {
            None => false,
            Some(_) => self.valid_until > Instant::now(),
        }
    }

    // Retrieves the access token for this device.
    // This function never fails but updates the internal state.
    pub fn get_access_token(&mut self, device_info: &DeviceInfo, server_info: &ServerInfo) {
        match reqwest::blocking::Client::builder()
            .timeout(Some(Duration::from_secs(90))) // 1m30s of timeout instead of the default 30s
            .build()
            .unwrap()
            .post(&server_info.token_uri)
            .header(AUTHORIZATION, format!("Key {}", server_info.api_key))
            .json(device_info)
            .send()
        {
            Ok(response) => {
                let status = response.status();
                if status != StatusCode::OK && status != StatusCode::CREATED {
                    error!("Request failed: {}", status);
                    self.token_info = None;
                    return;
                }

                match response.json::<AccessTokenInfo>() {
                    Ok(info) => {
                        // Evaluate how long this token is valid, but reduce the validity by 5 minutes to avoid
                        // races between the server and client.
                        self.valid_until =
                            Instant::now() + Duration::from_secs(info.expires_in - 60 * 5);
                        self.token_info = Some(info);
                        self.is_external = false;
                    }
                    Err(err) => {
                        error!("Failed to decode json access token: {}", err);
                        self.token_info = None;
                    }
                }
            }
            Err(err) => {
                error!("Failed to retrieve access token: {}", err);
                self.token_info = None;
            }
        }
    }

    // Creates a Hawk header for the current state, if possible.
    pub fn get_hawk_header(
        &self,
        method: Method,
        server_info: &ServerInfo,
        payload: Option<&str>,
    ) -> Option<String> {
        if !self.has_valid_token() {
            return None;
        }

        let method_str = match method {
            Method::GET => "GET",
            Method::POST => "POST",
        };

        if let Some(token_info) = &self.token_info {
            let key = base64::decode(&token_info.mac_key);
            if key.is_err() {
                error!("Failed to decode base64 key: {}", key.err().unwrap());
                return None;
            }
            let url = url::Url::parse(&server_info.api_uri);
            if url.is_err() {
                error!("Failed to parse metrics api url: {}", url.err().unwrap());
                return None;
            }

            let key = Key::new(key.unwrap(), SHA256);
            if key.is_err() {
                error!("Failed to create key: {}", key.err().unwrap());
                return None;
            }

            let credentials = Credentials {
                id: token_info.kid.clone(),
                key: key.unwrap(),
            };

            let mut payload_hash: Vec<u8> = Vec::new();
            if let Some(value) = payload {
                let hash = PayloadHasher::hash("application/json", SHA256, value);
                if hash.is_err() {
                    error!("Failed to hash payload: {}", hash.err().unwrap());
                    return None;
                }
                payload_hash = hash.unwrap();
            };

            // provide the details of the request to be authorized
            match RequestBuilder::from_url(method_str, &url.unwrap()) {
                Ok(builder) => {
                    let request = if payload_hash.is_empty() {
                        builder.hash(&payload_hash[..]).request()
                    } else {
                        builder.request()
                    };

                    // Get the resulting header, including the calculated MAC; this involves a random
                    // nonce, so the MAC will be different on every request.
                    match request.make_header(&credentials) {
                        Ok(header) => {
                            return Some(header.to_string());
                        }
                        Err(err) => {
                            error!("Failed to create Hawk header: {}", err);
                            return None;
                        }
                    }
                }
                Err(err) => {
                    error!("Failed to build Hawk request: {}", err);
                }
            }
        }
        None
    }
}

#[test]
fn valid_token() {
    let device_info = DeviceInfo {
        device_id: "a1f6c8f8-00ea-41bf-bec7-3eb93b61b076".into(),
        device_type: 1000,
        reference: "4044O-2AAQUS0".into(),
        imei: String::new(),
        model: "sp7731ef_18c10_native".into(),
        brand: "SPRD".into(),
        os: "KaiOS".into(),
        os_version: "2.5".into(),
        release_tag: "1.0a".into(),
    };
    let server_info = ServerInfo {
        token_uri: "https://api.kaiostech.com/v3.0/applications/ZW8svGSlaw1ZLCxWZPQA/tokens".into(),
        api_key: "zaP09k7OsOjXEulzSXsd".into(),
        api_uri: "https://api.kaiostech.com/v3.0/apps/metrics".into(),
    };

    let mut hawk = Hawk::default();
    hawk.get_access_token(&device_info, &server_info);
    assert!(hawk.has_valid_token());

    let header = hawk.get_hawk_header(Method::POST, &server_info, Some("Hello"));
    // We can't test the header value since a new nonce if created each time.
    assert!(header.is_some());
}

#[test]
fn invalid_token() {
    let device_info = DeviceInfo {
        device_id: "a1f6c8f8-00ea-41bf-bec7-3eb93b61b076".into(),
        device_type: 1000,
        reference: "4044O-2AAQUS0".into(),
        imei: String::new(),
        model: "sp7731ef_18c10_native".into(),
        brand: "SPRD".into(),
        os: "KaiOS".into(),
        os_version: "2.5".into(),
        release_tag: "1.0a".into(),
    };
    let server_info = ServerInfo {
        token_uri: "https://api.kaiostech.com/v3.0/applications/ZW8svGSlaw1ZLCxWZPQA/tokens".into(),
        api_key: "dummy_key".into(),
        api_uri: "https://api.kaiostech.com/v3.0/apps/metrics".into(),
    };

    let mut hawk = Hawk::default();
    hawk.get_access_token(&device_info, &server_info);
    assert!(!hawk.has_valid_token());
}
