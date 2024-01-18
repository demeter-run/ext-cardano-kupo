pub mod auth;

pub fn get_rate_limit_name(tier: &str) -> String {
    format!("rate-limiting-kupo-tier-{}", tier)
}
