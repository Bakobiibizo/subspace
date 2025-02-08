use crate::{mock::*, types::{ModuleId, ModuleMetadata, ModuleState}, Error, Event, ModuleGaps, ModuleInfo};
use frame_support::traits::Currency;
use frame_support::{assert_noop, assert_ok, BoundedVec};
use sp_core::H256;
use pallet_balances::Pallet as Balances;

fn setup_test_module() -> (u64, ModuleId, ModuleMetadata, u64) {
    let account = 1;
    let module_id = ModuleId::try_from(b"test_module".to_vec()).unwrap();
    let metadata = ModuleMetadata {
        version_major: 1,
        version_minor: 0,
        version_patch: 0,
        repo_url: BoundedVec::try_from(b"https://example.com/repo".to_vec()).unwrap(),
        build_script_url: BoundedVec::try_from(b"https://example.com/build".to_vec()).unwrap(),
        installer_script_url: BoundedVec::try_from(b"https://example.com/install".to_vec()).unwrap(),
        ipfs_hash: H256::zero(),
    };
    let stake = 100;
    add_balance(account, stake);
    (account, module_id, metadata, stake)
}

fn add_balance(account: u64, amount: u64) {
    let _ = <Balances::<Test> as Currency<u64>>::make_free_balance_be(&account, amount);
}

fn assert_last_event(generic_event: RuntimeEvent) {
    System::assert_last_event(generic_event);
}

#[test]
fn successfully_registers_new_module() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let (account, module_id, metadata, stake) = setup_test_module();

        assert_ok!(ModuleRegistrar::register_module(
            get_origin(account),
            module_id.clone(),
            metadata.clone(),
            stake
        ));

        let info = ModuleRegistrar::get_module(&module_id).unwrap();
        assert_eq!(info.owner, account);
        assert_eq!(info.metadata, metadata);
        assert_eq!(info.stake, stake);
        assert_eq!(info.state, ModuleState::Pending);

        // Verify event was emitted
        assert_last_event(Event::ModuleRegistered(module_id.clone(), account).into());
    });
}

#[test]
fn fails_to_register_duplicate_module() {
    new_test_ext().execute_with(|| {
        let (account, module_id, metadata, stake) = setup_test_module();

        assert_ok!(ModuleRegistrar::register_module(
            get_origin(account),
            module_id.clone(),
            metadata.clone(),
            stake
        ));

        assert_noop!(
            ModuleRegistrar::register_module(
                get_origin(account),
                module_id,
                metadata,
                stake
            ),
            Error::<Test>::ModuleAlreadyExists
        );
    });
}

#[test]
fn fails_when_insufficient_balance() {
    new_test_ext().execute_with(|| {
        let (account, module_id, metadata, stake) = setup_test_module();
        let insufficient_stake = stake + 1;

        assert_noop!(
            ModuleRegistrar::register_module(
                get_origin(account),
                module_id,
                metadata,
                insufficient_stake
            ),
            Error::<Test>::InsufficientBalance
        );
    });
}

#[test]
fn successfully_updates_module() {
    new_test_ext().execute_with(|| {
        let (account, module_id, metadata, stake) = setup_test_module();

        assert_ok!(ModuleRegistrar::register_module(
            get_origin(account),
            module_id.clone(),
            metadata,
            stake
        ));

        let new_metadata = ModuleMetadata {
            version_major: 2,
            version_minor: 0,
            version_patch: 0,
            repo_url: BoundedVec::try_from(b"https://example.com/repo2".to_vec()).unwrap(),
            build_script_url: BoundedVec::try_from(b"https://example.com/build2".to_vec()).unwrap(),
            installer_script_url: BoundedVec::try_from(b"https://example.com/install2".to_vec()).unwrap(),
            ipfs_hash: H256::zero(),
        };

        assert_ok!(ModuleRegistrar::update_module(
            get_origin(account),
            module_id.clone(),
            new_metadata.clone()
        ));

        let info = ModuleRegistrar::get_module(&module_id).unwrap();
        assert_eq!(info.metadata, new_metadata);
    });
}

#[test]
fn successfully_removes_module() {
    new_test_ext().execute_with(|| {
        let (account, module_id, metadata, stake) = setup_test_module();

        assert_ok!(ModuleRegistrar::register_module(
            get_origin(account),
            module_id.clone(),
            metadata,
            stake
        ));

        assert_ok!(ModuleRegistrar::remove_module(
            get_origin(account),
            module_id.clone()
        ));

        assert!(ModuleRegistrar::get_module(&module_id).is_none());
    });
}

