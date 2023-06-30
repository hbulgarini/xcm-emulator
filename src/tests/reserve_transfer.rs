use crate::Kusama;
use crate::*;
use std::sync::Once;
use xcm_emulator::{Parachain as Para, RelayChain as Relay};
use frame_support::traits::PalletInfoAccess;

#[test]
fn reserve_transfer_native_asset_from_relay_to_assets() {
    // Init tests variables
    let amount = KUSAMA_ED * 1000;
    let relay_sender_balance_before = Kusama::account_data_of(KusamaSender::get()).free;
    let para_receiver_balance_before = Statemine::account_data_of(StatemineReceiver::get()).free;

    let origin = <Kusama as Relay>::RuntimeOrigin::signed(KusamaSender::get());
    let assets_para_destination: VersionedMultiLocation =
        Kusama::child_location_of(Statemine::para_id()).into();
    let beneficiary: VersionedMultiLocation = AccountId32 {
        network: None,
        id: StatemineReceiver::get().into(),
    }
    .into();
    let native_assets: VersionedMultiAssets = (Here, amount).into();
    let fee_asset_item = 0;
    let weight_limit = WeightLimit::Unlimited;

    // Send XCM message from Relay Chain
    Kusama::execute_with(|| {
        assert_ok!(
            <Kusama as KusamaPallet>::XcmPallet::limited_reserve_transfer_assets(
                origin,
                bx!(assets_para_destination),
                bx!(beneficiary),
                bx!(native_assets),
                fee_asset_item,
                weight_limit,
            )
        );

        type RuntimeEvent = <Kusama as Relay>::RuntimeEvent;

        assert_expected_events!(
            Kusama,
            vec![
                RuntimeEvent::XcmPallet(pallet_xcm::Event::Attempted(Outcome::Complete(weight))) => {
                    weight: weight_within_threshold((REF_TIME_THRESHOLD, PROOF_SIZE_THRESHOLD), Weight::from_parts(754_244_000, 0), *weight),
                },
            ]
        );
    });

    // Receive XCM message in Assets Parachain
    Statemine::execute_with(|| {
        type RuntimeEvent = <Statemine as Para>::RuntimeEvent;

        assert_expected_events!(
            Statemine,
            vec![
                RuntimeEvent::DmpQueue(cumulus_pallet_dmp_queue::Event::ExecutedDownward {
                    outcome: Outcome::Incomplete(_, Error::UntrustedReserveLocation),
                    ..
                }) => {},
            ]
        );
    });

    // Check if balances are updated accordingly in Relay Chain and Assets Parachain
    let relay_sender_balance_after = Kusama::account_data_of(KusamaSender::get()).free;
    let para_sender_balance_after = Statemine::account_data_of(StatemineReceiver::get()).free;

    assert_eq!(
        relay_sender_balance_before - amount,
        relay_sender_balance_after
    );
    assert_eq!(para_sender_balance_after, para_receiver_balance_before);
}

#[test]
fn reserve_transfer_asset_from_statemine_parachain_to_penpal_parachain() {
    // Init tests variables
    const ASSET_ID: u32 = 1984;
    const AMOUNT: u128 = 20_000_000_000;
    const MINT_AMOUNT: u128 = 100_000_000_000_000;
    let root_statemine = <Statemine as Para>::RuntimeOrigin::root();
    let penpal_root = <PenpalKusama as Para>::RuntimeOrigin::root();

    let statemine_remote: MultiLocation = MultiLocation {
        parents: 1,
        interior: X1(Parachain(Statemine::para_id().into())),
    };
    let penpal_remote: MultiLocation = MultiLocation {
        parents: 1,
        interior: X1(Parachain(PenpalKusama::para_id().into())),
    };
    let statemine_origin = <Statemine as Para>::RuntimeOrigin::signed(StatemineSender::get());
    let beneficiary: VersionedMultiLocation = AccountId32 {
        network: None,
        id: PenpalKusamaReceiver::get().into(),
    }
    .into();
    let asset_to_transfer: VersionedMultiAssets = (
        X2(PalletInstance(50.into()), GeneralIndex(ASSET_ID as u128)),
        AMOUNT,
    )
        .into();
    let fee_asset_item = 0;
    let weight_limit = WeightLimit::Unlimited;

    PenpalKusama::execute_with(|| {
        type RuntimeEvent = <PenpalKusama as Para>::RuntimeEvent;

        assert_ok!(<PenpalKusama as PenpalKusamaPallet>::PolkadotXcm::force_xcm_version(
            penpal_root.clone(),
            bx!(statemine_remote),
            XCM_V3
        ));

        assert_ok!(<PenpalKusama as PenpalKusamaPallet>::Assets::force_create(
            penpal_root.clone(),
            ASSET_ID.into(),
            PenpalKusamaSender::get().into(),
            true,
            10000u128.into(),
        ));
    });

    Statemine::execute_with(|| {
        assert_ok!(<Statemine as StateminePallet>::Assets::force_create(
            root_statemine.clone(),
            ASSET_ID.into(),
            StatemineSender::get().into(),
            true,
            10000u128.into(),
        ));

        assert_ok!(<Statemine as StateminePallet>::Assets::mint(
            <Statemine as Para>::RuntimeOrigin::signed(StatemineSender::get()),
            ASSET_ID.into(),
            StatemineSender::get().into(),
            MINT_AMOUNT.into(),
        ));

        assert_ok!(
            <Statemine as StateminePallet>::PolkadotXcm::force_xcm_version(
                root_statemine,
                bx!(penpal_remote),
                XCM_V3
            )
        );

        assert_ok!(
            <Statemine as StateminePallet>::PolkadotXcm::limited_reserve_transfer_assets(
                <Statemine as Para>::RuntimeOrigin::signed(StatemineSender::get()),
                bx!(penpal_remote.into()),
                bx!(beneficiary),
                bx!(asset_to_transfer),
                fee_asset_item,
                weight_limit,
            )
        );
    });

    PenpalKusama::execute_with(|| {
        type RuntimeEvent = <PenpalKusama as Para>::RuntimeEvent;

        let balance =
            <PenpalKusama as PenpalKusamaPallet>::Assets::balance(ASSET_ID.into(), PenpalKusamaReceiver::get());
        println!("balance: {:?}", balance);
    });
}
