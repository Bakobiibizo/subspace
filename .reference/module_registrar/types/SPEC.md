# Types Component Specification

## Purpose
Defines core types, traits, and interfaces used throughout the Module Registrar Pallet.

## Dependencies
```toml
[dependencies]
frame_support = { workspace = true }
sp-std = { workspace = true }
sp-runtime = { workspace = true }
parity-scale-codec = { workspace = true, features = ["derive"] }
scale-info = { workspace = true, features = ["derive"] }
```

## Core Types
```rust
/// Type for module identifier
pub type ModuleId = BoundedVec<u8, MaxModuleIdLen>;

/// Type for URL storage with bounded length
pub type Url = BoundedVec<u8, MaxUrlLen>;

/// Core information about a registered module
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct ModuleInfo<AccountId, Balance> {
    pub owner: AccountId,
    pub metadata: ModuleMetadata,
    pub state: ModuleState,
    pub stake: Balance,
    pub validators: BoundedVec<AccountId, MaxValidatorsPerModule>,
    pub trust_score: u32,
}

/// Metadata for a module
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct ModuleMetadata {
    pub version_major: u16,
    pub version_minor: u16,
    pub version_patch: u16,
    pub repo_url: Url,
    pub build_script_url: Url,
    pub installer_script_url: Url,
    pub ipfs_hash: H256,
}

/// Possible states for a module
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum ModuleState {
    Pending,    // Awaiting validation
    Active,     // Validated and running
    Suspended,  // Temporarily disabled
    Deprecated, // No longer supported
}
```

## Validation Types
```rust
/// Validator staking information
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct ValidatorStake<AccountId, Balance> {
    pub validator: AccountId,
    pub total_stake: Balance,
    pub self_stake: Balance,
    pub delegated_stake: Balance,
    pub delegators: BoundedVec<(AccountId, Balance), MaxDelegatorsPerValidator>,
    pub commission_rate: Percent,
}

/// Requirements for validators
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct ValidatorRequirements {
    pub min_self_stake: u64,
    pub min_total_stake: u64,
    pub max_commission_rate: Percent,
    pub max_delegators: u32,
    pub validation_period: BlockNumber,
}

/// Validator performance weights
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct ValidatorWeights {
    pub stake_weight: u32,      // 40%
    pub performance_weight: u32, // 30%
    pub age_weight: u32,        // 20%
    pub trust_score: u32,       // 10%
    pub total_weight: u32,
}

impl ValidatorWeights {
    pub fn calculate_total(&mut self) {
        self.total_weight = self.stake_weight
            .saturating_add(self.performance_weight)
            .saturating_add(self.age_weight)
            .saturating_add(self.trust_score);
    }

    pub fn new() -> Self {
        let mut weights = Self {
            stake_weight: 0,
            performance_weight: 0,
            age_weight: 0,
            trust_score: 0,
            total_weight: 0,
        };
        weights.calculate_total();
        weights
    }
}
```

## Economic Types
```rust
/// Information about unbonding stakes
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct UnbondingInfo<AccountId, Balance, BlockNumber> {
    pub delegator: AccountId,
    pub validator: AccountId,
    pub amount: Balance,
    pub started_at: BlockNumber,
    pub completion_block: BlockNumber,
}

/// Resource usage metrics
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct ResourceUsage {
    pub cpu_usage: u32,
    pub memory_usage: u32,
    pub network_bandwidth: u32,
    pub storage_usage: u32,
}

/// Validation result with performance metrics
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct ValidationResult {
    pub ipfs_hash: H256,
    pub performance_metrics: BTreeMap<Vec<u8>, u32>,
    pub resource_usage: ResourceUsage,
}
```

## Constants
```rust
/// Constants for type bounds
pub const MAX_MODULE_ID_LEN: u32 = 32;
pub const MAX_URL_LEN: u32 = 256;
pub const MAX_VALIDATORS_PER_MODULE: u32 = 10;
pub const MAX_DELEGATORS_PER_VALIDATOR: u32 = 100;
```

## Error Types
```rust
/// Common error types
#[derive(Debug)]
pub enum Error {
    ModuleNotFound,
    InvalidState,
    InsufficientStake,
    TooManyValidators,
    TooManyDelegators,
    InvalidCommissionRate,
    ValidationInProgress,
    UnbondingInProgress,
}
```

## Traits
```rust
/// Helper trait for stake management
pub trait StakeHandler {
    type AccountId;
    type Balance: Zero;

    fn stake(account: &Self::AccountId, amount: Self::Balance) -> DispatchResult;
    fn unstake(account: &Self::AccountId, amount: Self::Balance) -> DispatchResult;
    fn slash(account: &Self::AccountId, amount: Self::Balance) -> DispatchResult;
}

/// Helper trait for validation operations
pub trait ModuleValidator {
    type AccountId;
    type BlockNumber;

    fn validate_module(module_id: ModuleId) -> Result<ValidationResult, DispatchError>;
    fn calculate_trust_score(metrics: &ValidationResult) -> u32;
    fn is_validator(account: &Self::AccountId) -> bool;
}
```

## Usage Guidelines
1. Type Safety
   - Use strong typing for all values
   - Implement proper error handling
   - Use bounded vectors for lists

2. Encoding
   - All types must implement Encode/Decode, MaxEncodedLen, and TypeInfo
   - Use proper scale encoding
   - Handle versioning

3. Performance
   - Minimize storage impact
   - Optimize for common operations
   - Consider gas costs

4. Integration
   - Maintain consistent interfaces
   - Use standard traits
   - Follow Substrate patterns
