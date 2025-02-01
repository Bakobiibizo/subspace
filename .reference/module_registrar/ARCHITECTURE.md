# Module Registrar Pallet Architecture

## Overview

The Module Registrar Pallet is a core component of the Subspace blockchain that enables decentralized package management and service registration. It replaces the previous subnet system with a more flexible and scalable architecture for managing off-chain modules that provide AI inference, storage, and validation services.

## Core Components

### 1. Storage and Types

```rust
pub type ModuleId = BoundedVec<u8, MaxModuleIdLen>;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct ModuleInfo<AccountId, Balance> {
    // Owner of the module
    owner: AccountId,
    // Module metadata
    metadata: ModuleMetadata,
    // Current state
    state: ModuleState,
    // Staked amount
    stake: Balance,
    // List of authorized validators
    validators: BoundedVec<AccountId, MaxValidatorsPerModule>,
    // Trust score
    trust_score: u32,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct ModuleMetadata {
    version_major: u16,
    version_minor: u16,
    version_patch: u16,
    repo_url: BoundedVec<u8, MaxUrlLen>,
    build_script_url: BoundedVec<u8, MaxUrlLen>,
    installer_script_url: BoundedVec<u8, MaxUrlLen>,
    ipfs_hash: H256,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum ModuleState {
    Pending,    // Awaiting validation
    Active,     // Validated and running
    Suspended,  // Temporarily disabled
    Deprecated, // No longer supported
}

// Enhanced Staking Types
pub struct ValidatorStake<AccountId, Balance> {
    validator: AccountId,
    total_stake: Balance,
    self_stake: Balance,
    delegated_stake: Balance,
    delegators: BoundedVec<(AccountId, Balance), MaxDelegatorsPerValidator>,
    commission_rate: Percent,
}

pub struct ValidatorRequirements {
    min_self_stake: u64,
    min_total_stake: u64,
    max_commission_rate: Percent,
    max_delegators: u32,
    validation_period: BlockNumber,
}

pub struct ValidatorWeights {
    stake_weight: u32,
    performance_weight: u32,
    age_weight: u32,
    trust_score: u32,
    total_weight: u32,
}

pub struct UnbondingInfo<AccountId, Balance, BlockNumber> {
    delegator: AccountId,
    validator: AccountId,
    amount: Balance,
    started_at: BlockNumber,
    completion_block: BlockNumber,
}

// Storage Items
#[pallet::storage]
pub type Validators<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, ValidatorStake<T::AccountId, BalanceOf<T>>>;

#[pallet::storage]
pub type ValidatorRequirementsByModule<T: Config> = StorageMap<_, Blake2_128Concat, ModuleId, ValidatorRequirements>;

#[pallet::storage]
pub type ValidatorWeightsByModule<T: Config> = StorageDoubleMap<
    _, 
    Blake2_128Concat, ModuleId,
    Blake2_128Concat, T::AccountId,
    ValidatorWeights
>;

#[pallet::storage]
pub type UnbondingRequests<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat, T::AccountId, // delegator
    Blake2_128Concat, T::AccountId, // validator
    UnbondingInfo<T::AccountId, BalanceOf<T>, T::BlockNumber>
>;

#[pallet::storage]
pub type UnbondingPeriod<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;
```

### 2. Extrinsics

#### Module Management
- `register_module(origin, module_id, metadata, stake)`
- `update_module(origin, module_id, metadata)`
- `deprecate_module(origin, module_id)`
- `suspend_module(origin, module_id)`
- `reactivate_module(origin, module_id)`

#### Validation
- `validate_module(origin, module_id, container_hash)`
- `nominate_validator(origin, module_id, validator)`
- `remove_validator(origin, module_id, validator)`

#### Staking and Economics
- `increase_stake(origin, module_id, additional_amount)`
- `decrease_stake(origin, module_id, amount)`
- `claim_rewards(origin, module_id)`

### 3. Events

