use super::*;
use frame_support::{
    pallet_prelude::*,
    traits::{Currency, LockIdentifier, LockableCurrency, WithdrawReasons},
};
use frame_system::{pallet_prelude::BlockNumberFor, Config as SystemConfig};
use sp_runtime::{
    traits::{Zero, Saturating},
    Percent, SaturatedConversion,
};
use sp_std::prelude::*;
use scale_info::TypeInfo;

const VALIDATOR_LOCK_ID: LockIdentifier = *b"validatr";

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct ValidatorPerformanceMetrics<T: Config>
where
    <T as pallet_governance::Config>::Currency: LockableCurrency<<T as SystemConfig>::AccountId>,
{
    pub total_validations: u32,
    pub successful_validations: u32,
    pub failed_validations: u32,
    pub age_in_blocks: u32,
    pub last_update: BlockNumberFor<T>,
}

impl<T: Config> Default for ValidatorPerformanceMetrics<T>
where
    <T as pallet_governance::Config>::Currency: LockableCurrency<<T as SystemConfig>::AccountId>,
{
    fn default() -> Self {
        Self {
            total_validations: 0,
            successful_validations: 0,
            failed_validations: 0,
            age_in_blocks: 0,
            last_update: Zero::zero(),
        }
    }
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo, Default)]
pub struct ValidationScore {
    pub stake_score: u32,
    pub performance_score: u32,
    pub age_score: u32,
    pub trust_score: u32,
    pub total_score: u32,
}

impl ValidationScore {
    pub fn calculate(
        stake: u128,
        max_stake: u128,
        performance: Percent,
        age: u32,
        max_age: u32,
        trust: u32,
    ) -> Self {
        // Normalize stake to 0-100
        let stake_score = if max_stake > 0 {
            ((stake * 100) / max_stake) as u32
        } else {
            0
        };

        // Convert performance to 0-100
        let performance_score = performance.deconstruct() as u32;

        // Normalize age to 0-100
        let age_score = if max_age > 0 {
            ((age as u128 * 100) / max_age as u128) as u32
        } else {
            0
        };

        // Trust score is already 0-100
        let trust_score = trust.min(100);

        // Calculate total score (weighted average)
        let total_score = stake_score * 40 +
            performance_score * 30 +
            age_score * 20 +
            trust_score * 10;

        Self {
            stake_score,
            performance_score,
            age_score,
            trust_score,
            total_score: total_score / 100,
        }
    }
}

