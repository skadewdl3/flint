pub use app::*;

pub mod app;
pub mod generate;
pub mod help;
pub mod init;
pub mod install;
pub mod test;

use crossterm::event::Event;
use flint_utils::Result;
use ratatui::widgets::WidgetRef;
use std::sync::mpsc::Sender;
use threadpool::ThreadPool;

pub trait AppWidget: WidgetRef {
    fn setup(&mut self) -> Result<()> {
        Ok(())
    }
    fn handle_events(&mut self, _event: Event) -> Result<()> {
        Ok(())
    }
    fn set_exit_sender(&mut self, _exit_sender: Sender<()>) {}

    fn set_thread_pool(&mut self, _thread_pool: &ThreadPool) {}
}
