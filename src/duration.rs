// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Temporal quantification

use core::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};
use core::time::Duration as StdDuration;
use core::{fmt, i64};
#[cfg(feature = "std")]
use std::error::Error;

#[cfg(any(feature = "rkyv", feature = "rkyv-16", feature = "rkyv-32", feature = "rkyv-64"))]
use rkyv::{Archive, Deserialize, Serialize};

/// The number of nanoseconds in a microsecond.
const NANOS_PER_MICRO: i32 = 1000;
/// The number of nanoseconds in a millisecond.
const NANOS_PER_MILLI: i32 = 1_000_000;
/// The number of nanoseconds in seconds.
pub(crate) const NANOS_PER_SEC: i32 = 1_000_000_000;
/// The number of microseconds per second.
const MICROS_PER_SEC: i64 = 1_000_000;
/// The number of milliseconds per second.
const MILLIS_PER_SEC: i64 = 1000;
/// The number of seconds in a minute.
const SECS_PER_MINUTE: i64 = 60;
/// The number of seconds in an hour.
const SECS_PER_HOUR: i64 = 3600;
/// The number of (non-leap) seconds in days.
const SECS_PER_DAY: i64 = 86_400;
/// The number of (non-leap) seconds in a week.
const SECS_PER_WEEK: i64 = 604_800;

macro_rules! try_opt {
    ($e:expr) => {
        match $e {
            Some(v) => v,
            None => return None,
        }
    };
}

/// ISO 8601 time duration with nanosecond precision.
///
/// This also allows for the negative duration; see individual methods for details.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
#[cfg_attr(
    any(feature = "rkyv", feature = "rkyv-16", feature = "rkyv-32", feature = "rkyv-64"),
    derive(Archive, Deserialize, Serialize),
    archive(compare(PartialEq, PartialOrd)),
    archive_attr(derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash))
)]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct Duration {
    secs: i64,
    nanos: i32, // Always 0 <= nanos < NANOS_PER_SEC
}

/// The minimum possible `Duration`: `-i64::MAX` milliseconds.
pub(crate) const MIN: Duration = Duration {
    secs: -i64::MAX / MILLIS_PER_SEC - 1,
    nanos: NANOS_PER_SEC + (-i64::MAX % MILLIS_PER_SEC) as i32 * NANOS_PER_MILLI,
};

/// The maximum possible `Duration`: `i64::MAX` milliseconds.
pub(crate) const MAX: Duration = Duration {
    secs: i64::MAX / MILLIS_PER_SEC,
    nanos: (i64::MAX % MILLIS_PER_SEC) as i32 * NANOS_PER_MILLI,
};

impl Duration {
    /// Makes a new `Duration` with given number of weeks.
    /// Equivalent to `Duration::seconds(weeks * 7 * 24 * 60 * 60)` with overflow checks.
    /// Panics when the duration is out of bounds.
    #[inline]
    #[must_use]
    pub fn weeks(weeks: i64) -> Duration {
        Duration::try_weeks(weeks).expect("Duration::weeks out of bounds")
    }

    /// Makes a new `Duration` with given number of weeks.
    /// Equivalent to `Duration::seconds(weeks * 7 * 24 * 60 * 60)` with overflow checks.
    /// Returns `None` when the duration is out of bounds.
    #[inline]
    pub fn try_weeks(weeks: i64) -> Option<Duration> {
        weeks.checked_mul(SECS_PER_WEEK).and_then(Duration::try_seconds)
    }

    /// Makes a new `Duration` with given number of days.
    /// Equivalent to `Duration::seconds(days * 24 * 60 * 60)` with overflow checks.
    /// Panics when the duration is out of bounds.
    #[inline]
    #[must_use]
    pub fn days(days: i64) -> Duration {
        Duration::try_days(days).expect("Duration::days out of bounds")
    }

    /// Makes a new `Duration` with given number of days.
    /// Equivalent to `Duration::seconds(days * 24 * 60 * 60)` with overflow checks.
    /// Returns `None` when the duration is out of bounds.
    #[inline]
    pub fn try_days(days: i64) -> Option<Duration> {
        days.checked_mul(SECS_PER_DAY).and_then(Duration::try_seconds)
    }

