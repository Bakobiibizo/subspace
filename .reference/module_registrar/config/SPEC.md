# Config Component Specification

## Purpose
Defines configuration traits and parameters for the Module Registrar Pallet, allowing for runtime-level customization and governance control.

## Dependencies
```toml
[dependencies]
frame_support = { workspace = true }
frame_system = { workspace = true }
sp-std = { workspace = true }
sp-runtime = { workspace = true }
crate::types = { path = "../types" }
```

## Configuration Trait
```rust
#[pallet::config]
pub trait Config: frame_system::Config {
    /// The overarching event type
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    
    /// Weight information for extrinsics
    type WeightInfo: WeightInfo;
    
    /// The currency type for staking and fees
    type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
    
    /// The origin which can perform admin actions
    type AdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;
    
    /// Maximum number of validators per module
    #[pallet::constant]
    type MaxValidatorsPerModule: Get<u32>;
    
    /// Maximum number of delegators per validator
    #[pallet::constant]
    type MaxDelegatorsPerValidator: Get<u32>;
    
    /// Minimum stake required for validators
    #[pallet::constant]
    type MinValidatorStake: Get<BalanceOf<Self>>;
    
    /// Minimum commission rate for validators
    #[pallet::constant]
    type MinCommissionRate: Get<Percent>;
    
    /// Maximum commission rate for validators
    #[pallet::constant]
    type MaxCommissionRate: Get<Percent>;
    
    /// Period for unbonding stakes
    #[pallet::constant]
    type UnbondingPeriod: Get<Self::BlockNumber>;
    
    /// Maximum size for IPFS content
    #[pallet::constant]
    type MaxContentSize: Get<u64>;
}
```

## Runtime Configuration
```rust
impl pallet_module_registrar::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = weights::pallet_module_registrar::WeightInfo<Runtime>;
    type Currency = Balances;
    type AdminOrigin = EnsureRoot<AccountId>;
    
    const MaxValidatorsPerModule: u32 = 10;
    const MaxDelegatorsPerValidator: u32 = 100;
    const MinValidatorStake: Balance = 5_000;
    const MinCommissionRate: Percent = Percent::from_percent(5);
    const MaxCommissionRate: Percent = Percent::from_percent(50);
    const UnbondingPeriod: BlockNumber = 7 * DAYS;
    const MaxContentSize: u64 = 1024 * 1024 * 50; // 50MB
}
```

## Genesis Configuration
```rust
#[pallet::genesis_config]
pub struct GenesisConfig<T: Config> {
    pub initial_validators: Vec<(T::AccountId, BalanceOf<T>)>,
    pub governance_parameters: GovernanceParameters,
    pub staking_parameters: StakingParameters,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct GovernanceParameters {
    pub voting_period: BlockNumber,
    pub proposal_bond: Balance,
    pub proposal_bond_minimum: Balance,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct StakingParameters {
    pub reward_curve: PiecewiseLinear<'static>,
    pub slash_reward_fraction: Perbill,
    pub slash_period: BlockNumber,
}
```

## Storage Configuration
```rust
#[pallet::storage]
pub type Parameters<T: Config> = StorageValue<_, Parameters, ValueQuery>;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Parameters {
    pub min_stake: Balance,
    pub max_validators: u32,
    pub unbonding_period: BlockNumber,
    pub commission_range: (Percent, Percent),
}
```

## Integration Points
1. Runtime Integration
   - Type definitions
   - Constant values
   - Origin checks

2. Governance System
   - Parameter updates
   - Access control
   - Upgrade handling

3. Economic Parameters
   - Staking configuration
   - Fee structure
   - Reward distribution

4. System Limits
   - Resource constraints
   - Performance bounds
   - Storage quotas

## Parameter Management
1. Governance Control
   - Parameter modification
   - Access restrictions
   - Update scheduling

2. Validation Rules
   - Range checks
   - Dependency validation
   - Consistency enforcement

3. Update Process
   - Atomic updates
   - Migration handling
   - Event emission
