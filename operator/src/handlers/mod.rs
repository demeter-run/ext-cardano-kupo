pub mod auth;
pub mod gateway;

// helpers
pub fn get_auth_name(name: &str) -> String {
    format!("kupo-auth-{}", name)
}

pub fn get_rate_limit_name(tier: &str) -> String {
    format!("rate-limiting-kupo-tier-{}", tier)
}