    /// Makes a new `Duration` with given number of hours.
    /// Equivalent to `Duration::seconds(hours * 60 * 60)` with overflow checks.
    /// Panics when the duration is out of bounds.
    #[inline]
    #[must_use]
    pub fn hours(hours: i64) -> Duration {
        Duration::try_hours(hours).expect("Duration::hours out of bounds")
    }

    /// Makes a new `Duration` with given number of hours.
    /// Equivalent to `Duration::seconds(hours * 60 * 60)` with overflow checks.
    /// Returns `None` when the duration is out of bounds.
    #[inline]
    pub fn try_hours(hours: i64) -> Option<Duration> {
        hours.checked_mul(SECS_PER_HOUR).and_then(Duration::try_seconds)
    }

    /// Makes a new `Duration` with given number of minutes.
    /// Equivalent to `Duration::seconds(minutes * 60)` with overflow checks.
    /// Panics when the duration is out of bounds.
    #[inline]
    #[must_use]
    pub fn minutes(minutes: i64) -> Duration {
        Duration::try_minutes(minutes).expect("Duration::minutes out of bounds")
    }

    /// Makes a new `Duration` with given number of minutes.
    /// Equivalent to `Duration::seconds(minutes * 60)` with overflow checks.
    /// Returns `None` when the duration is out of bounds.
    #[inline]
    pub fn try_minutes(minutes: i64) -> Option<Duration> {
        minutes.checked_mul(SECS_PER_MINUTE).and_then(Duration::try_seconds)
    }

    /// Makes a new `Duration` with given number of seconds.
    /// Panics when the duration is more than `i64::MAX` milliseconds
    /// or less than `-i64::MAX` milliseconds.
    #[inline]
    #[must_use]
    pub fn seconds(seconds: i64) -> Duration {
        Duration::try_seconds(seconds).expect("Duration::seconds out of bounds")
    }

    /// Makes a new `Duration` with given number of seconds.
    /// Returns `None` when the duration is more than `i64::MAX` milliseconds
    /// or less than `-i64::MAX` milliseconds.
    #[inline]
    pub fn try_seconds(seconds: i64) -> Option<Duration> {
        let d = Duration { secs: seconds, nanos: 0 };
        if d < MIN || d > MAX {
            return None;
        }
        Some(d)
    }

    /// Makes a new `Duration` with given number of milliseconds.
    #[inline]
    pub const fn milliseconds(milliseconds: i64) -> Duration {
        let (secs, millis) = div_mod_floor_64(milliseconds, MILLIS_PER_SEC);
        let nanos = millis as i32 * NANOS_PER_MILLI;
        Duration { secs, nanos }
    }

    /// Makes a new `Duration` with given number of microseconds.
    #[inline]
    pub const fn microseconds(microseconds: i64) -> Duration {
        let (secs, micros) = div_mod_floor_64(microseconds, MICROS_PER_SEC);
        let nanos = micros as i32 * NANOS_PER_MICRO;
        Duration { secs, nanos }
    }

    /// Makes a new `Duration` with given number of nanoseconds.
    #[inline]
    pub const fn nanoseconds(nanos: i64) -> Duration {
        let (secs, nanos) = div_mod_floor_64(nanos, NANOS_PER_SEC as i64);
        Duration { secs, nanos: nanos as i32 }
    }

    /// Returns the total number of whole weeks in the duration.
    #[inline]
    pub const fn num_weeks(&self) -> i64 {
        self.num_days() / 7
    }

    /// Returns the total number of whole days in the duration.
    pub const fn num_days(&self) -> i64 {
        self.num_seconds() / SECS_PER_DAY
    }

    /// Returns the total number of whole hours in the duration.
    #[inline]
    pub const fn num_hours(&self) -> i64 {
        self.num_seconds() / SECS_PER_HOUR
    }

    /// Returns the total number of whole minutes in the duration.
    #[inline]
    pub const fn num_minutes(&self) -> i64 {
        self.num_seconds() / SECS_PER_MINUTE
    }

    /// Returns the total number of whole seconds in the duration.
    pub const fn num_seconds(&self) -> i64 {
        // If secs is negative, nanos should be subtracted from the duration.
        if self.secs < 0 && self.nanos > 0 {
            self.secs + 1
        } else {
            self.secs
        }
    }

    /// Returns the number of nanoseconds such that
    /// `subsec_nanos() + num_seconds() * NANOS_PER_SEC` is the total number of
    /// nanoseconds in the duration.
    pub const fn subsec_nanos(&self) -> i32 {
        if self.secs < 0 && self.nanos > 0 {
            self.nanos - NANOS_PER_SEC
        } else {
            self.nanos
        }
    }

