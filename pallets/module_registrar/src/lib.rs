#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
pub use types::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod types;
pub mod weights;

use frame_support::{
    pallet_prelude::*,
    traits::{Currency, ReservableCurrency},
};
use frame_system::pallet_prelude::*;

type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
        type WeightInfo: weights::WeightInfo;
        type BlockNumber: Member + Parameter + Default + Copy + MaxEncodedLen + From<u32>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn modules)]
    pub type Modules<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        ModuleId,
        ModuleInfo<T::AccountId, BalanceOf<T>>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn validator_stakes)]
    pub type ValidatorStakes<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        ValidatorStake<T::AccountId, BalanceOf<T>>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn unbonding_info)]
    pub type UnbondingInfos<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        UnbondingInfo<T::AccountId, BalanceOf<T>, T::BlockNumber>,
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ModuleRegistered {
            module_id: ModuleId,
            owner: T::AccountId,
        },
        ModuleUpdated {
            module_id: ModuleId,
        },
        ModuleStateChanged {
            module_id: ModuleId,
            new_state: ModuleState,
        },
        ValidatorAdded {
            module_id: ModuleId,
            validator: T::AccountId,
        },
        ValidatorRemoved {
            module_id: ModuleId,
            validator: T::AccountId,
        },
        StakeAdded {
            account: T::AccountId,
            amount: BalanceOf<T>,
        },
        StakeRemoved {
            account: T::AccountId,
            amount: BalanceOf<T>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        ModuleNotFound,
        ModuleAlreadyExists,
        InvalidModuleId,
        InvalidModuleState,
        InvalidStakeAmount,
        InsufficientBalance,
        NotAuthorized,
        TooManyValidators,
        ValidatorNotFound,
        ValidatorAlreadyExists,
        UnbondingInProgress,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // Implementation will be added in subsequent updates
    }
}
