use jsonwebtoken::{decode, DecodingKey, Validation};
use uuid::Uuid;

use crate::models::Claims;

pub const JWT_SECRET: &str = "your-secret-key-change-this-in-production";

pub fn validate_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.as_ref()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}

pub fn extract_user_id_from_token(token: &str) -> Result<Uuid, Box<dyn std::error::Error>> {
    let claims = validate_token(token)?;
    let user_id = Uuid::parse_str(&claims.sub)?;
    Ok(user_id)
}