    /// Returns the total number of whole milliseconds in the duration,
    pub const fn num_milliseconds(&self) -> i64 {
        // A proper Duration will not overflow, because MIN and MAX are defined
        // such that the range is exactly i64 milliseconds.
        let secs_part = self.num_seconds() * MILLIS_PER_SEC;
        let nanos_part = self.subsec_nanos() / NANOS_PER_MILLI;
        secs_part + nanos_part as i64
    }

    /// Returns the total number of whole microseconds in the duration,
    /// or `None` on overflow (exceeding 2^63 microseconds in either direction).
    pub const fn num_microseconds(&self) -> Option<i64> {
        let secs_part = try_opt!(self.num_seconds().checked_mul(MICROS_PER_SEC));
        let nanos_part = self.subsec_nanos() / NANOS_PER_MICRO;
        secs_part.checked_add(nanos_part as i64)
    }

    /// Returns the total number of whole nanoseconds in the duration,
    /// or `None` on overflow (exceeding 2^63 nanoseconds in either direction).
    pub const fn num_nanoseconds(&self) -> Option<i64> {
        let secs_part = try_opt!(self.num_seconds().checked_mul(NANOS_PER_SEC as i64));
        let nanos_part = self.subsec_nanos();
        secs_part.checked_add(nanos_part as i64)
    }

    /// Add two durations, returning `None` if overflow occurred.
    #[must_use]
    pub fn checked_add(&self, rhs: &Duration) -> Option<Duration> {
        let mut secs = try_opt!(self.secs.checked_add(rhs.secs));
        let mut nanos = self.nanos + rhs.nanos;
        if nanos >= NANOS_PER_SEC {
            nanos -= NANOS_PER_SEC;
            secs = try_opt!(secs.checked_add(1));
        }
        let d = Duration { secs, nanos };
        // Even if d is within the bounds of i64 seconds,
        // it might still overflow i64 milliseconds.
        if d < MIN || d > MAX {
            None
        } else {
            Some(d)
        }
    }

    /// Subtract two durations, returning `None` if overflow occurred.
    #[must_use]
    pub fn checked_sub(&self, rhs: &Duration) -> Option<Duration> {
        let mut secs = try_opt!(self.secs.checked_sub(rhs.secs));
        let mut nanos = self.nanos - rhs.nanos;
        if nanos < 0 {
            nanos += NANOS_PER_SEC;
            secs = try_opt!(secs.checked_sub(1));
        }
        let d = Duration { secs, nanos };
        // Even if d is within the bounds of i64 seconds,
        // it might still overflow i64 milliseconds.
        if d < MIN || d > MAX {
            None
        } else {
            Some(d)
        }
    }

    /// Returns the duration as an absolute (non-negative) value.
    #[inline]
    pub const fn abs(&self) -> Duration {
        if self.secs < 0 && self.nanos != 0 {
            Duration { secs: (self.secs + 1).abs(), nanos: NANOS_PER_SEC - self.nanos }
        } else {
            Duration { secs: self.secs.abs(), nanos: self.nanos }
        }
    }

    /// The minimum possible `Duration`: `-i64::MAX` milliseconds.
    #[inline]
    pub const fn min_value() -> Duration {
        MIN
    }

    /// The maximum possible `Duration`: `i64::MAX` milliseconds.
    #[inline]
    pub const fn max_value() -> Duration {
        MAX
    }

    /// A duration where the stored seconds and nanoseconds are equal to zero.
    #[inline]
    pub const fn zero() -> Duration {
        Duration { secs: 0, nanos: 0 }
    }

    /// Returns `true` if the duration equals `Duration::zero()`.
    #[inline]
    pub const fn is_zero(&self) -> bool {
        self.secs == 0 && self.nanos == 0
    }

    /// Creates a `time::Duration` object from `std::time::Duration`
    ///
    /// This function errors when original duration is larger than the maximum
    /// value supported for this type.
    pub fn from_std(duration: StdDuration) -> Result<Duration, OutOfRangeError> {
        // We need to check secs as u64 before coercing to i64
        if duration.as_secs() > MAX.secs as u64 {
            return Err(OutOfRangeError(()));
        }
        let d = Duration { secs: duration.as_secs() as i64, nanos: duration.subsec_nanos() as i32 };
        if d > MAX {
            return Err(OutOfRangeError(()));
        }
        Ok(d)
    }

