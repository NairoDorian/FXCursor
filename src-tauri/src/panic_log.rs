//! Makes an unexpected panic leave diagnostic evidence.

use std::panic::PanicHookInfo;

fn describe(info: &PanicHookInfo<'_>) -> String {
    let message = info
        .payload()
        .downcast_ref::<&str>()
        .map(|s| (*s).to_string())
        .or_else(|| info.payload().downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "<non-string panic payload>".to_string());

    let location = info
        .location()
        .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
        .unwrap_or_else(|| "<unknown location>".to_string());

    let thread = std::thread::current();
    let thread_name = thread.name().unwrap_or("<unnamed>");

    format!("[panic] thread '{thread_name}' panicked at {location}: {message}")
}

/// Installs the logging panic hook. Call once at application launch.
pub fn install() {
    let default_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |info| {
        let msg = describe(info);
        eprintln!("{msg}");
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("panic.log")
        {
            use std::io::Write;
            let _ = writeln!(f, "{msg}");
        }
        default_hook(info);
    }));
}

#[cfg(test)]
mod tests {
    use super::describe;
    use std::sync::Mutex;

    fn describe_panic(body: impl FnOnce() + std::panic::UnwindSafe) -> String {
        static SERIAL: Mutex<()> = Mutex::new(());
        static CAPTURED: Mutex<Option<String>> = Mutex::new(None);

        let _serial = SERIAL.lock().unwrap_or_else(|p| p.into_inner());
        let previous = std::panic::take_hook();

        std::panic::set_hook(Box::new(|info| {
            let mut slot = CAPTURED.lock().unwrap_or_else(|p| p.into_inner());
            *slot = Some(describe(info));
        }));

        let _ = std::panic::catch_unwind(body);
        std::panic::set_hook(previous);

        CAPTURED
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .take()
            .expect("hook must have run")
    }

    #[test]
    fn panic_log_describes_literal_panic() {
        let desc = describe_panic(|| panic!("test panic message"));
        assert!(desc.contains("test panic message"));
        assert!(desc.contains("[panic]"));
    }
}
