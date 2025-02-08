use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
use pallet_module_registrar::{types::{ModuleId, ModuleMetadata, ModuleState}, Error, Event};
use sp_runtime::DispatchError;
use pallet_balances::Pallet as Balances;

mod register_module {
    use super::*;

    fn setup_test_module() -> (u32, ModuleId, ModuleMetadata, u64) {
        let account = 1;
        let module_id = ModuleId::from(b"test_module".to_vec());
        let metadata = ModuleMetadata {
            name: b"Test Module".to_vec(),
            description: b"A test module".to_vec(),
            version: b"1.0.0".to_vec(),
        };
        let stake = 1_000_000;

        // Add balance for registration
        add_balance(account, stake);

        (account, module_id, metadata, stake)
    }

    fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
        frame_system::Pallet::<T>::assert_last_event(generic_event.into());
    }

    #[test]
    fn successfully_registers_new_module() {
        new_test_ext().execute_with(|| {
            let (account, module_id, metadata, stake) = setup_test_module();

            assert_ok!(ModuleRegistrarMod::register_module(
                get_origin(account),
                module_id.clone(),
                metadata.clone(),
                stake
            ));

            let info = ModuleRegistrarMod::module_info(&module_id).unwrap();
            assert_eq!(info.owner, account);
            assert_eq!(info.metadata, metadata);
            assert_eq!(info.stake, stake);
            assert_eq!(info.state, ModuleState::Active);

            // Verify event was emitted
            assert_last_event::<Test>(Event::ModuleRegistered { 
                owner: account,
                module_id: module_id.clone(),
                stake,
            }.into());

            // Verify balance was reserved
            assert_eq!(Balances::reserved_balance(account), stake);
        });
    }

    #[test]
    fn fails_to_register_duplicate_module() {
        new_test_ext().execute_with(|| {
            let (account, module_id, metadata, stake) = setup_test_module();
            
            // First registration
            assert_ok!(ModuleRegistrarMod::register_module(
                get_origin(account),
                module_id.clone(),
                metadata.clone(),
                stake
            ));

            // Attempt duplicate registration
            assert_noop!(
                ModuleRegistrarMod::register_module(
                    get_origin(2),
                    module_id,
                    metadata,
                    stake
                ),
                Error::<Test>::ModuleAlreadyRegistered
            );
        });
    }

    #[test]
    fn fails_when_insufficient_balance() {
        new_test_ext().execute_with(|| {
            let (account, module_id, metadata, stake) = setup_test_module();
            
            // Remove balance
            let _ = Balances::slash(&account, stake);

            assert_noop!(
                ModuleRegistrarMod::register_module(
                    get_origin(account),
                    module_id.clone(),
                    metadata.clone(),
                    stake
                ),
                Error::<Test>::InsufficientBalance
            );

            // Verify no module was registered
            assert!(ModuleRegistrarMod::module_info(&module_id).is_none());
        });
    }

    #[test]
    fn fails_with_invalid_metadata() {
        new_test_ext().execute_with(|| {
            let (account, module_id, _, stake) = setup_test_module();

            // Create metadata with empty name
            let invalid_metadata = ModuleMetadata {
                name: vec![],
                description: b"Description".to_vec(),
                version: b"1.0.0".to_vec(),
            };

            assert_noop!(
                ModuleRegistrarMod::register_module(
                    get_origin(account),
                    module_id.clone(),
                    invalid_metadata,
                    stake
                ),
                Error::<Test>::InvalidMetadata
            );
        });
    }
}

mod update_module {
    use super::*;

    fn setup_registered_module() -> (u32, ModuleId, ModuleMetadata, u64) {
        let account = 1;
        let module_id = ModuleId::from(b"test_module".to_vec());
        let metadata = ModuleMetadata {
            name: b"Test Module".to_vec(),
            description: b"A test module".to_vec(),
            version: b"1.0.0".to_vec(),
        };
        let stake = 1_000_000;

        // Setup and register module
        add_balance(account, stake);
        assert_ok!(ModuleRegistrarMod::register_module(
            get_origin(account),
            module_id.clone(),
            metadata.clone(),
            stake
        ));

        (account, module_id, metadata, stake)
    }

