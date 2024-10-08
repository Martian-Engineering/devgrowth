use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    name: String,
    email: String,
    exp: usize,
    iat: usize,
    id: i64,
    access_token: String,
}

unsafe impl Send for Claims {}

pub fn validate_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let decoding_key = DecodingKey::from_secret(std::env::var("JWT_SECRET").unwrap().as_bytes());
    let validation = Validation::new(Algorithm::HS256);

    println!("About to decode token");
    match decode::<Claims>(token, &decoding_key, &validation) {
        Ok(token_data) => {
            println!("Header: {:?}", token_data.header);
            println!("Claims: {:?}", token_data.claims);
            println!("Token decoded successfully");
            println!("Token data: {:?}", token_data);

            // Check if the token has expired
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as usize;
            if token_data.claims.exp < now {
                Err(jsonwebtoken::errors::Error::from(
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature,
                ))
            } else {
                Ok(token_data.claims)
            }
        }
        Err(e) => {
            println!("Error decoding token: {:?}", e);
            Err(e)
        }
    }
}
