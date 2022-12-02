use core::fmt;
use core::iter::Step;
use core::ops::Add;
use std::str::FromStr;

use serde::Deserialize;
use thiserror::Error;

use crate::iter_const;
use crate::time::{Month, WeekDay, Year};
use crate::utils::StrExt;

#[macro_export]
macro_rules! date {
    ($year:literal : $month:literal : $day:literal) => {{
        const _YEAR: $crate::time::Year = $crate::time::Year::new($year);
        static_assertions::const_assert!($month >= 1 && $month <= 12);

        const _MONTH: $crate::time::Month = $crate::time::Month::new($month);

        // validate the day
        static_assertions::const_assert!($day != 0);
        static_assertions::const_assert!($day <= _YEAR.number_of_days_in_month(_MONTH));

        unsafe { $crate::time::Date::new_unchecked(_YEAR, _MONTH, $day) }
    }};
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(try_from = "String")]
pub struct Date {
    year: Year,
    month: Month,
    day: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("{year:04}-{month:02}-{day:02}: not a valid date")]
pub struct InvalidDate {
    year: Year,
    month: Month,
    day: usize,
}

/*
const fn cumulative_days_in_month_common_leap() -> [[u16; 11]; 2] {

    let mut result = [[0; 11]; 2];

    let sum = 0;
    iter_const!(for month in 0,..12 => {
        let month = Month::months()[month - 1];
        result
    });

    result
}*/

impl Date {
    pub fn new(year: impl Into<Year>, month: Month, day: usize) -> Result<Self, InvalidDate> {
        let year = year.into();
        if year.number_of_days_in_month(month) < day || day == 0 {
            return Err(InvalidDate { year, month, day });
        }

        Ok(Self { year, month, day })
    }

    #[doc(hidden)]
    #[must_use]
    pub const unsafe fn new_unchecked(year: Year, month: Month, day: usize) -> Self {
        Self { year, month, day }
    }

    pub const fn week_day(&self) -> WeekDay {
        self.year().week_day(self.month(), self.day())
    }

    pub const fn year(&self) -> Year {
        self.year
    }

    pub const fn month(&self) -> Month {
        self.month
    }

    pub const fn day(&self) -> usize {
        self.day
    }

    // TODO: write some good tests for this, also take care of https://github.com/kit-sdq/TimeSheetGenerator/pull/121
    pub const fn is_holiday(&self) -> bool {
        // check for christmas dates:
        self.month.is_eq(&Month::December) && (self.day() == 25 || self.day() == 26) ||
        // new year's day
        self.month.is_eq(&Month::January) && self.day() == 1 ||
        self.month.is_eq(&Month::January) && self.day() == 6 ||
        self.month.is_eq(&Month::November) && self.day() == 1

        // TODO: add remaining holidays
        // https://github.com/kit-sdq/TimeSheetGenerator/blob/master/src/main/java/checker/holiday/GermanyHolidayChecker.java
        // https://www.dgb.de/gesetzliche-feiertage-deutschland-2020-2021#badenwuerttemberg
        // https://crates.io/crates/json_typegen/0.5.0
    }

    // TODO: might make this more powerful
    pub fn formatted(&self, f: &str) -> String {
        f.replace("{year}", &format!("{:04}", self.year()))
            .replace("{month}", &format!("{:02}", self.month()))
            .replace("{day}", &format!("{:02}", self.day()))
    }

    /// Returns the date of the first day as a date in the month.
    #[must_use]
    pub const fn first_day(year: Year, month: Month) -> Self {
        Self {
            year,
            month,
            day: 1,
        }
    }

    /// Returns the date of the last day as a date in the month.
    #[must_use]
    pub const fn last_day(year: Year, month: Month) -> Self {
        Self {
            year,
            month,
            day: year.number_of_days_in_month(month),
        }
    }