    #[test]
    fn successfully_updates_module() {
        new_test_ext().execute_with(|| {
            let (account, module_id, _, stake) = setup_registered_module();

            let new_metadata = ModuleMetadata {
                name: b"Updated Module".to_vec(),
                description: b"An updated test module".to_vec(),
                version: b"2.0.0".to_vec(),
            };

            assert_ok!(ModuleRegistrarMod::update_module(
                get_origin(account),
                module_id.clone(),
                new_metadata.clone(),
            ));

            // Verify module info was updated
            let info = ModuleRegistrarMod::module_info(&module_id).unwrap();
            assert_eq!(info.metadata, new_metadata);
            assert_eq!(info.owner, account);
            assert_eq!(info.stake, stake);
            assert_eq!(info.state, ModuleState::Active);

            // Verify event was emitted
            assert_last_event::<Test>(Event::ModuleUpdated { 
                owner: account,
                module_id: module_id.clone(),
            }.into());
        });
    }

    #[test]
    fn fails_for_non_existent_module() {
        new_test_ext().execute_with(|| {
            let (account, _, metadata, _) = setup_registered_module();
            let non_existent = ModuleId::from(b"non_existent".to_vec());

            assert_noop!(
                ModuleRegistrarMod::update_module(
                    get_origin(account),
                    non_existent.clone(),
                    metadata.clone(),
                ),
                Error::<Test>::ModuleNotFound
            );

            // Verify module was not created
            assert!(ModuleRegistrarMod::module_info(&non_existent).is_none());
        });
    }

    #[test]
    fn fails_for_non_owner() {
        new_test_ext().execute_with(|| {
            let (account, module_id, metadata, stake) = setup_registered_module();
            let non_owner = 2;

            // Add balance for non_owner
            add_balance(non_owner, stake);

            assert_noop!(
                ModuleRegistrarMod::update_module(
                    get_origin(non_owner),
                    module_id.clone(),
                    metadata.clone(),
                ),
                Error::<Test>::NotModuleOwner
            );

            // Verify module info remains unchanged
            let info = ModuleRegistrarMod::module_info(&module_id).unwrap();
            assert_eq!(info.owner, account);
            assert_eq!(info.metadata, metadata);
        });
    }

    #[test]
    fn fails_with_invalid_metadata() {
        new_test_ext().execute_with(|| {
            let (account, module_id, metadata, _) = setup_registered_module();

            // Create metadata with empty version
            let invalid_metadata = ModuleMetadata {
                name: b"Valid Name".to_vec(),
                description: b"Valid Description".to_vec(),
                version: vec![],
            };

            assert_noop!(
                ModuleRegistrarMod::update_module(
                    get_origin(account),
                    module_id.clone(),
                    invalid_metadata,
                ),
                Error::<Test>::InvalidMetadata
            );

            // Verify module info remains unchanged
            let info = ModuleRegistrarMod::module_info(&module_id).unwrap();
            assert_eq!(info.metadata, metadata);
        });
    }
}

mod remove_module {
    use super::*;

    fn setup_registered_module() -> (u32, ModuleId, ModuleMetadata, u64) {
        let account = 1;
        let module_id = ModuleId::from(b"test_module".to_vec());
        let metadata = ModuleMetadata {
            name: b"Test Module".to_vec(),
            description: b"A test module".to_vec(),
            version: b"1.0.0".to_vec(),
        };
        let stake = 1_000_000;

        // Setup and register module
        add_balance(account, stake);
        assert_ok!(ModuleRegistrarMod::register_module(
            get_origin(account),
            module_id.clone(),
            metadata.clone(),
            stake
        ));

        (account, module_id, metadata, stake)
    }

    #[test]
    fn successfully_removes_module() {
        new_test_ext().execute_with(|| {
            let (account, module_id, _, stake) = setup_registered_module();

            // Get initial reserved balance
            let initial_reserved = Balances::reserved_balance(account);
            assert_eq!(initial_reserved, stake);

            assert_ok!(ModuleRegistrarMod::remove_module(
                get_origin(account),
                module_id.clone(),
            ));

            // Verify module was removed
            assert!(ModuleRegistrarMod::module_info(&module_id).is_none());

            // Verify event was emitted
            assert_last_event::<Test>(Event::ModuleRemoved { 
                owner: account,
                module_id: module_id.clone(),
            }.into());

            // Verify stake was unreserved
            assert_eq!(Balances::reserved_balance(account), 0);
            assert_eq!(Balances::free_balance(account), initial_reserved);
        });
    }

