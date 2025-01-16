use icu::{
    calendar::{DateTime, Gregorian},
    datetime::{options::length, TypedDateTimeFormatter},
    locid::locale,
};
use std::{fs::Metadata, time::UNIX_EPOCH};

/// Stores the ID and metadata of a password file.
#[derive(Debug, Clone)]
pub struct PasswordInfo {
    pub id: String,
    pub metadata: Metadata,
}

impl PasswordInfo {
    pub fn new(id: String, metadata: Metadata) -> Self {
        PasswordInfo { id, metadata }
    }

    pub fn last_modified(&self) -> String {
        if let Ok(modified_system_time) = self.metadata.modified() {
            if let Ok(duration) = modified_system_time.duration_since(UNIX_EPOCH) {
                // TypedDateTimeFormatter
                let options =
                    length::Bag::from_date_time_style(length::Date::Medium, length::Time::Short)
                        .into();
                let dtf =
                    TypedDateTimeFormatter::<Gregorian>::try_new(&locale!("en").into(), options)
                        .expect("failed to create TypedDateTimeFormatter instance.");
                // DateTime
                let datetime =
                    DateTime::from_minutes_since_local_unix_epoch(duration.as_secs() as i32 / 60)
                        .to_calendar(Gregorian);
                return dtf.format_to_string(&datetime);
            }
        }
        String::from("Unknown")
    }
}
