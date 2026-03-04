//! Solidity contracts for UTS

/// EAS contract
pub mod eas;

/// Incomplete Solidity interfaces for the L1 Gateway contract. This is not a complete interface,
/// but only includes the events and functions that are relevant to the server/backend.
pub mod gateway;

/// Incomplete Solidity interfaces for the L2 Anchoring Manager. This is not a complete interface,
/// but only includes the events and functions that are relevant to the server/backend.
pub mod manager;

pub mod fee_oracle;