    #[test]
    fn fails_for_non_existent_module() {
        new_test_ext().execute_with(|| {
            let (account, _, _, _) = setup_registered_module();
            let non_existent = ModuleId::from(b"non_existent".to_vec());

            assert_noop!(
                ModuleRegistrarMod::remove_module(
                    get_origin(account),
                    non_existent.clone(),
                ),
                Error::<Test>::ModuleNotFound
            );

            // Verify module still doesn't exist
            assert!(ModuleRegistrarMod::module_info(&non_existent).is_none());
        });
    }

    #[test]
    fn fails_for_non_owner() {
        new_test_ext().execute_with(|| {
            let (account, module_id, metadata, stake) = setup_registered_module();
            let non_owner = 2;

            // Add balance for non_owner
            add_balance(non_owner, stake);

            assert_noop!(
                ModuleRegistrarMod::remove_module(
                    get_origin(non_owner),
                    module_id.clone(),
                ),
                Error::<Test>::NotModuleOwner
            );

            // Verify module still exists and is unchanged
            let info = ModuleRegistrarMod::module_info(&module_id).unwrap();
            assert_eq!(info.owner, account);
            assert_eq!(info.metadata, metadata);
            assert_eq!(info.stake, stake);

            // Verify stake is still reserved
            assert_eq!(Balances::reserved_balance(account), stake);
        });
    }
}

mod stake_management {
    use super::*;

    fn setup_registered_module() -> (u32, ModuleId, ModuleMetadata, u64) {
        let (account, module_id, metadata, stake) = super::register_module::setup_test_module();
        
        assert_ok!(ModuleRegistrarMod::register_module(
            get_origin(account),
            module_id.clone(),
            metadata.clone(),
            stake
        ));

        (account, module_id, metadata, stake)
    }

    #[test]
    fn stake_is_locked_during_registration() {
        new_test_ext().execute_with(|| {
            let (account, _, _, stake) = setup_registered_module();
            
            // Verify stake is locked
            assert_eq!(Balances::usable_balance(account), 0);
            assert_eq!(Balances::total_balance(account), stake);
        });
    }

    #[test]
    fn stake_is_unlocked_after_removal() {
        new_test_ext().execute_with(|| {
            let (account, module_id, _, stake) = setup_registered_module();
            
            // Remove module
            assert_ok!(ModuleRegistrarMod::remove_module(
                get_origin(account),
                module_id
            ));

            // Verify stake is unlocked
            assert_eq!(Balances::usable_balance(account), stake);
            assert_eq!(Balances::total_balance(account), stake);
        });
    }

    #[test]
    fn cannot_transfer_locked_stake() {
        new_test_ext().execute_with(|| {
            let (account, _, _, stake) = setup_registered_module();
            let recipient = 2;

            // Attempt to transfer locked stake
            assert_noop!(
                Balances::transfer(get_origin(account), recipient, stake),
                pallet_balances::Error::<Test>::LiquidityRestrictions
            );
        });
    }
}

mod module_gaps {
    use super::*;

    #[test]
    fn tracks_module_gaps_correctly() {
        new_test_ext().execute_with(|| {
            let (account1, module_id1, metadata1, stake) = super::register_module::setup_test_module();
            let (account2, module_id2, metadata2, _) = super::register_module::setup_test_module();

            // Register first module
            assert_ok!(ModuleRegistrarMod::register_module(
                get_origin(account1),
                module_id1.clone(),
                metadata1,
                stake
            ));

            // Register second module
            assert_ok!(ModuleRegistrarMod::register_module(
                get_origin(account2),
                module_id2.clone(),
                metadata2,
                stake
            ));

            // Remove first module to create a gap
            assert_ok!(ModuleRegistrarMod::remove_module(
                get_origin(account1),
                module_id1
            ));

            // Verify gap is tracked
            let gaps = ModuleRegistrarMod::module_gaps();
            assert!(!gaps.is_empty());
        });
    }

    #[test]
    fn reuses_gaps_for_new_modules() {
        new_test_ext().execute_with(|| {
            let (account1, module_id1, metadata1, stake) = super::register_module::setup_test_module();
            let (account2, module_id2, metadata2, _) = super::register_module::setup_test_module();
            let (account3, module_id3, metadata3, _) = super::register_module::setup_test_module();

            // Register and remove first module
            assert_ok!(ModuleRegistrarMod::register_module(
                get_origin(account1),
                module_id1.clone(),
                metadata1,
                stake
            ));
            assert_ok!(ModuleRegistrarMod::remove_module(
                get_origin(account1),
                module_id1
            ));

            // Register second module
            assert_ok!(ModuleRegistrarMod::register_module(
                get_origin(account2),
                module_id2,
                metadata2,
                stake
            ));

            // Register third module in the gap
            assert_ok!(ModuleRegistrarMod::register_module(
                get_origin(account3),
                module_id3,
                metadata3,
                stake
            ));

            // Verify gap was reused
            let gaps = ModuleRegistrarMod::module_gaps();
            assert!(gaps.is_empty());
        });
    }
}

