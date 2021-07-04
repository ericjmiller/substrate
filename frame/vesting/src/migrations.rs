//! Storage migrations for the vesting pallet.

use super::*;

// Migration from single schedule to multiple schedules.
pub(crate) mod v1 {
	use super::*;

	#[cfg(feature = "try-runtime")]
	pub(crate) fn pre_migrate<T: Config>() -> Result<(), &'static str> {
		assert!(StorageVersion::<T>::get() == Releases::V0, "Storage version too high.");

		// TODO: figure out how to iterate over old type of values
		// Vesting::<T>::translate::<VestingInfo<BalanceOf<T>, T::BlockNumber>, _>(
		// 	|_key, vesting_info| {
		// 		assert!(vesting_info.per_block() > Zero::zero(), "A schedule with per_block of 0 exists");
		// 		Some(vesting_info)
		// });

		Ok(())
	}
	/// Migrate from single schedule to multi schedule storage
	pub(crate) fn migrate<T: Config>() -> Weight {
		let mut reads_writes = 0;

		Vesting::<T>::translate::<VestingInfo<BalanceOf<T>, T::BlockNumber>, _>(
			|_key, vesting_info| {
				reads_writes += 1;
				let v: Option<
					BoundedVec<VestingInfo<BalanceOf<T>, T::BlockNumber>, T::MaxVestingSchedules>,
				> = vec![vesting_info].try_into().ok();

				v
			},
		);

		T::DbWeight::get().reads_writes(reads_writes, reads_writes)
	}

	#[cfg(feature = "try-runtime")]
	pub(crate) fn post_migrate<T: Config>() -> Result<(), &'static str> {
		assert_eq!(StorageVersion::<T>::get(), Releases::V1);

		for (key, schedules) in Vesting::<T>::iter() {
			log::debug!(target: LOG_TARGET, "[post_migrate] Vesting key {}", key);
			// Assert the new bound vec respects size.
			assert!(schedules.len() > 0, "A bounded vec with no items was created.");
			assert!(schedules.len() <= T::MaxVestingSchedules::get() as usize, "A bounded vec with too many items was created.");

			for s in schedules {
				// Check for infinite schedules
				assert!(s.per_block() > Zero::zero(), "A schedule with per_block of 0 exists");
			}
		}

		Ok(())
	}
}