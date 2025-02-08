# Module Registrar Specification

## Overview
The Module Registrar pallet manages the registration, validation, and lifecycle of modules in the Subspace network. It provides functionality for module registration, validator staking, and performance tracking.

## Core Features
1. Module Registration
   - Register new modules with metadata
   - Update existing module metadata
   - Remove modules
   - Track module state transitions

2. Validator Management
   - Validator registration with stake
   - Performance tracking
   - Slashing mechanism
   - Validator set rotation

3. Module Gap Management
   - Efficient module ID reuse
   - Bounded gap storage
   - First-available ID allocation

## Dependencies

### Runtime Dependencies
1. `pallet-balances`
   - Required for: Currency trait implementation
   - Usage: Stake management and slashing
   - Version: Must match polkadot-sdk version (currently polkadot-stable2409)

2. `frame-support`
   - Required for: Core pallet functionality
   - Features needed: 
     - `Currency`
     - `LockableCurrency`
     - `LockIdentifier`
     - `WithdrawReasons`

3. `frame-system`
   - Required for: System integration
   - Features needed:
     - Account management
     - Event handling
     - Block number handling

### Configuration Requirements
1. Runtime Configuration
```rust
pub trait Config: frame_system::Config {
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    type Currency: LockableCurrency<Self::AccountId>;
    type WeightInfo: weights::WeightInfo;
    type MaxValidatorsPerSet: Get<u32>;
    type MaxSlashingEvents: Get<u32>;
    type MaxModulesPerValidator: Get<u32>;
    type MaxModuleIdLen: Get<u32>;
    type MaxModuleGaps: Get<u32>;
}
```

2. Constants
   - `LOCK_ID`: LockIdentifier = *b"modulerg"
   - `MAX_MODULE_ID_LEN`: u32 = 32
   - `MAX_URL_LEN`: u32 = 256
   - `MAX_VALIDATORS_PER_MODULE`: u32 = 10
   - `MAX_DELEGATORS_PER_VALIDATOR`: u32 = 100

## Storage Requirements
1. `ValidatorStake`: Maps AccountId to ValidatorStake
2. `ModuleInfo`: Maps ModuleId to ModuleInfo
3. `ModuleGaps`: Stores available module IDs
4. `ValidatorPerformanceStorage`: Tracks validator metrics
5. `ValidatorScores`: Stores validator rankings
6. `ValidatorTrustScores`: Maintains trust scores

## Security Considerations
1. Access Control
   - Governance checks for admin operations
   - Authorized reporter system for performance tracking
   - Proper stake locking mechanism

2. Economic Security
   - Slashing for misbehavior
   - Minimum stake requirements
   - Performance-based rewards

## Integration Requirements
1. Runtime Integration
   - Add pallet to `construct_runtime!`
   - Configure pallet in runtime
   - Set up proper event handling

2. Genesis Configuration
   - Optional initial validators
   - Optional initial modules
   - System parameters

3. RPC Interface
   - Query module information
   - Query validator status
   - Performance metrics
