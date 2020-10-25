use chrono::{naive::NaiveDateTime, offset::TimeZone, DateTime, Local};

use std::time::SystemTime;

/// Filter based on time ranges.
#[derive(Debug, PartialEq)]
pub enum TimeFilter {
    Before(SystemTime),
    After(SystemTime),
}

impl TimeFilter {
    fn from_str(ref_time: &SystemTime, s: &str) -> Option<SystemTime> {
        humantime::parse_duration(s)
            .map(|duration| *ref_time - duration)
            .ok()
            .or_else(|| {
                humantime::parse_rfc3339_weak(s)
                    .or_else(|_| humantime::parse_rfc3339_weak(&(s.to_owned() + " 00:00:00")))
                    .ok()
                    .and_then(to_local_system_time)
            })
    }

    pub fn before(ref_time: &SystemTime, s: &str) -> Option<TimeFilter> {
        TimeFilter::from_str(ref_time, s).map(TimeFilter::Before)
    }

    pub fn after(ref_time: &SystemTime, s: &str) -> Option<TimeFilter> {
        TimeFilter::from_str(ref_time, s).map(TimeFilter::After)
    }

    pub fn applies_to(&self, t: &SystemTime) -> bool {
        match self {
            TimeFilter::Before(limit) => t <= limit,
            TimeFilter::After(limit) => t >= limit,
        }
    }
}

/// The humantime `parse_rfc3339_weak` function returns a UTC-based SystemTime,
/// the following is to convert to a local SystemTime
fn to_local_system_time(system_time: SystemTime) -> Option<SystemTime> {
    // convert to duration since epoch
    system_time
        .duration_since(SystemTime::UNIX_EPOCH)
        .ok()
        .and_then(|from_epoch| {
            // convert to local datetime
            Local
                .from_local_datetime(&NaiveDateTime::from_timestamp(
                    from_epoch.as_secs() as _,
                    from_epoch.subsec_nanos(),
                ))
                .single()
        })
        .and_then(|local_time| {
            // convert adjusted time back to SystemTime
            let local_epoch: DateTime<Local> = DateTime::from(SystemTime::UNIX_EPOCH);

            local_time
                .signed_duration_since(local_epoch)
                .to_std()
                .ok()
                .map(|duration| SystemTime::UNIX_EPOCH + duration)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn is_time_filter_applicable() {
        let ref_time =
            to_local_system_time(humantime::parse_rfc3339("2010-10-10T10:10:10Z").unwrap())
                .unwrap();

        assert!(TimeFilter::after(&ref_time, "1min")
            .unwrap()
            .applies_to(&ref_time));
        assert!(!TimeFilter::before(&ref_time, "1min")
            .unwrap()
            .applies_to(&ref_time));

        let t1m_ago = ref_time - Duration::from_secs(60);
        assert!(!TimeFilter::after(&ref_time, "30sec")
            .unwrap()
            .applies_to(&t1m_ago));
        assert!(TimeFilter::after(&ref_time, "2min")
            .unwrap()
            .applies_to(&t1m_ago));

        assert!(TimeFilter::before(&ref_time, "30sec")
            .unwrap()
            .applies_to(&t1m_ago));
        assert!(!TimeFilter::before(&ref_time, "2min")
            .unwrap()
            .applies_to(&t1m_ago));

        let t10s_before = "2010-10-10 10:10:00";
        assert!(!TimeFilter::before(&ref_time, t10s_before)
            .unwrap()
            .applies_to(&ref_time));
        assert!(TimeFilter::before(&ref_time, t10s_before)
            .unwrap()
            .applies_to(&t1m_ago));

        assert!(TimeFilter::after(&ref_time, t10s_before)
            .unwrap()
            .applies_to(&ref_time));
        assert!(!TimeFilter::after(&ref_time, t10s_before)
            .unwrap()
            .applies_to(&t1m_ago));
    }
}
