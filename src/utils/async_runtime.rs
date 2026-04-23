use std::thread;

/// Spawns a blocking task on a background thread and delivers the result
/// back on the GLib main loop via a channel.
///
/// `task` runs off the UI thread.
/// `callback` runs on the UI thread with the result of `task`.
pub fn spawn_blocking<T, F, C>(task: F, callback: C)
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
    C: FnOnce(T) + 'static,
{
    let (sender, receiver) = glib::MainContext::channel::<T>(glib::Priority::DEFAULT);

    thread::spawn(move || {
        let result = task();
        let _ = sender.send(result);
    });

    receiver.attach(None, move |result| {
        callback(result);
        glib::ControlFlow::Break
    });
}
