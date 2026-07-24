use rmcp::model::ProtocolVersion;

/// Single MCP revision supported by Sora.
pub const TARGET_PROTOCOL_VERSION: ProtocolVersion = ProtocolVersion::V_2025_11_25;

/// Stable MCP implementation name reported during initialization.
pub const SERVER_NAME: &str = "sora";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_revision_is_pinned_to_the_goal_contract() {
        assert_eq!(TARGET_PROTOCOL_VERSION.as_str(), "2025-11-25");
        assert_eq!(ProtocolVersion::default(), TARGET_PROTOCOL_VERSION);
    }
}
