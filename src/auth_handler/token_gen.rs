use chrono::Utc;
use rand::Rng;

const CHARSET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz1234567890";
const LENGTH: usize = 32;

/// generates a unique token
pub fn generate_token() -> String {
    // uses ThreadRng, which "should be secure" as the rand docs state, we will assume it is. 
    let mut rng = rand::rng();

    // add a timestamp to ensure uniqueness
    let time_sufix = Utc::now().timestamp_millis();

    let token: String = (0..LENGTH).map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET.chars().nth(idx).unwrap()
        }).collect();

    format!("{}{}", token, time_sufix)
}