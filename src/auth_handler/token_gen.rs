use rand::Rng;

const CHARSET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz1234567890";
const LENGTH: usize = 32;

pub fn generate_token() -> String {
    // FIXME: add timestamp to ensure uniqueness
    // uses ThreadRng, which "should be secure" as the rand docs state, we will assume it is. 
    let mut rng = rand::rng(); 

    (0..LENGTH).map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET.chars().nth(idx).unwrap()
        }).collect()
}