use crate::*;
use frame_support::{pallet_prelude::*, traits::Currency};
use frame_system::{self, ensure_signed, Error, Event, Config, Pallet, pallet_prelude::*};
use pallet_subspace::Pallet as PalletSubspace;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::Percent;
use sp_std::vec::Vec;

/// Module type classification for registration and validation
#[derive(Clone, Debug, PartialEq, Eq, MaxEncodedLen, TypeInfo, Decode, Encode)]
pub enum ModuleType {
    Inference,
    Storage,
    Validation,
    Custom(BoundedVec<u8, ConstU32<32>>),
}

/// Emission control configuration for validators
#[derive(Clone, Debug, PartialEq, Eq, MaxEncodedLen, TypeInfo, Decode, Encode)]
#[scale_info(skip_type_params(T))]
pub struct EmissionControl<T: Config> {
    validator: <T as frame_system::Config>::AccountId,
    adjustment: EmissionAdjustment,
    duration: <T as frame_system::Config>::BlockNumber,
    proposed_by: <T as frame_system::Config>::AccountId,
    approved_at: <T as frame_system::Config>::BlockNumber,
}

#[derive(Clone, Debug, PartialEq, Eq, MaxEncodedLen, TypeInfo, Decode, Encode)]
pub enum EmissionAdjustment {
    Reduce(Percent),
    Suspend,
    Resume,
}

/// Slashing action for validator penalties
#[derive(Clone, Debug, PartialEq, Eq, MaxEncodedLen, TypeInfo, Decode, Encode)]
#[scale_info(skip_type_params(T))]
pub struct SlashAction<T: Config> {
    target: <T as frame_system::Config>::AccountId,
    amount: Percent,
    reason: BoundedVec<u8, ConstU32<256>>,
    proposed_by: <T as frame_system::Config>::AccountId,
}

/// Storage items for DAO security controls
#[pallet::storage]
pub type ValidatorEmissions<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    <T as frame_system::Config>::AccountId,
    EmissionControl<T>,
    OptionQuery,
>;

#[pallet::storage]
pub type PendingSlashes<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    <T as frame_system::Config>::AccountId,
    SlashAction<T>,
    OptionQuery,
>;

impl<T: Config> Pallet<T> {
    /// Validate a module application based on type and requirements
    pub fn validate_module_application(
        application: &CuratorApplication<T>,
        key: &<T as frame_system::Config>::AccountId,
    ) -> DispatchResult {
        let module_type = ModuleType::decode(&mut &application.data[..])
            .map_err(|_| Error::<T>::InvalidModuleType)?;

        // Verify stake amount meets module type requirements
        let min_stake = match module_type {
            ModuleType::Inference => T::MinInferenceStake::get(),
            ModuleType::Storage => T::MinStorageStake::get(),
            ModuleType::Validation => T::MinValidationStake::get(),
            ModuleType::Custom(_) => T::MinCustomStake::get(),
        };

        ensure!(
            PalletSubspace::<T>::has_enough_balance(key, min_stake),
            Error::<T>::InsufficientStake
        );

        Ok(())
    }

    /// Propose an emission adjustment for a validator
    pub fn propose_emission_adjustment(
        origin: OriginFor<T>,
        validator: <T as frame_system::Config>::AccountId,
        adjustment: EmissionAdjustment,
        duration: <T as frame_system::Config>::BlockNumber,
    ) -> DispatchResult {
        let proposer = ensure_signed(origin)?;
        ensure!(Self::is_dao_member(&proposer), Error::<T>::NotDAOMember);

        // Ensure validator exists
        ensure!(
            governance::Validators::<T>::contains_key(&validator),
            Error::<T>::ValidatorNotFound
        );

        // Validate adjustment parameters
        match adjustment {
            EmissionAdjustment::Reduce(percent) => {
                ensure!(
                    percent <= Percent::from_percent(90),
                    Error::<T>::InvalidEmissionAdjustment
                );
            }
            _ => {}
        }

        let current_block = frame_system::Pallet::<T>::block_number();
        let emission_control = EmissionControl {
            validator: validator.clone(),
            adjustment,
            duration,
            proposed_by: proposer,
            approved_at: current_block,
        };

        ValidatorEmissions::<T>::insert(&validator, emission_control);
        Self::deposit_event(Event::<T>::EmissionAdjustmentProposed(validator));

        Ok(())
    }

    /// Propose slashing a validator
    pub fn propose_slash(
        origin: OriginFor<T>,
        target: <T as frame_system::Config>::AccountId,
        amount: Percent,
        reason: Vec<u8>,
    ) -> DispatchResult {
        let proposer = ensure_signed(origin)?;
        ensure!(Self::is_dao_member(&proposer), Error::<T>::NotDAOMember);

        // Ensure validator exists
        ensure!(
            governance::Validators::<T>::contains_key(&target),
            Error::<T>::ValidatorNotFound
        );

        // Validate slash amount
        ensure!(
            amount <= Percent::from_percent(100),
            Error::<T>::InvalidSlashAmount
        );

        let slash_action = SlashAction {
            target: target.clone(),
            amount,
            reason: BoundedVec::truncate_from(reason),
            proposed_by: proposer,
        };

        PendingSlashes::<T>::insert(&target, slash_action);
        Self::deposit_event(Event::<T>::SlashProposed(target));

        Ok(())
    }

    /// Execute a pending slash action
    pub fn execute_slash(
        origin: OriginFor<T>,
        target: <T as frame_system::Config>::AccountId,
    ) -> DispatchResult {
        let executor = ensure_signed(origin)?;
        ensure!(Self::is_dao_member(&executor), Error::<T>::NotDAOMember);

        let slash_action = PendingSlashes::<T>::take(&target)
            .ok_or(Error::<T>::SlashActionNotFound)?;

        // Execute the slash
        if let Some(stake) = governance::Validators::<T>::get(&target) {
            let slash_amount = stake.total_stake * slash_action.amount;
            PalletSubspace::<T>::slash_validator(&target, slash_amount)?;
        }

        Self::deposit_event(Event::<T>::SlashExecuted(target));
        Ok(())
    }

    /// Check if an account is a DAO member
    pub fn is_dao_member(account: &<T as frame_system::Config>::AccountId) -> bool {
        Self::is_in_legit_whitelist(account)
    }
}
