pub fn generate_auth_token() -> String {
    use rand::{Rng, rng};
    // Define the character set to use for the token.
    // This includes uppercase, lowercase letters, and digits.
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    const TOKEN_LENGTH: usize = 12; // Set your desired token length here

    let mut rng = rng();
    let token: String = (0..TOKEN_LENGTH)
        .map(|_| {
            // Pick a random index into the CHARSET array
            let idx = rng.random_range(0..CHARSET.len());
            // Convert the random byte (ASCII char) to a char
            CHARSET[idx] as char
        })
        .collect(); // Collect all characters into a String

    token
}

pub fn calculate_response(data: &str) -> String {
    data.to_string()
}