```rust
pub enum Event<T: Config> {
    ModuleRegistered(ModuleId, AccountId),
    ModuleUpdated(ModuleId),
    ModuleDeprecated(ModuleId),
    ModuleSuspended(ModuleId),
    ModuleReactivated(ModuleId),
    ModuleValidated(ModuleId, AccountId),
    ValidatorNominated(ModuleId, AccountId),
    ValidatorRemoved(ModuleId, AccountId),
    StakeIncreased(ModuleId, Balance),
    StakeDecreased(ModuleId, Balance),
    RewardsClaimed(ModuleId, AccountId, Balance),
}
```

### 4. Errors

```rust
pub enum Error<T> {
    ModuleNotFound,
    ModuleAlreadyExists,
    InsufficientStake,
    UnauthorizedOperation,
    InvalidMetadata,
    InvalidState,
    TooManyValidators,
    ValidatorAlreadyNominated,
    ValidatorNotFound,
    InsufficientBalance,
}
```

### 4. Validation System

The validation system implements a multi-layered approach to ensure module integrity and performance:

#### Validator Selection
- Validators must meet minimum stake requirements per module
- Selection based on weighted scoring system:
  - Stake weight (40%): Based on total stake amount
  - Performance weight (30%): Historical validation success rate
  - Age weight (20%): Length of time as validator
  - Trust score (10%): Community and governance ratings

#### Validation Process
```rust
pub enum ValidationState {
    Pending,
    InProgress {
        started_at: BlockNumber,
        validator: AccountId,
    },
    Completed {
        completed_at: BlockNumber,
        validator: AccountId,
        result: ValidationResult,
    },
    Failed {
        failed_at: BlockNumber,
        validator: AccountId,
        error: ValidationError,
    },
}

pub struct ValidationResult {
    ipfs_hash: H256,
    performance_metrics: BTreeMap<String, u32>,
    resource_usage: ResourceUsage,
}

pub struct ResourceUsage {
    cpu_usage: u32,
    memory_usage: u32,
    network_bandwidth: u32,
    storage_usage: u32,
}
```

#### Staking Mechanics
1. **Validator Staking**
   - Must maintain minimum self-stake of 5000 COM
   - Can accept unlimited delegations
   - Commission rate bounded by module parameters min 5% max 50%

2. **Delegator Staking**
   - Can delegate to a single validator
   - Subject to unbonding period
     - 7 day mandatory waiting period for withdrawals
     - Tokens locked during unbonding
     - No rewards earned during unbonding
     - Slashing still possible during unbonding
   - Rewards based on stake ratio minus commission

3. **Slashing Conditions**
   - Invalid validation results
   - Missed validation duties
   - Malicious behavior detection
   - Governance-triggered slashing

## Integration Points

### 1. Runtime Integration

The Module Registrar Pallet integrates with the following runtime components:

- **Balances Pallet**: For stake management and rewards
- **Governance Pallet**: For parameter updates and emergency controls
- **System Pallet**: For basic system operations

### 2. Off-chain Components

- **IPFS Integration**: For storing validated module containers
- **Runtime API**: For querying module state and metadata
- **RPC Interface**: For submitting validation results

## Security and Trust Model

### 1. Stake-Based Security

- Minimum stake requirements for module registration
- Slashing conditions for malicious behavior
- Stake scaling based on module usage and trust score

### 2. Trust Score Calculation

```rust
pub struct TrustScore {
    base_score: u32,
    validator_ratings: Vec<u32>,
    user_ratings: Vec<u32>,
    uptime: u32,
    performance_metrics: Vec<u32>,
}
```

### 3. Validator Selection

- Minimum stake requirement for validators
- Performance-based validator rotation
- Slashing for incorrect validations

## Governance Parameters

The following parameters are governed by the DAO:

- `MinModuleStake`: Minimum stake required to register a module
- `MaxValidatorsPerModule`: Maximum number of validators per module
- `ValidationPeriod`: Time window for module validation
- `MinValidatorStake`: Minimum stake required to become a validator
- `RewardRate`: Rate at which rewards are distributed
- `SlashingRate`: Rate at which stakes are slashed for violations

## Future Considerations

1. **Cross-Chain Integration**
   - Bridge support for cross-chain module interactions
   - Cross-chain stake and reward management

2. **Scalability Improvements**
   - Hierarchical validation system
   - Sharded module storage

3. **Enhanced Security**
   - Zero-knowledge proof integration
   - Advanced slashing mechanisms