    #[must_use]
    const fn apply_offset(week_day: WeekDay, day: usize) -> usize {
        // TODO: could be replaced with week_day.as_usize() - 1
        let offset = {
            match week_day {
                WeekDay::Monday => 0,
                WeekDay::Tuesday => 1,
                WeekDay::Wednesday => 2,
                WeekDay::Thursday => 3,
                WeekDay::Friday => 4,
                WeekDay::Saturday => 5,
                WeekDay::Sunday => 6,
            }
        };

        // In rust divisions always round down.
        // Dividing any number x by 7 for which holds:
        // 7 * n <= x < 7 * (n + 1) will result in n
        //
        // The first week number is 1 and not 0, so to each day 7 is added.
        //
        // Then the offset is added to the day, so that all mondays are a multiple of 7.
        // (one can calculate the week_numbers for weeks starting not on monday the same
        //  way, just make the day where the week starts a multiple of 7)
        //
        // Months starting with a monday will have the days 1, 8, 15, 22, 29
        // The offset is added so that they will be 0, 7, 14, 21, 28 (or with the + 7):
        // 7, 14, 21, 28, 35
        //  7 / 7 = 1
        // 14 / 7 = 2
        // 21 / 7 = 3
        // 28 / 7 = 4
        // 35 / 7 = 5
        offset + 7 - 1 + day
    }

    #[must_use]
    pub const fn week_number(&self) -> usize {
        Self::apply_offset(
            Self::first_day(self.year(), self.month()).week_day(),
            self.day(),
        ) / 7
    }

    /// Returns the date of the last day in the current week.
    #[must_use]
    pub const fn week_end(&self) -> Self {
        Self {
            year: self.year(),
            month: self.month(),
            day: {
                let distance = self.week_day().days_until(WeekDay::Sunday);
                if self.day() + distance > self.year().number_of_days_in_month(self.month()) {
                    self.year().number_of_days_in_month(self.month())
                } else {
                    self.day() + distance
                }
            },
        }
    }

    #[must_use]
    pub const fn week_start(&self) -> Self {
        Self {
            year: self.year(),
            month: self.month(),
            day: {
                let distance = WeekDay::Monday.days_until(self.week_day());
                if self.day() <= distance {
                    1
                } else {
                    self.day() - distance
                }
            },
        }
    }

    #[must_use]
    pub const fn is_workday(&self) -> bool {
        !self.is_holiday() && !self.week_day().is_eq(&WeekDay::Sunday)
    }

    const fn ordinal(&self) -> u16 {
        let mut result = 0;

        let mut current_month = Month::January;
        while !self.month().is_eq(&current_month) {
            result += self.year().number_of_days_in_month(current_month) as u16;
            current_month = current_month.next();
        }

        result + self.day() as u16
    }

    const fn days_since_base_date(&self) -> usize {
        self.year.days_since_base_date() + self.ordinal() as usize
    }

    #[must_use]
    const fn add_days(mut self, days: usize) -> Self {
        iter_const!(for _i in 0,..days => {
            self.day += 1;
            if self.day > self.year().number_of_days_in_month(self.month()) {
                if self.month.is_eq(&Month::December) {
                    self.year = self.year.next();
                }

                self.month = self.month().next();
                self.day = 1;
            }
        });

        self
    }

    #[must_use]
    const fn from_days_since_base_date(days: usize) -> Self {
        let year = Year::from_days_since_base_date(days);
        let ordinal = days - year.days_since_base_date();
        Self::from_ordinal(year, ordinal as u16)
    }

    #[must_use]
    const fn from_ordinal(year: Year, ordinal: u16) -> Self {
        if year.days() < ordinal as usize || ordinal == 0 {
            panic!("Invalid ordinal for year");
        }

        Self::first_day(year, Month::January).add_days((ordinal - 1) as usize)
    }

    /// Returns the date when the next week starts or `None` if the next week
    /// would be in the next month.
    #[must_use]
    pub const fn next_week_start(&self) -> Option<Self> {
        let next_week = self.week_start().add_days(7);

        if next_week.month().is_eq(&self.month()) {
            Some(next_week.week_start())
        } else {
            None
        }
    }
}