    /// Creates a `std::time::Duration` object from `time::Duration`
    ///
    /// This function errors when duration is less than zero. As standard
    /// library implementation is limited to non-negative values.
    pub fn to_std(&self) -> Result<StdDuration, OutOfRangeError> {
        if self.secs < 0 {
            return Err(OutOfRangeError(()));
        }
        Ok(StdDuration::new(self.secs as u64, self.nanos as u32))
    }
}

impl Neg for Duration {
    type Output = Duration;

    #[inline]
    fn neg(self) -> Duration {
        if self.nanos == 0 {
            Duration { secs: -self.secs, nanos: 0 }
        } else {
            Duration { secs: -self.secs - 1, nanos: NANOS_PER_SEC - self.nanos }
        }
    }
}

impl Add for Duration {
    type Output = Duration;

    fn add(self, rhs: Duration) -> Duration {
        self.checked_add(&rhs).expect("`Duration + Duration` overflowed")
    }
}

impl Sub for Duration {
    type Output = Duration;

    fn sub(self, rhs: Duration) -> Duration {
        self.checked_sub(&rhs).expect("`Duration - Duration` overflowed")
    }
}

impl AddAssign for Duration {
    fn add_assign(&mut self, rhs: Duration) {
        let new = self.checked_add(&rhs).expect("`Duration + Duration` overflowed");
        *self = new;
    }
}

impl SubAssign for Duration {
    fn sub_assign(&mut self, rhs: Duration) {
        let new = self.checked_sub(&rhs).expect("`Duration - Duration` overflowed");
        *self = new;
    }
}

impl Mul<i32> for Duration {
    type Output = Duration;

    fn mul(self, rhs: i32) -> Duration {
        // Multiply nanoseconds as i64, because it cannot overflow that way.
        let total_nanos = self.nanos as i64 * rhs as i64;
        let (extra_secs, nanos) = div_mod_floor_64(total_nanos, NANOS_PER_SEC as i64);
        let secs = self.secs * rhs as i64 + extra_secs;
        Duration { secs, nanos: nanos as i32 }
    }
}

impl Div<i32> for Duration {
    type Output = Duration;

    fn div(self, rhs: i32) -> Duration {
        let mut secs = self.secs / rhs as i64;
        let carry = self.secs - secs * rhs as i64;
        let extra_nanos = carry * NANOS_PER_SEC as i64 / rhs as i64;
        let mut nanos = self.nanos / rhs + extra_nanos as i32;
        if nanos >= NANOS_PER_SEC {
            nanos -= NANOS_PER_SEC;
            secs += 1;
        }
        if nanos < 0 {
            nanos += NANOS_PER_SEC;
            secs -= 1;
        }
        Duration { secs, nanos }
    }
}

impl<'a> core::iter::Sum<&'a Duration> for Duration {
    fn sum<I: Iterator<Item = &'a Duration>>(iter: I) -> Duration {
        iter.fold(Duration::zero(), |acc, x| acc + *x)
    }
}

impl core::iter::Sum<Duration> for Duration {
    fn sum<I: Iterator<Item = Duration>>(iter: I) -> Duration {
        iter.fold(Duration::zero(), |acc, x| acc + x)
    }
}

impl fmt::Display for Duration {
    /// Format a duration using the [ISO 8601] format
    ///
    /// [ISO 8601]: https://en.wikipedia.org/wiki/ISO_8601#Durations
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // technically speaking, negative duration is not valid ISO 8601,
        // but we need to print it anyway.
        let (abs, sign) = if self.secs < 0 { (-*self, "-") } else { (*self, "") };

        let days = abs.secs / SECS_PER_DAY;
        let secs = abs.secs - days * SECS_PER_DAY;
        let hasdate = days != 0;
        let hastime = (secs != 0 || abs.nanos != 0) || !hasdate;

        write!(f, "{}P", sign)?;

        if hasdate {
            write!(f, "{}D", days)?;
        }
        if hastime {
            if abs.nanos == 0 {
                write!(f, "T{}S", secs)?;
            } else if abs.nanos % NANOS_PER_MILLI == 0 {
                write!(f, "T{}.{:03}S", secs, abs.nanos / NANOS_PER_MILLI)?;
            } else if abs.nanos % NANOS_PER_MICRO == 0 {
                write!(f, "T{}.{:06}S", secs, abs.nanos / NANOS_PER_MICRO)?;
            } else {
                write!(f, "T{}.{:09}S", secs, abs.nanos)?;
            }
        }
        Ok(())
    }
}

