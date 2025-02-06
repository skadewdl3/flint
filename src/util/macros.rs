#[macro_export]
macro_rules! handle_key_event {
    ($event:expr, { $($key:pat => $action:expr),+ $(,)? }) => {

        use crossterm::event::{Event, KeyEventKind};
        match $event {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    $($key => $action,)*
                    _ => {}
                }
            }
            _ => {}
        }
    };
}
