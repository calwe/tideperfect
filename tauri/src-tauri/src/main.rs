// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tideperfect::TidePerfectError;

#[snafu::report]
#[tokio::main]
async fn main() -> Result<(), TidePerfectError> {
    tideperfect_gui_lib::run()
}
