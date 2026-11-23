//! Arranque del programa de escritorio. Todo el código está en la biblioteca (`lib.rs`) para
//! poder compartirlo con la versión de Android.

// En Windows, sin ventana de consola en la versión final.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() -> eframe::Result<()> {
    qr_creator::run()
}
