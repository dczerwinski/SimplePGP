use std::thread;

use glib::prelude::*;

/// Spawns a blocking task on a background thread and delivers the result
/// back on the GLib main loop.
///
/// `task` runs off the UI thread.
/// `callback` runs on the UI thread with the result of `task`.
pub fn spawn_blocking<T, F, C>(task: F, callback: C)
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
    C: FnOnce(T) + 'static,
{
    let (sender, receiver) = async_channel::bounded::<T>(1);
    let main_ctx = glib::MainContext::default();

    main_ctx.spawn_local(async move {
        if let Ok(result) = receiver.recv().await {
            callback(result);
        }
    });

    thread::spawn(move || {
        let result = task();
        let _ = sender.send_blocking(result);
    });
}
