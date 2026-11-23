//! # Creador de QR
//!
//! Programa para crear códigos QR sin internet, con todas las reglas de la norma y muchas
//! opciones de diseño. Funciona en Windows, Linux y macOS (escritorio) y se puede compilar
//! para Android e iOS.
//!
//! Partes del programa:
//! - [`qr`]: codifica el texto según la norma (corrección, versión, máscara, modo).
//! - [`content`]: plantillas (Wi-Fi, contacto, evento...) que generan el texto del QR.
//! - [`style`]: todas las opciones de diseño, los diseños de la galería y las reglas.
//! - [`render`]: dibuja el QR decorado (PNG y SVG) y comprueba que se pueda leer.
//! - [`app`]: la ventana y sus controles.
//! - [`i18n`]: traducciones de la interfaz.
//! - [`platform`]: lo que cambia entre escritorio y móvil (archivos, portapapeles).

pub mod app;
pub mod content;
pub mod i18n;
pub mod platform;
pub mod qr;
pub mod render;
pub mod style;

use eframe::egui;
use std::path::PathBuf;

/// Arranca el programa en el escritorio (Windows, Linux, macOS) o en iOS.
///
/// Con `--screenshot <archivo.png>` abre la ventana sin quitar el foco, dibuja unos 120
/// cuadros, guarda una foto de la ventana en ese archivo y se cierra (lo usan las
/// comprobaciones automáticas).
pub fn run() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // `--exportar-iconos <carpeta>`: escribe los iconos para los instaladores y termina.
    if let Some(dir) = args.iter().position(|a| a == "--exportar-iconos").and_then(|i| args.get(i + 1)) {
        if let Err(e) = render::export_icons(std::path::Path::new(dir)) {
            eprintln!("No se pudieron escribir los iconos: {e}");
            std::process::exit(1);
        }
        return Ok(());
    }

    let shot = args
        .iter()
        .position(|a| a == "--screenshot")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from);

    // `--tamano 420x860` abre la ventana con otro tamaño (para probar la vista de teléfono).
    let size = args
        .iter()
        .position(|a| a == "--tamano")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.split_once('x'))
        .and_then(|(w, h)| Some([w.parse::<f32>().ok()?, h.parse::<f32>().ok()?]))
        .unwrap_or([1320.0, 860.0]);

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("QR Creator")
        .with_app_id("qr-creator")
        .with_icon(window_icon())
        .with_inner_size(size)
        .with_min_inner_size([420.0, 560.0]);
    if shot.is_some() {
        // No robar el foco a lo que la persona esté haciendo.
        viewport = viewport.with_active(false);
        // En Windows, además, detrás de las demás ventanas (sigue dibujándose).
        #[cfg(target_os = "windows")]
        {
            viewport = viewport.with_window_level(egui::WindowLevel::AlwaysOnBottom);
        }
    }

    #[allow(unused_mut)]
    let mut options = eframe::NativeOptions { viewport, ..Default::default() };

    // En macOS, no traer la app al frente cuando se hace la captura.
    #[cfg(target_os = "macos")]
    if shot.is_some() {
        options.event_loop_builder = Some(Box::new(|b| {
            use winit::platform::macos::EventLoopBuilderExtMacOS;
            b.with_activate_ignoring_other_apps(false);
        }));
    }

    // En iOS se dibuja con Metal (wgpu); OpenGL no está disponible allí.
    #[cfg(target_os = "ios")]
    {
        options.renderer = eframe::Renderer::Wgpu;
    }

    eframe::run_native("QR Creator", options, Box::new(move |cc| Ok(Box::new(app::QrApp::new(cc, shot)))))
}

/// Icono de la ventana y de la barra de tareas (dibujado por el propio programa).
fn window_icon() -> egui::IconData {
    let img = render::app_icon(128);
    egui::IconData { width: img.width(), height: img.height(), rgba: img.into_raw() }
}

/// Punto de entrada en Android: el sistema llama a esta función al abrir la app.
#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    // Carpeta privada de la app para ajustes y archivos exportados.
    if let Some(p) = app.external_data_path().or_else(|| app.internal_data_path()) {
        platform::set_data_dir(p);
    }
    let options = eframe::NativeOptions { android_app: Some(app), ..Default::default() };
    let _ = eframe::run_native(
        "QR Creator",
        options,
        Box::new(|cc| Ok(Box::new(app::QrApp::new(cc, None)))),
    );
}
