//! # XCM Support Module.
//!
//! ## Overview
//!
//! The XCM support module provides supporting traits, types and
//! implementations, to support cross-chain message(XCM) integration with ORML
//! modules.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use frame_support::dispatch::{DispatchError, DispatchResult};
use frame_support::{decl_error, decl_event, decl_module, sp_runtime::traits::Hash, traits::EnsureOrigin, weights::Weight};

use sp_runtime::traits::{CheckedConversion, Convert};
use sp_std::{convert::TryFrom, marker::PhantomData, prelude::*};

use xcm::v0::{MultiAsset, MultiLocation, Xcm};
use xcm_executor::traits::{FilterAssetLocation, MatchesFungible};
use xcm_builder::EnsureXcmOrigin;
use xcm::{
	v0::{Error as XcmError, Junction, SendXcm, Outcome},
	VersionedXcm,
};
use frame_system::pallet_prelude::*;
use cumulus_primitives_core::{
	DmpMessageHandler, InboundDownwardMessage,
	InboundHrmpMessage, OutboundHrmpMessage, ParaId, UpwardMessageSender,
};




use orml_traits::location::Reserve;

pub use currency_adapter::MultiCurrencyAdapter;

mod currency_adapter;

mod tests;

/// Type of XCM message executor.
pub trait ExecuteXcm<Call> {
	/// Execute some XCM `message` from `origin` using no more than `weight_limit` weight. The weight limit is
	/// a basic hard-limit and the implementation may place further restrictions or requirements on weight and
	/// other aspects.
	fn execute_xcm(origin: MultiLocation, message: Xcm<Call>, weight_limit: Weight) -> Outcome;
}



/// A `MatchesFungible` implementation. It matches concrete fungible assets
/// whose `id` could be converted into `CurrencyId`.
pub struct IsNativeConcrete<CurrencyId, CurrencyIdConvert>(PhantomData<(CurrencyId, CurrencyIdConvert)>);
impl<CurrencyId, CurrencyIdConvert, Amount> MatchesFungible<Amount> for IsNativeConcrete<CurrencyId, CurrencyIdConvert>
where
	CurrencyIdConvert: Convert<MultiLocation, Option<CurrencyId>>,
	Amount: TryFrom<u128>,
{
	fn matches_fungible(a: &MultiAsset) -> Option<Amount> {
		if let MultiAsset::ConcreteFungible { id, amount } = a {
			if CurrencyIdConvert::convert(id.clone()).is_some() {
				return CheckedConversion::checked_from(*amount);
			}
		}
		None
	}
}

/// A `FilterAssetLocation` implementation. Filters multi native assets whose
/// reserve is same with `origin`.
pub struct MultiNativeAsset;
impl FilterAssetLocation for MultiNativeAsset {
	fn filter_asset_location(asset: &MultiAsset, origin: &MultiLocation) -> bool {
		if let Some(ref reserve) = asset.reserve() {
			if reserve == origin {
				return true;
			}
		}
		false
	}
}

/// Handlers unknown asset deposit and withdraw.
pub trait UnknownAsset {
	/// Deposit unknown asset.
	fn deposit(asset: &MultiAsset, to: &MultiLocation) -> DispatchResult;

	/// Withdraw unknown asset.
	fn withdraw(asset: &MultiAsset, from: &MultiLocation) -> DispatchResult;
}

const NO_UNKNOWN_ASSET_IMPL: &str = "NoUnknownAssetImpl";

impl UnknownAsset for () {
	fn deposit(_asset: &MultiAsset, _to: &MultiLocation) -> DispatchResult {
		Err(DispatchError::Other(NO_UNKNOWN_ASSET_IMPL))
	}
	fn withdraw(_asset: &MultiAsset, _from: &MultiLocation) -> DispatchResult {
		Err(DispatchError::Other(NO_UNKNOWN_ASSET_IMPL))
	}
}