impl Add<usize> for Date {
    type Output = Self;

    fn add(self, days: usize) -> Self::Output {
        self.add_days(days)
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:04}-{:02}-{:02}",
            self.year.as_usize(),
            self.month.as_usize(),
            self.day
        )
    }
}

impl Step for Date {
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
        <usize as Step>::steps_between(&start.days_since_base_date(), &end.days_since_base_date())
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        <usize as Step>::forward_checked(start.days_since_base_date(), count)
            .map(Self::from_days_since_base_date)
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        <usize as Step>::backward_checked(start.days_since_base_date(), count)
            .map(Self::from_days_since_base_date)
    }
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("`{}`: not a valid date, format must be `YYYY-MM-DD`", string)]
pub struct ParseDateError {
    string: String,
}

impl FromStr for Date {
    type Err = ParseDateError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if let [Some(year), Some(month), Some(day)] = string.split_exact::<3>("-") {
            let year = Year::new(year.parse::<usize>().map_err(|_| ParseDateError {
                string: string.to_string(),
            })?);
            let month = Month::try_from(month.parse::<usize>().map_err(|_| ParseDateError {
                string: string.to_string(),
            })?)
            .map_err(|_| ParseDateError {
                string: string.to_string(),
            })?;
            let day = day.parse().map_err(|_| ParseDateError {
                string: string.to_string(),
            })?;

            Ok(Self::new(year, month, day).map_err(|_| ParseDateError {
                string: string.to_string(),
            })?)
        } else {
            Err(ParseDateError {
                string: string.to_string(),
            })
        }
    }
}

impl TryFrom<String> for Date {
    type Error = ParseDateError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    use crate::min;
    use crate::utils::IteratorExt;
    use std::ops::RangeInclusive;

    #[test]
    fn test_date() {
        //
        assert_eq!(
            Date::new(Year::new(2022), Month::January, 31).map(|d| d.to_string()),
            Ok("2022-01-31".to_string())
        );
    }

    #[must_use]
    fn sort_array<T: Ord, const N: usize>(mut array: [T; N]) -> [T; N] {
        array.sort();
        array
    }

    #[test]
    fn test_date_sorting() {
        assert_eq!(
            sort_array([date!(2022:01:03), date!(2022:01:02), date!(2022:01:01)]),
            [date!(2022:01:01), date!(2022:01:02), date!(2022:01:03)]
        );

        assert_eq!(
            sort_array([date!(2012:01:03), date!(2013:01:02), date!(2024:01:01)]),
            [date!(2012:01:03), date!(2013:01:02), date!(2024:01:01)]
        );

        assert_eq!(
            sort_array([date!(2000:01:01), date!(2000:04:01), date!(2000:03:01)]),
            [date!(2000:01:01), date!(2000:03:01), date!(2000:04:01)]
        );
    }

    #[test]
    fn test_add_day() {
        assert_eq!(date!(2022:01:01).add_days(1), date!(2022:01:02));
        assert_eq!(date!(2022:01:01).add_days(30), date!(2022:01:31));
        assert_eq!(date!(2022:01:01).add_days(31), date!(2022:02:01));
        assert_eq!(date!(2022:01:01).add_days(58), date!(2022:02:28));
        assert_eq!(date!(2022:01:01).add_days(59), date!(2022:03:01));

        assert_eq!(date!(2022:12:24).add_days(8), date!(2023:01:01));
        assert_eq!(date!(2022:12:24).add_days(8 + 365), date!(2024:01:01));
    }

    #[test]
    fn test_ordinal() {
        assert_eq!(date!(2022:01:01).ordinal(), 1);
        assert_eq!(date!(2022:02:01).ordinal(), 32);
        assert_eq!(date!(2022:02:05).ordinal(), 36);

        for year in Year::new(2020)..=Year::new(3000) {
            let mut current_ordinal = 0;
            for month in Month::months() {
                for day in 1..=year.number_of_days_in_month(month) {
                    current_ordinal += 1;
                    let date = Date::new(year, month, day).unwrap();

                    assert_eq!(date.ordinal(), current_ordinal);
                }
            }
        }
    }