/// Represents error when converting `Duration` to/from a standard library
/// implementation
///
/// The `std::time::Duration` supports a range from zero to `u64::MAX`
/// *seconds*, while this module supports signed range of up to
/// `i64::MAX` of *milliseconds*.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutOfRangeError(());

impl fmt::Display for OutOfRangeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Source duration value is out of range for the target type")
    }
}

#[cfg(feature = "std")]
impl Error for OutOfRangeError {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        "out of range error"
    }
}

#[inline]
const fn div_mod_floor_64(this: i64, other: i64) -> (i64, i64) {
    (this.div_euclid(other), this.rem_euclid(other))
}

#[cfg(all(feature = "arbitrary", feature = "std"))]
impl arbitrary::Arbitrary<'_> for Duration {
    fn arbitrary(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Duration> {
        const MIN_SECS: i64 = -i64::MAX / MILLIS_PER_SEC - 1;
        const MAX_SECS: i64 = i64::MAX / MILLIS_PER_SEC;

        let secs: i64 = u.int_in_range(MIN_SECS..=MAX_SECS)?;
        let nanos: i32 = u.int_in_range(0..=(NANOS_PER_SEC - 1))?;
        let duration = Duration { secs, nanos };

        if duration < MIN || duration > MAX {
            Err(arbitrary::Error::IncorrectFormat)
        } else {
            Ok(duration)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::OutOfRangeError;
    use super::{Duration, MAX, MIN};
    use core::time::Duration as StdDuration;

    #[test]
    fn test_duration() {
        assert!(Duration::seconds(1) != Duration::zero());
        assert_eq!(Duration::seconds(1) + Duration::seconds(2), Duration::seconds(3));
        assert_eq!(
            Duration::seconds(86_399) + Duration::seconds(4),
            Duration::days(1) + Duration::seconds(3)
        );
        assert_eq!(Duration::days(10) - Duration::seconds(1000), Duration::seconds(863_000));
        assert_eq!(Duration::days(10) - Duration::seconds(1_000_000), Duration::seconds(-136_000));
        assert_eq!(
            Duration::days(2) + Duration::seconds(86_399) + Duration::nanoseconds(1_234_567_890),
            Duration::days(3) + Duration::nanoseconds(234_567_890)
        );
        assert_eq!(-Duration::days(3), Duration::days(-3));
        assert_eq!(
            -(Duration::days(3) + Duration::seconds(70)),
            Duration::days(-4) + Duration::seconds(86_400 - 70)
        );

        let mut d = Duration::default();
        d += Duration::minutes(1);
        d -= Duration::seconds(30);
        assert_eq!(d, Duration::seconds(30));
    }

    #[test]
    fn test_duration_num_days() {
        assert_eq!(Duration::zero().num_days(), 0);
        assert_eq!(Duration::days(1).num_days(), 1);
        assert_eq!(Duration::days(-1).num_days(), -1);
        assert_eq!(Duration::seconds(86_399).num_days(), 0);
        assert_eq!(Duration::seconds(86_401).num_days(), 1);
        assert_eq!(Duration::seconds(-86_399).num_days(), 0);
        assert_eq!(Duration::seconds(-86_401).num_days(), -1);
        assert_eq!(Duration::days(i32::MAX as i64).num_days(), i32::MAX as i64);
        assert_eq!(Duration::days(i32::MIN as i64).num_days(), i32::MIN as i64);
    }

    #[test]
    fn test_duration_num_seconds() {
        assert_eq!(Duration::zero().num_seconds(), 0);
        assert_eq!(Duration::seconds(1).num_seconds(), 1);
        assert_eq!(Duration::seconds(-1).num_seconds(), -1);
        assert_eq!(Duration::milliseconds(999).num_seconds(), 0);
        assert_eq!(Duration::milliseconds(1001).num_seconds(), 1);
        assert_eq!(Duration::milliseconds(-999).num_seconds(), 0);
        assert_eq!(Duration::milliseconds(-1001).num_seconds(), -1);
    }

    #[test]
    fn test_duration_num_milliseconds() {
        assert_eq!(Duration::zero().num_milliseconds(), 0);
        assert_eq!(Duration::milliseconds(1).num_milliseconds(), 1);
        assert_eq!(Duration::milliseconds(-1).num_milliseconds(), -1);
        assert_eq!(Duration::microseconds(999).num_milliseconds(), 0);
        assert_eq!(Duration::microseconds(1001).num_milliseconds(), 1);
        assert_eq!(Duration::microseconds(-999).num_milliseconds(), 0);
        assert_eq!(Duration::microseconds(-1001).num_milliseconds(), -1);
        assert_eq!(Duration::milliseconds(i64::MAX).num_milliseconds(), i64::MAX);
        assert_eq!(Duration::milliseconds(-i64::MAX).num_milliseconds(), -i64::MAX);
        assert_eq!(MAX.num_milliseconds(), i64::MAX);
        assert_eq!(MIN.num_milliseconds(), -i64::MAX);
    }

    #[test]
    fn test_duration_num_microseconds() {
        assert_eq!(Duration::zero().num_microseconds(), Some(0));
        assert_eq!(Duration::microseconds(1).num_microseconds(), Some(1));
        assert_eq!(Duration::microseconds(-1).num_microseconds(), Some(-1));
        assert_eq!(Duration::nanoseconds(999).num_microseconds(), Some(0));
        assert_eq!(Duration::nanoseconds(1001).num_microseconds(), Some(1));
        assert_eq!(Duration::nanoseconds(-999).num_microseconds(), Some(0));
        assert_eq!(Duration::nanoseconds(-1001).num_microseconds(), Some(-1));
        assert_eq!(Duration::microseconds(i64::MAX).num_microseconds(), Some(i64::MAX));
        assert_eq!(Duration::microseconds(-i64::MAX).num_microseconds(), Some(-i64::MAX));
        assert_eq!(MAX.num_microseconds(), None);
        assert_eq!(MIN.num_microseconds(), None);

        // overflow checks
        const MICROS_PER_DAY: i64 = 86_400_000_000;
        assert_eq!(
            Duration::days(i64::MAX / MICROS_PER_DAY).num_microseconds(),
            Some(i64::MAX / MICROS_PER_DAY * MICROS_PER_DAY)
        );
        assert_eq!(
            Duration::days(-i64::MAX / MICROS_PER_DAY).num_microseconds(),
            Some(-i64::MAX / MICROS_PER_DAY * MICROS_PER_DAY)
        );
        assert_eq!(Duration::days(i64::MAX / MICROS_PER_DAY + 1).num_microseconds(), None);
        assert_eq!(Duration::days(-i64::MAX / MICROS_PER_DAY - 1).num_microseconds(), None);
    }

    #[test]
    fn test_duration_num_nanoseconds() {
        assert_eq!(Duration::zero().num_nanoseconds(), Some(0));
        assert_eq!(Duration::nanoseconds(1).num_nanoseconds(), Some(1));
        assert_eq!(Duration::nanoseconds(-1).num_nanoseconds(), Some(-1));
        assert_eq!(Duration::nanoseconds(i64::MAX).num_nanoseconds(), Some(i64::MAX));
        assert_eq!(Duration::nanoseconds(-i64::MAX).num_nanoseconds(), Some(-i64::MAX));
        assert_eq!(MAX.num_nanoseconds(), None);
        assert_eq!(MIN.num_nanoseconds(), None);

        // overflow checks
        const NANOS_PER_DAY: i64 = 86_400_000_000_000;
        assert_eq!(
            Duration::days(i64::MAX / NANOS_PER_DAY).num_nanoseconds(),
            Some(i64::MAX / NANOS_PER_DAY * NANOS_PER_DAY)
        );
        assert_eq!(
            Duration::days(-i64::MAX / NANOS_PER_DAY).num_nanoseconds(),
            Some(-i64::MAX / NANOS_PER_DAY * NANOS_PER_DAY)
        );
        assert_eq!(Duration::days(i64::MAX / NANOS_PER_DAY + 1).num_nanoseconds(), None);
        assert_eq!(Duration::days(-i64::MAX / NANOS_PER_DAY - 1).num_nanoseconds(), None);
    }

    #[test]
    fn test_duration_checked_ops() {
        assert_eq!(
            Duration::milliseconds(i64::MAX - 1).checked_add(&Duration::microseconds(999)),
            Some(Duration::milliseconds(i64::MAX - 2) + Duration::microseconds(1999))
        );
        assert!(Duration::milliseconds(i64::MAX)
            .checked_add(&Duration::microseconds(1000))
            .is_none());

        assert_eq!(
            Duration::milliseconds(-i64::MAX).checked_sub(&Duration::milliseconds(0)),
            Some(Duration::milliseconds(-i64::MAX))
        );
        assert!(Duration::milliseconds(-i64::MAX)
            .checked_sub(&Duration::milliseconds(1))
            .is_none());
    }

    #[test]
    fn test_duration_abs() {
        assert_eq!(Duration::milliseconds(1300).abs(), Duration::milliseconds(1300));
        assert_eq!(Duration::milliseconds(1000).abs(), Duration::milliseconds(1000));
        assert_eq!(Duration::milliseconds(300).abs(), Duration::milliseconds(300));
        assert_eq!(Duration::milliseconds(0).abs(), Duration::milliseconds(0));
        assert_eq!(Duration::milliseconds(-300).abs(), Duration::milliseconds(300));
        assert_eq!(Duration::milliseconds(-700).abs(), Duration::milliseconds(700));
        assert_eq!(Duration::milliseconds(-1000).abs(), Duration::milliseconds(1000));
        assert_eq!(Duration::milliseconds(-1300).abs(), Duration::milliseconds(1300));
        assert_eq!(Duration::milliseconds(-1700).abs(), Duration::milliseconds(1700));
        assert_eq!(Duration::milliseconds(-i64::MAX).abs(), Duration::milliseconds(i64::MAX));
    }

    #[test]
    #[allow(clippy::erasing_op)]
    fn test_duration_mul() {
        assert_eq!(Duration::zero() * i32::MAX, Duration::zero());
        assert_eq!(Duration::zero() * i32::MIN, Duration::zero());
        assert_eq!(Duration::nanoseconds(1) * 0, Duration::zero());
        assert_eq!(Duration::nanoseconds(1) * 1, Duration::nanoseconds(1));
        assert_eq!(Duration::nanoseconds(1) * 1_000_000_000, Duration::seconds(1));
        assert_eq!(Duration::nanoseconds(1) * -1_000_000_000, -Duration::seconds(1));
        assert_eq!(-Duration::nanoseconds(1) * 1_000_000_000, -Duration::seconds(1));
        assert_eq!(
            Duration::nanoseconds(30) * 333_333_333,
            Duration::seconds(10) - Duration::nanoseconds(10)
        );
        assert_eq!(
            (Duration::nanoseconds(1) + Duration::seconds(1) + Duration::days(1)) * 3,
            Duration::nanoseconds(3) + Duration::seconds(3) + Duration::days(3)
        );
        assert_eq!(Duration::milliseconds(1500) * -2, Duration::seconds(-3));
        assert_eq!(Duration::milliseconds(-1500) * 2, Duration::seconds(-3));
    }

    #[test]
    fn test_duration_div() {
        assert_eq!(Duration::zero() / i32::MAX, Duration::zero());
        assert_eq!(Duration::zero() / i32::MIN, Duration::zero());
        assert_eq!(Duration::nanoseconds(123_456_789) / 1, Duration::nanoseconds(123_456_789));
        assert_eq!(Duration::nanoseconds(123_456_789) / -1, -Duration::nanoseconds(123_456_789));
        assert_eq!(-Duration::nanoseconds(123_456_789) / -1, Duration::nanoseconds(123_456_789));
        assert_eq!(-Duration::nanoseconds(123_456_789) / 1, -Duration::nanoseconds(123_456_789));
        assert_eq!(Duration::seconds(1) / 3, Duration::nanoseconds(333_333_333));
        assert_eq!(Duration::seconds(4) / 3, Duration::nanoseconds(1_333_333_333));
        assert_eq!(Duration::seconds(-1) / 2, Duration::milliseconds(-500));
        assert_eq!(Duration::seconds(1) / -2, Duration::milliseconds(-500));
        assert_eq!(Duration::seconds(-1) / -2, Duration::milliseconds(500));
        assert_eq!(Duration::seconds(-4) / 3, Duration::nanoseconds(-1_333_333_333));
        assert_eq!(Duration::seconds(-4) / -3, Duration::nanoseconds(1_333_333_333));
    }

    #[test]
    fn test_duration_sum() {
        let duration_list_1 = [Duration::zero(), Duration::seconds(1)];
        let sum_1: Duration = duration_list_1.iter().sum();
        assert_eq!(sum_1, Duration::seconds(1));

        let duration_list_2 =
            [Duration::zero(), Duration::seconds(1), Duration::seconds(6), Duration::seconds(10)];
        let sum_2: Duration = duration_list_2.iter().sum();
        assert_eq!(sum_2, Duration::seconds(17));

        let duration_arr =
            [Duration::zero(), Duration::seconds(1), Duration::seconds(6), Duration::seconds(10)];
        let sum_3: Duration = duration_arr.into_iter().sum();
        assert_eq!(sum_3, Duration::seconds(17));
    }

    #[test]
    fn test_duration_fmt() {
        assert_eq!(Duration::zero().to_string(), "PT0S");
        assert_eq!(Duration::days(42).to_string(), "P42D");
        assert_eq!(Duration::days(-42).to_string(), "-P42D");
        assert_eq!(Duration::seconds(42).to_string(), "PT42S");
        assert_eq!(Duration::milliseconds(42).to_string(), "PT0.042S");
        assert_eq!(Duration::microseconds(42).to_string(), "PT0.000042S");
        assert_eq!(Duration::nanoseconds(42).to_string(), "PT0.000000042S");
        assert_eq!((Duration::days(7) + Duration::milliseconds(6543)).to_string(), "P7DT6.543S");
        assert_eq!(Duration::seconds(-86_401).to_string(), "-P1DT1S");
        assert_eq!(Duration::nanoseconds(-1).to_string(), "-PT0.000000001S");

        // the format specifier should have no effect on `Duration`
        assert_eq!(
            format!("{:30}", Duration::days(1) + Duration::milliseconds(2345)),
            "P1DT2.345S"
        );
    }

    #[test]
    fn test_to_std() {
        assert_eq!(Duration::seconds(1).to_std(), Ok(StdDuration::new(1, 0)));
        assert_eq!(Duration::seconds(86_401).to_std(), Ok(StdDuration::new(86_401, 0)));
        assert_eq!(Duration::milliseconds(123).to_std(), Ok(StdDuration::new(0, 123_000_000)));
        assert_eq!(
            Duration::milliseconds(123_765).to_std(),
            Ok(StdDuration::new(123, 765_000_000))
        );
        assert_eq!(Duration::nanoseconds(777).to_std(), Ok(StdDuration::new(0, 777)));
        assert_eq!(MAX.to_std(), Ok(StdDuration::new(9_223_372_036_854_775, 807_000_000)));
        assert_eq!(Duration::seconds(-1).to_std(), Err(OutOfRangeError(())));
        assert_eq!(Duration::milliseconds(-1).to_std(), Err(OutOfRangeError(())));
    }

    #[test]
    fn test_from_std() {
        assert_eq!(Ok(Duration::seconds(1)), Duration::from_std(StdDuration::new(1, 0)));
        assert_eq!(Ok(Duration::seconds(86_401)), Duration::from_std(StdDuration::new(86_401, 0)));
        assert_eq!(
            Ok(Duration::milliseconds(123)),
            Duration::from_std(StdDuration::new(0, 123_000_000))
        );
        assert_eq!(
            Ok(Duration::milliseconds(123_765)),
            Duration::from_std(StdDuration::new(123, 765_000_000))
        );
        assert_eq!(Ok(Duration::nanoseconds(777)), Duration::from_std(StdDuration::new(0, 777)));
        assert_eq!(
            Ok(MAX),
            Duration::from_std(StdDuration::new(9_223_372_036_854_775, 807_000_000))
        );
        assert_eq!(
            Duration::from_std(StdDuration::new(9_223_372_036_854_776, 0)),
            Err(OutOfRangeError(()))
        );
        assert_eq!(
            Duration::from_std(StdDuration::new(9_223_372_036_854_775, 807_000_001)),
            Err(OutOfRangeError(()))
        );
    }

    #[test]
    #[cfg(feature = "rkyv-validation")]
    fn test_rkyv_validation() {
        let duration = Duration::seconds(1);
        let bytes = rkyv::to_bytes::<_, 16>(&duration).unwrap();
        assert_eq!(rkyv::from_bytes::<Duration>(&bytes).unwrap(), duration);
    }
}