impl<T: Config + TypeInfo> Pallet<T>
where
    <T as pallet_governance::Config>::Currency: LockableCurrency<<T as SystemConfig>::AccountId>,
{
    // Register a new validator
    pub fn do_register_validator(
        who: &T::AccountId,
        stake: BalanceOf<T>,
        requirements: types::ValidatorRequirements<BlockNumberFor<T>>,
    ) -> DispatchResult {
        // Ensure minimum stake requirements are met
        let min_stake: BalanceOf<T> = requirements.min_self_stake.saturated_into();
        ensure!(stake >= min_stake, Error::<T>::InsufficientStake);

        // Lock the stake
        <<T as pallet_governance::Config>::Currency as LockableCurrency<T::AccountId>>::set_lock(
            VALIDATOR_LOCK_ID,
            who,
            stake,
            WithdrawReasons::all(),
        );

        // Initialize validator stake info
        let validator_stake = types::ValidatorStake {
            validator: who.clone(),
            total_stake: stake,
            self_stake: stake,
            delegated_stake: Zero::zero(),
            delegators: BoundedVec::default(),
            commission_rate: requirements.max_commission_rate,
        };

        // Initialize performance metrics
        let performance = ValidatorPerformanceMetrics::<T>::default();

        // Store validator information
        <ValidatorStake<T>>::insert(who, validator_stake);
        <ValidatorPerformanceStorage<T>>::insert(who, performance);

        // Emit event
        Self::deposit_event(Event::ValidatorRegistered(who.clone()));

        Ok(())
    }

    // Update validator performance metrics
    pub fn update_validator_performance(
        who: &T::AccountId,
        success: bool,
    ) -> DispatchResult {
        <ValidatorPerformanceStorage<T>>::try_mutate(who, |perf| -> DispatchResult {
            let perf = perf.as_mut().ok_or(Error::<T>::ValidatorNotFound)?;
            
            perf.total_validations = perf.total_validations.saturating_add(1);
            if success {
                perf.successful_validations = perf.successful_validations.saturating_add(1);
            } else {
                perf.failed_validations = perf.failed_validations.saturating_add(1);
            }
            
            perf.last_update = frame_system::Pallet::<T>::block_number();
            Ok(())
        })
    }

    // Calculate and update validator scores
    pub fn calculate_validator_scores() -> DispatchResult {
        let validators: Vec<_> = <ValidatorStake<T>>::iter().collect();
        if validators.is_empty() {
            return Ok(());
        }

        // Find maximum stake for normalization
        let max_stake = validators
            .iter()
            .map(|(_, stake)| stake.total_stake)
            .max()
            .unwrap_or_else(Zero::zero);

        // Find maximum age for normalization
        let max_age = <ValidatorPerformanceStorage<T>>::iter()
            .map(|(_, perf)| perf.age_in_blocks)
            .max()
            .unwrap_or(1);

        // Calculate and store scores for each validator
        for (validator, stake) in validators {
            if let Some(perf) = <ValidatorPerformanceStorage<T>>::get(&validator) {
                let performance = Percent::from_rational(
                    perf.successful_validations,
                    perf.total_validations.max(1),
                );

                let score = ValidationScore::calculate(
                    stake.total_stake.saturated_into::<u128>(),
                    max_stake.saturated_into::<u128>(),
                    performance,
                    perf.age_in_blocks,
                    max_age,
                    <ValidatorTrustScores<T>>::get(&validator),
                );

                <ValidatorScores<T>>::insert(&validator, score);
            }
        }

        Ok(())
    }

    // Select top validators based on scores
    pub fn select_top_validators(count: u32) -> Vec<T::AccountId> {
        let mut validators: Vec<_> = <ValidatorScores<T>>::iter()
            .collect();

        // Sort by total score in descending order
        validators.sort_by(|(_, a), (_, b)| b.total_score.cmp(&a.total_score));

        // Take top N validators
        validators
            .into_iter()
            .take(count as usize)
            .map(|(validator, _)| validator)
            .collect()
    }

    // Slash a validator for misbehavior
    pub fn do_slash_validator(
        who: &T::AccountId,
        slash_amount: BalanceOf<T>,
    ) -> DispatchResult {
        <ValidatorStake<T>>::try_mutate(who, |validator| -> DispatchResult {
            let validator = validator.as_mut().ok_or(Error::<T>::ValidatorNotFound)?;
            
            // Calculate proportional slash amounts for delegators
            let total_stake = validator.total_stake;
            let self_slash = slash_amount.saturating_mul(validator.self_stake.into())
                / total_stake.into();

            // Slash validator's self stake
            <T as pallet_governance::Config>::Currency::slash(who, self_slash);
            validator.self_stake = validator.self_stake.saturating_sub(self_slash);
            validator.total_stake = validator.total_stake.saturating_sub(slash_amount);

            // Record slashing event
            SlashingEvents::<T>::try_mutate(|events| {
                events.try_push(types::SlashEvent {
                    validator: who.clone(),
                    amount: slash_amount,
                    block: frame_system::Pallet::<T>::block_number(),
                })
            }).map_err(|_| Error::<T>::TooManySlashingEvents)?;

            // Emit event
            Self::deposit_event(Event::ValidatorSlashed(who.clone(), slash_amount));

            Ok(())
        })
    }
}