#[test]
fn successfully_changes_state() {
    new_test_ext().execute_with(|| {
        let (account, module_id, metadata, stake) = setup_test_module();

        // Register module
        assert_ok!(ModuleRegistrar::register_module(
            get_origin(account),
            module_id.clone(),
            metadata,
            stake
        ));

        // First activate the module
        assert_ok!(ModuleRegistrar::change_module_state(
            get_origin(account),
            module_id.clone(),
            ModuleState::Active
        ));

        // Then suspend it
        assert_ok!(ModuleRegistrar::change_module_state(
            get_origin(account),
            module_id.clone(),
            ModuleState::Suspended
        ));

        let info = ModuleRegistrar::get_module(&module_id).unwrap();
        assert_eq!(info.state, ModuleState::Suspended);

        // Change state back to Active
        assert_ok!(ModuleRegistrar::change_module_state(
            get_origin(account),
            module_id.clone(),
            ModuleState::Active
        ));

        let info = ModuleRegistrar::module_info(&module_id).unwrap();
        assert_eq!(info.state, ModuleState::Active);
    });
}

#[test]
fn stake_is_locked_during_registration() {
    new_test_ext().execute_with(|| {
        let (account, module_id, metadata, stake) = setup_test_module();

        assert_ok!(ModuleRegistrar::register_module(
            get_origin(account),
            module_id,
            metadata,
            stake
        ));

        assert_eq!(Balances::<Test>::free_balance(&account), stake);
        assert_eq!(Balances::<Test>::usable_balance(&account), 0);
    });
}

#[test]
fn stake_is_unlocked_after_removal() {
    new_test_ext().execute_with(|| {
        let (account, module_id, metadata, stake) = setup_test_module();

        assert_ok!(ModuleRegistrar::register_module(
            get_origin(account),
            module_id.clone(),
            metadata,
            stake
        ));

        assert_ok!(ModuleRegistrar::remove_module(
            get_origin(account),
            module_id
        ));

        assert_eq!(Balances::<Test>::free_balance(&account), stake);
        assert_eq!(Balances::<Test>::usable_balance(&account), stake);
    });
}

#[test]
fn tracks_module_gaps_correctly() {
    new_test_ext().execute_with(|| {
        let (account, module_id, metadata, stake) = setup_test_module();

        // Register module
        assert_ok!(ModuleRegistrar::register_module(
            get_origin(account),
            module_id.clone(),
            metadata.clone(),
            stake
        ));

        // Remove module
        assert_ok!(ModuleRegistrar::remove_module(
            get_origin(account),
            module_id.clone()
        ));

        // Verify gap was created
        let gaps = ModuleGaps::<Test>::get();
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0], module_id);
    });
}

#[test]
fn reuses_gaps_for_new_modules() {
    new_test_ext().execute_with(|| {
        let (account1, module_id1, metadata1, stake) = setup_test_module();
        let (_account2, _module_id2, _metadata2, _) = setup_test_module();
        let (account3, _module_id3, metadata3, _) = setup_test_module();

        // Register and remove first module
        assert_ok!(ModuleRegistrar::register_module(
            get_origin(account1),
            module_id1.clone(),
            metadata1,
            stake
        ));
        let module_id1_clone = module_id1.clone();
        assert_ok!(ModuleRegistrar::remove_module(
            get_origin(account1),
            module_id1_clone
        ));

        // Debug: Check state after removal
        let gaps = ModuleGaps::<Test>::get();
        println!("Gaps after removal: {:?}", gaps);
        let exists = ModuleInfo::<Test>::contains_key(&module_id1);
        println!("Module exists after removal: {}", exists);

        // Debug: Check gaps list before second registration
        let gaps = ModuleGaps::<Test>::get();
        println!("Gaps before second registration: {:?}", gaps);
        println!("Trying to register module_id1: {:?}", module_id1);
        println!("Module exists before second registration: {}", ModuleInfo::<Test>::contains_key(&module_id1));

        // Register third module with same ID as first (should reuse gap)
        assert_ok!(ModuleRegistrar::register_module(
            get_origin(account3),
            module_id1.clone(),
            metadata3.clone(),
            stake
        ));

        // Debug: Check state after reusing gap
        let gaps = ModuleGaps::<Test>::get();
        println!("Gaps after reusing gap: {:?}", gaps);
        println!("Module exists after reusing gap: {}", ModuleInfo::<Test>::contains_key(&module_id1));

        // Verify gap was reused
        assert!(gaps.is_empty());
        assert!(ModuleInfo::<Test>::contains_key(&module_id1));
    });
}
