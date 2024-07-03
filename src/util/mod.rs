pub mod log {
    use std::fmt::Display;

    use chrono::Utc;

    pub fn timed_msg<T>(msg: T, when: chrono::DateTime<Utc>) 
    where T: Display
    {
        eprintln!("{}: {}", when, msg);
    }
}
