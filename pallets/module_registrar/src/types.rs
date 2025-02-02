use frame_support::pallet_prelude::*;
use sp_core::H256;
use sp_runtime::{Percent, traits::Zero};
use sp_std::prelude::*;

/// Constants for type bounds
pub const MAX_MODULE_ID_LEN: u32 = 32;
pub const MAX_URL_LEN: u32 = 256;
pub const MAX_VALIDATORS_PER_MODULE: u32 = 10;
pub const MAX_DELEGATORS_PER_VALIDATOR: u32 = 100;

/// Type for module identifier
pub type ModuleId = BoundedVec<u8, ConstU32<MAX_MODULE_ID_LEN>>;

/// Type for URL storage with bounded length
pub type Url = BoundedVec<u8, ConstU32<MAX_URL_LEN>>;

/// Core information about a registered module
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct ModuleInfo<AccountId, Balance> {
    /// Owner of the module
    pub owner: AccountId,
    /// Module metadata
    pub metadata: ModuleMetadata,
    /// Current state
    pub state: ModuleState,
    /// Staked amount
    pub stake: Balance,
    /// List of authorized validators
    pub validators: BoundedVec<AccountId, ConstU32<MAX_VALIDATORS_PER_MODULE>>,
    /// Trust score
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

/// Validator staking information
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct ValidatorStake<AccountId, Balance> {
    pub validator: AccountId,
    pub total_stake: Balance,
    pub self_stake: Balance,
    pub delegated_stake: Balance,
    pub delegators: BoundedVec<(AccountId, Balance), ConstU32<MAX_DELEGATORS_PER_VALIDATOR>>,
    pub commission_rate: Percent,
}

/// Requirements for validators
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct ValidatorRequirements<BlockNumber> {
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
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct ValidationResult {
    pub ipfs_hash: H256,
    pub performance_metrics: BoundedVec<(BoundedVec<u8, ConstU32<32>>, u32), ConstU32<100>>,
    pub resource_usage: ResourceUsage,
}

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

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct SlashEvent<AccountId, Balance, BlockNumber> {
    pub validator: AccountId,
    pub amount: Balance,
    pub block: BlockNumber,
}
