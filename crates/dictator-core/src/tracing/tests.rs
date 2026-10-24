use super::*;

#[test]
fn subscriber_filters_events_and_appender_flushes_them() {
    let directory = tempfile::tempdir().expect("log directory");
    let path = directory.path().join("host.log");
    let file = std::fs::File::create(&path).expect("log file");
    let (writer, guard) = appender::non_blocking(file);
    let subscriber = subscriber::fmt()
        .with_env_filter(subscriber::EnvFilter::new("info"))
        .with_writer(writer)
        .with_ansi(false)
        .without_time()
        .finish();

    dispatcher::with_default(&Dispatch::new(subscriber), || {
        info!(decree = "supreme", "host event");
        debug!("filtered debug event");
    });
    drop(guard);

    let output = std::fs::read_to_string(path).expect("flushed logs");
    assert!(output.contains("host event"));
    assert!(output.contains("decree=\"supreme\""));
    assert!(!output.contains("filtered debug event"));
}
