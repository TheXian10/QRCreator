//! Lo que cambia según el sistema: dónde se guardan los archivos, los cuadros de diálogo
//! para abrir y guardar, y el portapapeles.
//!
//! - En Windows, Linux y macOS se usan los cuadros de diálogo del sistema (crate `rfd`) y el
//!   portapapeles (crate `arboard`).
//! - En Android e iOS no hay cuadros de diálogo de escritorio: los archivos se guardan en la
//!   carpeta de la propia app y la interfaz muestra una lista para abrirlos.

use std::path::PathBuf;
use std::sync::OnceLock;

/// `true` cuando el programa corre en un teléfono o tableta.
pub const IS_MOBILE: bool = cfg!(any(target_os = "android", target_os = "ios"));

/// Carpeta de datos que el sistema da a la app (en Android la fija `android_main`).
static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Fija la carpeta de datos (solo la primera llamada cuenta).
#[allow(dead_code)]
pub fn set_data_dir(p: PathBuf) {
    let _ = DATA_DIR.set(p);
}

/// Carpeta donde la app guarda sus ajustes y, en el móvil, lo que exporta.
pub fn data_dir() -> PathBuf {
    if let Some(p) = DATA_DIR.get() {
        return p.clone();
    }
    // Windows: %APPDATA%\creador-qr
    if let Some(a) = std::env::var_os("APPDATA") {
        return PathBuf::from(a).join("creador-qr");
    }
    // iOS: la carpeta Documents de la app (visible en la app Archivos).
    if cfg!(target_os = "ios") {
        if let Some(h) = std::env::var_os("HOME") {
            return PathBuf::from(h).join("Documents");
        }
    }
    // Linux y macOS: $XDG_CONFIG_HOME o ~/.config
    if let Some(x) = std::env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(x).join("creador-qr");
    }
    if let Some(h) = std::env::var_os("HOME") {
        return PathBuf::from(h).join(".config").join("creador-qr");
    }
    std::env::temp_dir().join("creador-qr")
}

/// Subcarpeta de la carpeta de datos (se crea si no existe).
pub fn sub_dir(name: &str) -> PathBuf {
    let d = data_dir().join(name);
    let _ = std::fs::create_dir_all(&d);
    d
}

/// Qué clase de archivo se abre o se guarda.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    Png,
    Svg,
    Jpg,
    Design,
    Image,
}

impl FileKind {
    /// Extensiones que se aceptan.
    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            FileKind::Png => &["png"],
            FileKind::Svg => &["svg"],
            FileKind::Jpg => &["jpg", "jpeg"],
            FileKind::Design => &["json"],
            FileKind::Image => &["png", "jpg", "jpeg", "gif", "bmp", "webp", "ico"],
        }
    }
    /// Subcarpeta que se usa en el móvil.
    pub fn mobile_folder(self) -> &'static str {
        match self {
            FileKind::Png | FileKind::Svg | FileKind::Jpg => "QR",
            FileKind::Design => "Disenos",
            FileKind::Image => "Logos",
        }
    }
}

/// Pide al usuario dónde guardar. En el móvil devuelve directamente un nombre nuevo dentro
/// de la carpeta de la app (no hay cuadro de diálogo).
pub fn pick_save(kind: FileKind, title: &str, default_name: &str, filter_name: &str) -> Option<PathBuf> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        rfd::FileDialog::new()
            .set_title(title)
            .add_filter(filter_name, kind.extensions())
            .set_file_name(default_name)
            .save_file()
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = (title, filter_name);
        let dir = sub_dir(kind.mobile_folder());
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let (stem, ext) = default_name.rsplit_once('.').unwrap_or((default_name, kind.extensions()[0]));
        Some(dir.join(format!("{stem}-{stamp}.{ext}")))
    }
}

/// Pide al usuario un archivo para abrir. En el móvil devuelve `None`: la interfaz enseña
/// en su lugar la lista de `list_files`.
pub fn pick_open(kind: FileKind, title: &str, filter_name: &str) -> Option<PathBuf> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        rfd::FileDialog::new().set_title(title).add_filter(filter_name, kind.extensions()).pick_file()
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = (kind, title, filter_name);
        None
    }
}

/// Archivos de una clase que hay en la carpeta de la app (para el móvil), los más nuevos primero.
pub fn list_files(kind: FileKind) -> Vec<PathBuf> {
    let dir = sub_dir(kind.mobile_folder());
    let mut v: Vec<(std::time::SystemTime, PathBuf)> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .map(|e| kind.extensions().contains(&e.to_lowercase().as_str()))
                .unwrap_or(false)
        })
        .map(|p| (std::fs::metadata(&p).and_then(|m| m.modified()).unwrap_or(std::time::UNIX_EPOCH), p))
        .collect();
    v.sort_by(|a, b| b.0.cmp(&a.0));
    v.into_iter().map(|(_, p)| p).collect()
}

/// ¿Se puede copiar una imagen al portapapeles en este sistema?
pub const CAN_COPY_IMAGE: bool = !IS_MOBILE;

/// Copia una imagen RGBA al portapapeles.
pub fn copy_image(width: usize, height: usize, rgba: Vec<u8>) -> Result<(), String> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        arboard::Clipboard::new()
            .and_then(|mut c| {
                c.set_image(arboard::ImageData { width, height, bytes: std::borrow::Cow::Owned(rgba) })
            })
            .map_err(|e| e.to_string())
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = (width, height, rgba);
        Err("clipboard".into())
    }
}

/// Idioma del sistema, por ejemplo "es-ES" o "en-US".
pub fn system_locale() -> String {
    sys_locale::get_locale().unwrap_or_else(|| "es".into())
}

/// Fuente (TTF/OTF) del sistema para la interfaz, si hay alguna conocida.
pub fn ui_font() -> Option<Vec<u8>> {
    const CANDIDATES: &[&str] = &[
        // Windows
        "C:\\Windows\\Fonts\\segoeui.ttf",
        // macOS
        "/System/Library/Fonts/Supplemental/Arial.ttf",
        "/Library/Fonts/Arial.ttf",
        // Android
        "/system/fonts/Roboto-Regular.ttf",
        // Linux
        "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/google-noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
    ];
    CANDIDATES.iter().find_map(|p| std::fs::read(p).ok())
}

/// Fuente gruesa (negrita) del sistema para el texto del marco del QR.
pub fn bold_font() -> Option<Vec<u8>> {
    const CANDIDATES: &[&str] = &[
        "C:\\Windows\\Fonts\\segoeuib.ttf",
        "C:\\Windows\\Fonts\\arialbd.ttf",
        "/System/Library/Fonts/Supplemental/Arial Bold.ttf",
        "/Library/Fonts/Arial Bold.ttf",
        "/system/fonts/Roboto-Bold.ttf",
        "/usr/share/fonts/truetype/noto/NotoSans-Bold.ttf",
        "/usr/share/fonts/noto/NotoSans-Bold.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
        "/usr/share/fonts/TTF/DejaVuSans-Bold.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf",
    ];
    CANDIDATES.iter().find_map(|p| std::fs::read(p).ok())
}
