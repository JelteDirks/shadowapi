pub mod log {
    use std::fmt::Display;
    use std::path::PathBuf;
    use std::fs::File;
    use std::io::Write;

    pub fn timed_msg<T>(msg: T, when: chrono::DateTime<chrono::Utc>)
    where T: Display
    {
        println!("{}: {}", when, msg);
    }

    pub struct LoggingState {
        shadow_file: File,
        main_file: File,
    }

    impl LoggingState {
        pub fn new<T>(dir: T) -> Option<LoggingState>
        where T: Into<PathBuf>
        {
            let directory: PathBuf = dir.into();

            let mut shadow_file: PathBuf = directory.clone();
            shadow_file.push("shadow.log");

            let mut log_file: PathBuf = directory.clone();
            log_file.push("main.log");

            // TODO: files should not be truncated when created
            let shadow_file = match File::create(shadow_file) {
                Ok(f) => f,
                Err(_) => return None,
            };

            let log_file = match File::create(log_file) {
                Ok(f) => f,
                Err(_) => return None,
            };

            Some(LoggingState {
                shadow_file,
                main_file: log_file,
            })
        }
    }

    impl LoggingState {

        pub fn shadow_info<T>(&mut self, msg: T)
        where T: Display
        {
            let _ = self.shadow_file.write_all(format!("{}:[INFO] {}\n", chrono::Utc::now(), msg).as_bytes());
        }

        pub fn main_info<T>(&mut self, msg: T)
        where T: Display
        {
            let _ = self.main_file.write_all(format!("{}:[INFO] {}\n", chrono::Utc::now(), msg).as_bytes());
        }

        pub fn main<T>(&mut self, msg: T)
        where T: Display
        {
            self.main_info(msg);
        }

        pub fn shadow<T>(&mut self, msg: T)
        where T: Display
        {
            self.shadow_info(msg);
        }

    }
}