mod change_module_state {
    use super::*;

    fn setup_registered_module() -> (u32, ModuleId, ModuleMetadata, u64) {
        let account = 1;
        let module_id = ModuleId::from(b"test_module".to_vec());
        let metadata = ModuleMetadata {
            name: b"Test Module".to_vec(),
            description: b"A test module".to_vec(),
            version: b"1.0.0".to_vec(),
        };
        let stake = 1_000_000;

        // Setup and register module
        add_balance(account, stake);
        assert_ok!(ModuleRegistrarMod::register_module(
            get_origin(account),
            module_id.clone(),
            metadata.clone(),
            stake
        ));

        (account, module_id, metadata, stake)
    }

    #[test]
    fn successfully_changes_state() {
        new_test_ext().execute_with(|| {
            let (account, module_id, metadata, stake) = setup_registered_module();

            // Verify initial state
            let initial_info = ModuleRegistrarMod::module_info(&module_id).unwrap();
            assert_eq!(initial_info.state, ModuleState::Active);

            // Change state to Inactive
            assert_ok!(ModuleRegistrarMod::change_module_state(
                get_origin(account),
                module_id.clone(),
                ModuleState::Inactive,
            ));

            // Verify state was changed
            let info = ModuleRegistrarMod::module_info(&module_id).unwrap();
            assert_eq!(info.state, ModuleState::Inactive);
            assert_eq!(info.owner, account);
            assert_eq!(info.metadata, metadata);
            assert_eq!(info.stake, stake);

            // Verify event was emitted
            assert_last_event::<Test>(Event::ModuleStateChanged { 
                owner: account,
                module_id: module_id.clone(),
                state: ModuleState::Inactive,
            }.into());

            // Change back to Active
            assert_ok!(ModuleRegistrarMod::change_module_state(
                get_origin(account),
                module_id.clone(),
                ModuleState::Active,
            ));

            // Verify state was changed back
            let final_info = ModuleRegistrarMod::module_info(&module_id).unwrap();
            assert_eq!(final_info.state, ModuleState::Active);

            // Verify event was emitted
            assert_last_event::<Test>(Event::ModuleStateChanged { 
                owner: account,
                module_id: module_id.clone(),
                state: ModuleState::Active,
            }.into());
        });
    }

    #[test]
    fn fails_for_non_existent_module() {
        new_test_ext().execute_with(|| {
            let (account, _, _, _) = setup_registered_module();
            let non_existent = ModuleId::from(b"non_existent".to_vec());

            assert_noop!(
                ModuleRegistrarMod::change_module_state(
                    get_origin(account),
                    non_existent.clone(),
                    ModuleState::Inactive,
                ),
                Error::<Test>::ModuleNotFound
            );

            // Verify module still doesn't exist
            assert!(ModuleRegistrarMod::module_info(&non_existent).is_none());
        });
    }

    #[test]
    fn fails_for_non_owner() {
        new_test_ext().execute_with(|| {
            let (account, module_id, metadata, stake) = setup_registered_module();
            let non_owner = 2;

            // Add balance for non_owner
            add_balance(non_owner, stake);

            assert_noop!(
                ModuleRegistrarMod::change_module_state(
                    get_origin(non_owner),
                    module_id.clone(),
                    ModuleState::Inactive,
                ),
                Error::<Test>::NotModuleOwner
            );

            // Verify module info remains unchanged
            let info = ModuleRegistrarMod::module_info(&module_id).unwrap();
            assert_eq!(info.owner, account);
            assert_eq!(info.metadata, metadata);
            assert_eq!(info.stake, stake);
            assert_eq!(info.state, ModuleState::Active);
        });
    }

    #[test]
    fn fails_for_same_state() {
        new_test_ext().execute_with(|| {
            let (account, module_id, _, _) = setup_registered_module();

            assert_noop!(
                ModuleRegistrarMod::change_module_state(
                    get_origin(account),
                    module_id.clone(),
                    ModuleState::Active,  // Module is already active
                ),
                Error::<Test>::NoStateChange
            );
        });
    }
}