    #[test]
    fn test_from_days_since_base_date() {
        for year in Year::new(2020)..=Year::new(2025) {
            for month in Month::months() {
                for day in 1..year.number_of_days_in_month(month) {
                    let date = Date::new(year, month, day).unwrap();

                    assert_eq!(
                        Date::from_days_since_base_date(date.days_since_base_date()),
                        date
                    );
                }
            }
        }
    }

    #[inline]
    #[track_caller]
    fn test_week_number_value(
        year: Year,
        month: Month,
        expected: usize,
        days: impl IntoIterator<Item = usize>,
    ) {
        for day in days {
            let actual = Date::new(year, month, day).unwrap().week_number();
            assert_eq!(
                expected, actual,
                "week_number({}-{}-{:02}): expected: {}, actual: {}",
                year, month, day, expected, actual,
            );
        }
    }

    fn iter_weeks(year: Year, month: Month) -> Vec<(usize, RangeInclusive<usize>)> {
        let mut result = Vec::new();
        // NOTE: if monday is the first day, then this will be 0
        let day_before_first_monday = Date::new(year, month, 1)
            .unwrap()
            .week_day()
            .days_until(WeekDay::Monday);
        let days_in_month = year.number_of_days_in_month(month);

        let mut init = 0;
        if day_before_first_monday != 0 {
            result.push((1, 1..=day_before_first_monday));
            init = 1;
        }

        result.extend(
            (day_before_first_monday + 1..=days_in_month)
                .step_by(7)
                .map_with(init + 1, move |day, week_start| {
                    (
                        (week_start, day..=min!(day + 6, days_in_month)),
                        week_start + 1,
                    )
                }),
        );

        result
    }

    #[test]
    fn test_week_start_end() {
        for year in Year::new(2000)..=Year::new(2022) {
            for month in Month::months() {
                for (_, days) in iter_weeks(year, month) {
                    let week_start = Date::new(year, month, *days.start()).unwrap();
                    let week_end = Date::new(year, month, *days.end()).unwrap();

                    for day in days.into_iter().map(|d| Date::new(year, month, d).unwrap()) {
                        assert_eq!(day.week_start(), week_start, "week_start of day: {}", day);
                        assert_eq!(day.week_end(), week_end, "week_end of day: {}", day);
                    }
                }
            }
        }
    }

    #[test]
    fn test_week_number() {
        let year = Year::new(2022);
        let month = Month::November;

        test_week_number_value(year, month, 1, 1..=6);
        test_week_number_value(year, month, 2, 7..=13);
        test_week_number_value(year, month, 3, 14..=20);
        test_week_number_value(year, month, 4, 21..=27);
        test_week_number_value(year, month, 5, 28..=30);

        let year = Year::new(2022);
        let month = Month::December;

        test_week_number_value(year, month, 1, 1..=4);
        test_week_number_value(year, month, 2, 5..=11);
        test_week_number_value(year, month, 3, 12..=18);
        test_week_number_value(year, month, 4, 19..=25);
        test_week_number_value(year, month, 5, 26..=31);

        let year = Year::new(2021);
        let month = Month::November;

        test_week_number_value(year, month, 1, 1..=7);
        test_week_number_value(year, month, 2, 8..=14);
        test_week_number_value(year, month, 3, 15..=21);
        test_week_number_value(year, month, 4, 22..=28);
        test_week_number_value(year, month, 5, 29..=30);
    }

    #[test]
    fn test_week_number_elaborate() {
        for year in Year::new(1990)..=Year::new(2030) {
            for month in Month::months() {
                for (week_number, week) in iter_weeks(year, month) {
                    test_week_number_value(year, month, week_number, week);
                }
            }
        }
    }
}
