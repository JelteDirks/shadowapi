pub mod log {
    use chrono::Utc;

    pub fn timed_msg(msg: String, when: chrono::DateTime<Utc>) {
        eprintln!("{}: {}", when, msg);
    }
}
