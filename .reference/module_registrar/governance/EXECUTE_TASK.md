# EXECUTE TASK: Implement Core DAO Security Controls

## Context
- Current implementation: pallets/governance/src/lib.rs
- Dependencies: 
  - Existing whitelist system
  - Module registrar pallet
  - Governance pallet
- Related files: 
  - GOVERNANCE_ENHANCEMENT_SPEC.md
  - PROGRESS.md

## Objectives
1. Repurpose whitelist for module registration
2. Add basic emission control
3. Implement simple slashing mechanism

## Implementation Steps

### Phase 1: Whitelist Integration (Week 1)
1. Extend CuratorApplication
```rust
pub struct CuratorApplication {
    // Existing fields
    module_type: ModuleType,
    stake_amount: Balance,
}

pub enum ModuleType {
    Inference,
    Storage,
    Validation,
    Custom(Vec<u8>),
}
```

2. Update application validation
```rust
impl<T: Config> Pallet<T> {
    fn validate_module_application(
        application: &CuratorApplication,
        key: &T::AccountId,
    ) -> DispatchResult {
        // Validate stake amount
        // Check module type requirements
        // Verify module capabilities
        Ok(())
    }
}
```

### Phase 2: Emission Control (Week 2)
1. Add minimal emission control structures
```rust
pub struct EmissionControl {
    validator: AccountId,
    adjustment: EmissionAdjustment,
    duration: BlockNumber,
}

pub enum EmissionAdjustment {
    Reduce(Percent),
    Suspend,
    Resume,
}

#[pallet::storage]
pub type ValidatorEmissions<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    EmissionControl<T>,
    OptionQuery,
>;
```

2. Add emission control extrinsics
```rust
impl<T: Config> Pallet<T> {
    pub fn propose_emission_adjustment(
        origin: OriginFor<T>,
        validator: T::AccountId,
        adjustment: EmissionAdjustment,
        duration: T::BlockNumber,
    ) -> DispatchResult {
        // Ensure DAO member
        // Validate adjustment parameters
        // Create and store proposal
        Ok(())
    }

    pub fn execute_emission_adjustment(
        origin: OriginFor<T>,
        proposal_id: ProposalId,
    ) -> DispatchResult {
        // Verify proposal passed
        // Apply emission adjustment
        // Emit events
        Ok(())
    }
}
```

### Phase 3: Basic Slashing (Week 3)
1. Add slashing structures
```rust
pub struct SlashAction {
    target: AccountId,
    amount: Percent,
    reason: Vec<u8>,
}

#[pallet::storage]
pub type PendingSlashes<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    SlashAction<T>,
    OptionQuery,
>;
```

2. Add slashing extrinsics
```rust
impl<T: Config> Pallet<T> {
    pub fn propose_slash(
        origin: OriginFor<T>,
        target: T::AccountId,
        amount: Percent,
        reason: Vec<u8>,
    ) -> DispatchResult {
        // Ensure DAO member
        // Validate slash parameters
        // Create and store proposal
        Ok(())
    }

    pub fn execute_slash(
        origin: OriginFor<T>,
        proposal_id: ProposalId,
    ) -> DispatchResult {
        // Verify proposal passed
        // Execute slashing
        // Emit events
        Ok(())
    }
}
```

### Testing Requirements
1. Whitelist Tests
   - Module registration flow
   - Application validation
   - Stake verification

2. Emission Control Tests
   - Proposal creation
   - Adjustment execution
   - Duration handling

3. Slashing Tests
   - Proposal validation
   - Execution accuracy
   - Event emission

## Documentation Updates
1. Update PROGRESS.md with implementation status
2. Add new governance capabilities to ARCHITECTURE.md
3. Document DAO member instructions for using new controls

## Migration Plan
1. No breaking changes to existing whitelist functionality
2. New storage items will be initialized empty
3. Existing proposals will be unaffected

## Risks and Mitigations
1. Risk: Emission control abuse
   - Mitigation: Require multiple DAO signatures
   - Mitigation: Add cool-down periods

2. Risk: Excessive slashing
   - Mitigation: Cap maximum slash percentage
   - Mitigation: Require higher voting threshold

3. Risk: Module registration spam
   - Mitigation: Maintain minimum stake requirement
   - Mitigation: Add application rate limiting

## Success Criteria
1. DAO can effectively control validator emissions
2. Bad actors can be quickly slashed
3. Module registration process is secure and efficient
4. All operations are properly logged and transparent

## Rollback Plan
1. Keep old whitelist logic
2. Version new storage items
3. Prepare reversion scripts
