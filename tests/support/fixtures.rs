//! Test fixtures and constants.

use age::secrecy::ExposeSecret;
use age::x25519;

/// A valid age public key for testing team operations.
pub const BOB_PUBLIC_KEY: &str = "age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p";

/// An invalid public key for negative tests.
pub const INVALID_PUBLIC_KEY: &str = "not-a-valid-age-key";

/// Standard test secrets used across multiple tests.
pub const STANDARD_SECRETS: &[(&str, &str)] = &[
    ("DATABASE_URL", "postgres://localhost/mydb"),
    ("API_KEY", "sk-test-12345"),
    ("JWT_SECRET", "super-secret-jwt-token"),
    ("REDIS_URL", "redis://localhost:6379"),
    ("S3_BUCKET", "my-app-bucket"),
];

/// Sample .env file content for import tests.
pub const SAMPLE_ENV: &str = "KEY1=value1\nKEY2=value2\nKEY3=value3\n";

/// Sample .env with edge cases.
pub const SAMPLE_ENV_COMPLEX: &str = r#"
# This is a comment
SIMPLE=value
QUOTED="quoted value"
SINGLE_QUOTED='single quoted'
SPACES_IN_VALUE=hello world

# Another comment
SPECIAL_CHARS=p@ssw0rd!#$%
"#;

/// Generate a fresh age keypair for testing.
///
/// Returns (public_key, private_key) as strings.
pub fn generate_age_keypair() -> (String, String) {
    let secret_key = x25519::Identity::generate();
    let public_key = secret_key.to_public();
    let private_key = secret_key.to_string().expose_secret().to_string();
    (public_key.to_string(), private_key)
}
