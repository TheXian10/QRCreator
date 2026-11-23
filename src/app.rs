//! La ventana del programa.
//!
//! Distribución en pantallas anchas (escritorio):
//! ```text
//! ┌──────────────────────── barra superior ────────────────────────┐
//! │ menú │ panel de opciones │          vista previa del QR         │
//! │ (ico)│ (tarjetas)        │   información  ·  botones de guardar │
//! └──────┴───────────────────┴──────────────────────────────────────┘
//! ```
//! En pantallas estrechas (teléfonos) todo va en una sola columna que se desplaza:
//! vista previa arriba, después las pestañas y las opciones.
//!
//! Todo el texto visible pasa por `tr()` / `trf()` (módulo `i18n`) para poder mostrarse en
//! español, inglés, portugués o francés.

use crate::content::{Content, Kind, WifiSec};
use crate::i18n::{self, tr, trf, Lang};
use crate::platform::{self, FileKind};
use crate::qr::{self, QrError, QrResult};
use crate::render;
use crate::style::*;
use eframe::egui::{self, Color32, FontFamily, FontId, RichText, Stroke, Vec2};
use image::RgbaImage;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Ancho en píxeles con el que se dibuja la vista previa.
const PREVIEW_PX: u32 = 720;
/// Color principal de la interfaz (violeta azulado).
const ACCENT: Color32 = Color32::from_rgb(99, 91, 255);
/// Por debajo de este ancho (en puntos) se usa la distribución de una columna.
const COMPACT_WIDTH: f32 = 860.0;

// ───────────────────────────────────────────────────────────── tema claro / oscuro

/// Modo oscuro activo (se lee desde las funciones de dibujo sin pasar el estado).
static DARK: AtomicBool = AtomicBool::new(false);

/// Colores de la interfaz para el tema activo.
struct Pal {
    /// Fondo de los paneles laterales y la barra superior.
    panel: Color32,
    /// Fondo de la zona de la vista previa.
    canvas: Color32,
    /// Fondo de las tarjetas.
    card: Color32,
    /// Borde de las tarjetas.
    card_stroke: Color32,
    /// Fondo de los botones "pastilla" y de las fichas sin elegir.
    field: Color32,
    /// Fondo al pasar el ratón.
    hover: Color32,
    /// Fondo de lo elegido.
    sel_fill: Color32,
    /// Texto principal y títulos.
    text: Color32,
    /// Texto secundario.
    hint: Color32,
    /// Color de los iconos sin elegir.
    icon: Color32,
    warn: Color32,
    good: Color32,
    bad: Color32,
    /// Cuadros del tablero que se ve detrás de un QR transparente.
    checker: Color32,
    shadow: Color32,
}

fn pal() -> Pal {
    if DARK.load(Ordering::Relaxed) {
        Pal {
            panel: Color32::from_rgb(24, 25, 33),
            canvas: Color32::from_rgb(16, 17, 23),
            card: Color32::from_rgb(32, 34, 45),
            card_stroke: Color32::from_rgb(48, 51, 66),
            field: Color32::from_rgb(42, 44, 58),
            hover: Color32::from_rgb(54, 57, 75),
            sel_fill: Color32::from_rgb(48, 46, 96),
            text: Color32::from_rgb(232, 233, 245),
            hint: Color32::from_rgb(146, 149, 172),
            icon: Color32::from_rgb(190, 193, 215),
            warn: Color32::from_rgb(245, 185, 80),
            good: Color32::from_rgb(92, 214, 140),
            bad: Color32::from_rgb(255, 125, 105),
            checker: Color32::from_rgb(218, 218, 226),
            shadow: Color32::from_black_alpha(110),
        }
    } else {
        Pal {
            panel: Color32::WHITE,
            canvas: Color32::from_rgb(243, 244, 249),
            card: Color32::WHITE,
            card_stroke: Color32::from_rgb(230, 231, 240),
            field: Color32::from_rgb(244, 245, 250),
            hover: Color32::from_rgb(234, 235, 247),
            sel_fill: Color32::from_rgb(238, 237, 255),
            text: Color32::from_rgb(28, 30, 48),
            hint: Color32::from_rgb(112, 116, 138),
            icon: Color32::from_rgb(70, 74, 98),
            warn: Color32::from_rgb(185, 110, 0),
            good: Color32::from_rgb(22, 140, 76),
            bad: Color32::from_rgb(205, 64, 44),
            checker: Color32::from_rgb(236, 237, 242),
            shadow: Color32::from_black_alpha(22),
        }
    }
}

/// Aplica el tema (colores, esquinas, espacios) a toda la interfaz.
fn apply_theme(ctx: &egui::Context, dark: bool) {
    DARK.store(dark, Ordering::Relaxed);
    let p = pal();
    let mut v = if dark { egui::Visuals::dark() } else { egui::Visuals::light() };
    v.selection.bg_fill = ACCENT;
    v.selection.stroke = Stroke::new(1.0f32, Color32::WHITE);
    v.hyperlink_color = if dark { Color32::from_rgb(160, 155, 255) } else { ACCENT };
    v.panel_fill = p.panel;
    v.window_fill = p.card;
    v.window_stroke = Stroke::new(1.0f32, p.card_stroke);
    v.extreme_bg_color = p.field;
    v.faint_bg_color = p.field;
    v.override_text_color = Some(p.text);
    let radius = egui::CornerRadius::same(8);
    for w in [
        &mut v.widgets.noninteractive,
        &mut v.widgets.inactive,
        &mut v.widgets.hovered,
        &mut v.widgets.active,
        &mut v.widgets.open,
    ] {
        w.corner_radius = radius;
    }
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0f32, p.card_stroke);
    v.widgets.inactive.weak_bg_fill = p.field;
    v.widgets.inactive.bg_fill = p.field;
    v.widgets.inactive.bg_stroke = Stroke::NONE;
    v.widgets.hovered.weak_bg_fill = p.hover;
    v.widgets.hovered.bg_fill = p.hover;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0f32, ACCENT.gamma_multiply(0.5));
    v.widgets.active.weak_bg_fill = p.sel_fill;
    v.window_corner_radius = egui::CornerRadius::same(14);
    v.menu_corner_radius = egui::CornerRadius::same(10);
    v.slider_trailing_fill = true;
    ctx.set_visuals(v);
    ctx.style_mut(|s| {
        s.spacing.item_spacing = Vec2::new(8.0, 8.0);
        s.spacing.button_padding = Vec2::new(12.0, 6.0);
        s.spacing.interact_size.y = 30.0;
        s.spacing.slider_width = 180.0;
        s.spacing.combo_width = 200.0;
        s.text_styles.insert(egui::TextStyle::Body, FontId::proportional(14.5));
        s.text_styles.insert(egui::TextStyle::Button, FontId::proportional(14.5));
        s.text_styles.insert(egui::TextStyle::Small, FontId::proportional(12.0));
        s.text_styles.insert(egui::TextStyle::Heading, FontId::new(20.0, bold()));
    });
}

/// Familia de letra gruesa (se registra en `install_fonts`).
fn bold() -> FontFamily {
    FontFamily::Name("negrita".into())
}

/// Carga la letra del sistema para la interfaz (más legible que la de serie) y una variante
/// gruesa para los títulos. Los iconos siguen saliendo de las fuentes de egui.
fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let defaults = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    if let Some(bytes) = platform::ui_font() {
        fonts.font_data.insert("sistema".into(), Arc::new(egui::FontData::from_owned(bytes)));
        if let Some(f) = fonts.families.get_mut(&FontFamily::Proportional) {
            f.insert(0, "sistema".into());
        }
    }
    let mut bold_list = defaults;
    if let Some(bytes) = platform::bold_font() {
        fonts.font_data.insert("sistema-negrita".into(), Arc::new(egui::FontData::from_owned(bytes)));
        bold_list.insert(0, "sistema-negrita".into());
    }
    fonts.families.insert(bold(), bold_list);
    ctx.set_fonts(fonts);
}

// ───────────────────────────────────────────────────────────── ajustes guardados

/// Preferencias que se recuerdan entre usos (tema e idioma).
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
struct Settings {
    modo_oscuro: bool,
    idioma: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self { modo_oscuro: false, idioma: Lang::from_locale(&platform::system_locale()).code().into() }
    }
}

fn settings_path() -> PathBuf {
    platform::data_dir().join("ajustes.json")
}

fn load_settings() -> Settings {
    std::fs::read_to_string(settings_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_settings(s: &Settings) {
    let p = settings_path();
    if let Some(dir) = p.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(txt) = serde_json::to_string_pretty(s) {
        let _ = std::fs::write(p, txt);
    }
}

// ───────────────────────────────────────────────────────────── estado

/// Las secciones de opciones.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Gallery,
    Content,
    Shapes,
    Colors,
    Logo,
    Frame,
    Rules,
}

impl Tab {
    const ALL: [Tab; 7] = [Tab::Gallery, Tab::Content, Tab::Shapes, Tab::Colors, Tab::Logo, Tab::Frame, Tab::Rules];
    fn icon(self) -> &'static str {
        match self {
            Tab::Gallery => "✨",
            Tab::Content => "📝",
            Tab::Shapes => "◼",
            Tab::Colors => "🎨",
            Tab::Logo => "🖼",
            Tab::Frame => "🏷",
            Tab::Rules => "⚙",
        }
    }
    fn label(self) -> &'static str {
        match self {
            Tab::Gallery => tr("Diseños"),
            Tab::Content => tr("Contenido"),
            Tab::Shapes => tr("Formas"),
            Tab::Colors => tr("Colores"),
            Tab::Logo => tr("Logo"),
            Tab::Frame => tr("Marco"),
            Tab::Rules => tr("Reglas"),
        }
    }
    fn subtitle(self) -> &'static str {
        match self {
            Tab::Gallery => tr("Elige un diseño listo y luego personalízalo"),
            Tab::Content => tr("Lo que se abrirá al escanear el código"),
            Tab::Shapes => tr("La forma de los puntos y de las esquinas"),
            Tab::Colors => tr("Colores sólidos o degradados"),
            Tab::Logo => tr("Tu imagen en el centro del código"),
            Tab::Frame => tr("Una etiqueta que invite a escanear"),
            Tab::Rules => tr("Ajustes técnicos de la norma QR"),
        }
    }
}

/// Resultado de comprobar si el QR se puede leer.
enum Check {
    Pending(Instant),
    Ok(render::ReadKind),
    Fail(render::ReadError),
    None,
}

/// Captura de pantalla pedida con `--screenshot` (para las comprobaciones automáticas).
struct Shot {
    path: PathBuf,
    frames: u32,
    requested: bool,
}

/// Mensaje breve que aparece abajo a la derecha.
struct Toast {
    text: String,
    ok: bool,
    since: Instant,
}

/// Iconos dibujados por el propio programa (blancos; se tiñen al pintarlos).
struct Icons {
    modules: Vec<egui::TextureHandle>,
    frames: Vec<egui::TextureHandle>,
    balls: Vec<egui::TextureHandle>,
}

pub struct QrApp {
    settings: Settings,
    tab: Tab,
    content: Content,
    rules: Rules,
    style: Style,
    logo: Option<RgbaImage>,
    logo_name: String,
    /// Cambia cada vez que cambia el logo (para saber cuándo redibujar).
    logo_gen: u64,
    /// Letra gruesa para el texto del marco del QR.
    font: ab_glyph::FontArc,
    qr: Result<QrResult, QrError>,
    payload: String,
    last_key: Option<(String, Rules, Style, u64)>,
    tex: Option<egui::TextureHandle>,
    preview: Option<RgbaImage>,
    scene: Option<render::Scene>,
    check: Check,
    export_px: u32,
    toast: Option<Toast>,
    shot: Option<Shot>,
    icons: Icons,
    thumbs: Vec<egui::TextureHandle>,
    thumbs_key: Option<(String, Rules, u64, Lang)>,
    /// En el móvil: lista de archivos abierta para elegir (logo o diseño).
    file_list: Option<FileKind>,
}

/// Letra gruesa para el texto del marco del QR (la del sistema o una incluida).
fn load_frame_font() -> ab_glyph::FontArc {
    platform::bold_font()
        .and_then(|b| ab_glyph::FontArc::try_from_vec(b).ok())
        .unwrap_or_else(|| ab_glyph::FontArc::try_from_slice(epaint_default_fonts::UBUNTU_LIGHT).expect("fuente interna"))
}

/// Sube una imagen como textura de egui.
fn to_texture(ctx: &egui::Context, name: &str, img: &RgbaImage) -> egui::TextureHandle {
    let ci = egui::ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], img.as_raw());
    ctx.load_texture(name, ci, egui::TextureOptions::LINEAR)
}

/// Un diseño de la galería con su texto de marco traducido al idioma actual.
fn preset_style(p: &Preset) -> Style {
    let mut st = (p.style)();
    st.frame_text = tr(&st.frame_text).to_string();
    st
}

impl QrApp {
    pub fn new(cc: &eframe::CreationContext<'_>, shot: Option<PathBuf>) -> Self {
        let args: Vec<String> = std::env::args().collect();
        let has = |a: &str| args.iter().any(|x| x == a);

        // Preferencias (los argumentos de prueba mandan sobre lo guardado).
        let mut settings = load_settings();
        if has("--modo-oscuro") {
            settings.modo_oscuro = true;
        }
        if has("--modo-claro") {
            settings.modo_oscuro = false;
        }
        if let Some(i) = args.iter().position(|a| a == "--idioma") {
            if let Some(code) = args.get(i + 1) {
                settings.idioma = Lang::from_locale(code).code().into();
            }
        }
        i18n::set_lang(Lang::from_locale(&settings.idioma));

        install_fonts(&cc.egui_ctx);
        apply_theme(&cc.egui_ctx, settings.modo_oscuro);

        // Iconos de formas, marcos y centros de los ojos.
        let font = load_frame_font();
        let ctx = &cc.egui_ctx;
        let icons = Icons {
            modules: ModuleShape::ALL
                .iter()
                .enumerate()
                .map(|(i, s)| to_texture(ctx, &format!("ico-mod-{i}"), &render::module_icon(*s, &font, 96)))
                .collect(),
            frames: EyeFrame::ALL
                .iter()
                .enumerate()
                .map(|(i, s)| to_texture(ctx, &format!("ico-ojo-{i}"), &render::eye_frame_icon(*s, &font, 96)))
                .collect(),
            balls: EyeBall::ALL
                .iter()
                .enumerate()
                .map(|(i, s)| to_texture(ctx, &format!("ico-centro-{i}"), &render::eye_ball_icon(*s, &font, 96)))
                .collect(),
        };

        // `--pestana <n>` abre directamente una sección (útil para las capturas de prueba).
        let tab = if has("--galeria") {
            Tab::Gallery
        } else {
            args.iter()
                .position(|a| a == "--pestana")
                .and_then(|i| args.get(i + 1))
                .and_then(|n| n.parse::<usize>().ok())
                .and_then(|n| Tab::ALL.get(n).copied())
                .unwrap_or(Tab::Content)
        };
        Self {
            settings,
            tab,
            content: Content::default(),
            rules: Rules { ecc: Ecc::Q, ..Rules::default() },
            style: preset_style(&PRESETS[1]),
            logo: None,
            logo_name: String::new(),
            logo_gen: 0,
            font,
            qr: Err(QrError::Empty),
            payload: String::new(),
            last_key: None,
            tex: None,
            preview: None,
            scene: None,
            check: Check::None,
            export_px: 1024,
            toast: None,
            shot: shot.map(|path| Shot { path, frames: 0, requested: false }),
            icons,
            thumbs: Vec::new(),
            thumbs_key: None,
            file_list: None,
        }
    }

    /// Enseña un mensaje breve.
    fn say(&mut self, text: impl Into<String>, ok: bool) {
        self.toast = Some(Toast { text: text.into(), ok, since: Instant::now() });
    }

    /// Cambia el idioma y traduce también el texto del marco si es uno de los sugeridos.
    fn set_lang(&mut self, lang: Lang) {
        let old = i18n::lang();
        if old == lang {
            return;
        }
        self.style.frame_text = i18n::retranslate(&self.style.frame_text, old, lang);
        i18n::set_lang(lang);
        self.settings.idioma = lang.code().into();
        save_settings(&self.settings);
    }

    fn set_dark(&mut self, ctx: &egui::Context, dark: bool) {
        self.settings.modo_oscuro = dark;
        apply_theme(ctx, dark);
        save_settings(&self.settings);
    }

    // ─────────────────────────────────────────────────── dibujo y comprobación

    /// Vuelve a crear el QR y la vista previa si algo cambió.
    fn refresh(&mut self, ctx: &egui::Context) {
        let payload = self.content.payload();
        let key = (payload.clone(), self.rules.clone(), self.style.clone(), self.logo_gen);
        if self.last_key.as_ref() == Some(&key) {
            return;
        }
        self.last_key = Some(key);
        self.payload = payload;
        self.qr = qr::encode(&self.payload, &self.rules);
        match &self.qr {
            Ok(q) => {
                let scene = render::build_scene(q, &self.style, &self.rules, self.logo.is_some());
                let img = render::rasterize(&scene, self.logo.as_ref(), &self.font, PREVIEW_PX);
                let ci = egui::ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], img.as_raw());
                match &mut self.tex {
                    Some(t) => t.set(ci, egui::TextureOptions::LINEAR),
                    None => self.tex = Some(ctx.load_texture("vista-previa", ci, egui::TextureOptions::LINEAR)),
                }
                self.preview = Some(img);
                self.scene = Some(scene);
                self.check = Check::Pending(Instant::now());
            }
            Err(_) => {
                self.preview = None;
                self.check = Check::None;
            }
        }
    }

    /// Comprueba si el QR se puede leer cuando la persona deja de tocar los controles
    /// (espera un momento para no hacerlo en cada movimiento de un deslizador).
    fn run_check(&mut self, ctx: &egui::Context) {
        if let Check::Pending(t) = self.check {
            let wait = if self.shot.is_some() { Duration::ZERO } else { Duration::from_millis(350) };
            if t.elapsed() >= wait {
                if let (Some(img), Some(scene)) = (&self.preview, &self.scene) {
                    self.check = match render::verify(img, scene, &self.payload) {
                        Ok(k) => Check::Ok(k),
                        Err(e) => Check::Fail(e),
                    };
                }
            } else {
                ctx.request_repaint_after(wait - t.elapsed());
            }
        }
    }

    /// Dibuja el QR al tamaño de exportación.
    fn render_export(&self, px: u32) -> Option<RgbaImage> {
        let q = self.qr.as_ref().ok()?;
        let scene = render::build_scene(q, &self.style, &self.rules, self.logo.is_some());
        Some(render::rasterize(&scene, self.logo.as_ref(), &self.font, px))
    }

    /// Mira qué ha pasado al guardar y lo cuenta.
    fn report_saved(&mut self, res: Result<(), String>, path: &Path) {
        match res {
            Ok(()) => self.say(trf("Guardado en {0}", &[&path.display()]), true),
            Err(e) => self.say(trf("No se pudo guardar: {0}", &[&e]), false),
        }
    }

    fn save_png(&mut self) {
        let Some(path) = platform::pick_save(FileKind::Png, tr("Guardar QR como PNG"), "codigo-qr.png", tr("Imagen PNG")) else {
            return;
        };
        let res = match self.render_export(self.export_px) {
            Some(img) => img.save_with_format(&path, image::ImageFormat::Png).map_err(|e| e.to_string()),
            None => Err(tr("No hay un QR válido.").to_string()),
        };
        self.report_saved(res, &path);
    }

    fn save_svg(&mut self) {
        let Ok(q) = &self.qr else { return };
        let scene = render::build_scene(q, &self.style, &self.rules, self.logo.is_some());
        let svg = render::to_svg(&scene, self.logo.as_ref(), self.export_px);
        let Some(path) = platform::pick_save(FileKind::Svg, tr("Guardar QR como SVG"), "codigo-qr.svg", tr("Imagen vectorial SVG")) else {
            return;
        };
        let res = std::fs::write(&path, svg).map_err(|e| e.to_string());
        self.report_saved(res, &path);
    }

    fn copy_image(&mut self) {
        let Some(img) = self.render_export(self.export_px) else { return };
        let (w, h) = (img.width() as usize, img.height() as usize);
        match platform::copy_image(w, h, img.into_raw()) {
            Ok(()) => self.say(tr("Imagen copiada. Pégala donde quieras."), true),
            Err(e) => self.say(trf("No se pudo copiar: {0}", &[&e]), false),
        }
    }

    fn save_design(&mut self) {
        let Some(path) = platform::pick_save(FileKind::Design, tr("Guardar diseño"), "mi-diseno-qr.json", tr("Diseño de QR")) else {
            return;
        };
        let df = DesignFile { style: self.style.clone(), rules: self.rules.clone() };
        let res = serde_json::to_string_pretty(&df)
            .map_err(|e| e.to_string())
            .and_then(|s| std::fs::write(&path, s).map_err(|e| e.to_string()));
        self.report_saved(res, &path);
    }

    /// Abre un diseño guardado (en el móvil, desde la lista de archivos).
    fn ask_load_design(&mut self) {
        if platform::IS_MOBILE {
            self.file_list = Some(FileKind::Design);
        } else if let Some(p) = platform::pick_open(FileKind::Design, tr("Cargar diseño"), tr("Diseño de QR")) {
            self.load_design(&p);
        }
    }

    fn load_design(&mut self, path: &Path) {
        match std::fs::read_to_string(path)
            .map_err(|e| e.to_string())
            .and_then(|s| serde_json::from_str::<DesignFile>(&s).map_err(|e| e.to_string()))
        {
            Ok(df) => {
                self.style = df.style;
                self.rules = df.rules;
                self.say(tr("Diseño cargado."), true);
            }
            Err(e) => self.say(trf("Ese archivo no es un diseño válido: {0}", &[&e]), false),
        }
    }

    /// Pone una imagen como logo (se reduce si es enorme) y sube la corrección a H.
    fn set_logo(&mut self, img: RgbaImage, name: String) {
        let img = if img.width().max(img.height()) > 1024 {
            let k = 1024.0 / img.width().max(img.height()) as f32;
            image::imageops::resize(
                &img,
                ((img.width() as f32 * k) as u32).max(1),
                ((img.height() as f32 * k) as u32).max(1),
                image::imageops::FilterType::Lanczos3,
            )
        } else {
            img
        };
        self.logo = Some(img);
        self.logo_name = name;
        self.logo_gen += 1;
        if self.rules.ecc != Ecc::H {
            self.rules.ecc = Ecc::H;
            self.say(tr("Subí la corrección de errores a H para que el logo no impida leer el código."), true);
        }
    }

    fn ask_logo(&mut self) {
        if platform::IS_MOBILE {
            self.file_list = Some(FileKind::Image);
        } else if let Some(p) = platform::pick_open(FileKind::Image, tr("Elegir logo"), tr("Imágenes")) {
            self.open_logo(&p);
        }
    }

    fn open_logo(&mut self, path: &Path) {
        match image::open(path) {
            Ok(img) => {
                let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                self.set_logo(img.to_rgba8(), name);
            }
            Err(e) => self.say(trf("No se pudo abrir la imagen: {0}", &[&e]), false),
        }
    }

    /// Dibuja una miniatura de cada diseño de la galería con el contenido actual.
    fn refresh_thumbs(&mut self, ctx: &egui::Context) {
        let Ok(q) = &self.qr else { return };
        let key = (self.payload.clone(), self.rules.clone(), self.logo_gen, i18n::lang());
        if self.thumbs_key.as_ref() == Some(&key) && self.thumbs.len() == PRESETS.len() {
            return;
        }
        self.thumbs.clear();
        for (i, p) in PRESETS.iter().enumerate() {
            let st = preset_style(p);
            let scene = render::build_scene(q, &st, &self.rules, self.logo.is_some());
            let img = render::rasterize(&scene, self.logo.as_ref(), &self.font, 240);
            self.thumbs.push(to_texture(ctx, &format!("miniatura-{i}"), &img));
        }
        self.thumbs_key = Some(key);
    }

    // ─────────────────────────────────────────────────── barra superior y navegación

    fn top_bar(&mut self, ui: &mut egui::Ui, compact: bool) {
        ui.horizontal(|ui| {
            app_logo(ui, 32.0);
            ui.add_space(2.0);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.label(RichText::new(tr("Creador de QR")).family(bold()).size(18.0));
                if !compact {
                    ui.label(RichText::new(tr("Todo se hace en tu dispositivo, sin internet")).small().color(pal().hint));
                }
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Tema
                let (icon, tip) = if self.settings.modo_oscuro { ("☀", tr("Modo claro")) } else { ("🌙", tr("Modo oscuro")) };
                if icon_button(ui, icon).on_hover_text(tip).clicked() {
                    let d = !self.settings.modo_oscuro;
                    self.set_dark(ui.ctx(), d);
                }
                // Idioma
                let mut lang = i18n::lang();
                egui::ComboBox::from_id_salt("idioma")
                    .width(if compact { 60.0 } else { 130.0 })
                    .selected_text(if compact { format!("🌐 {}", lang.code().to_uppercase()) } else { format!("🌐 {}", lang.name()) })
                    .show_ui(ui, |ui| {
                        for l in Lang::ALL {
                            ui.selectable_value(&mut lang, l, l.name());
                        }
                    });
                self.set_lang(lang);
                ui.add_space(4.0);
                // Diseños guardados
                // Diseños guardados en un menú desplegable (más limpio que dos botones).
                ui.menu_button(if compact { "☰".to_string() } else { format!("☰  {}", tr("Mis diseños")) }, |ui| {
                    ui.set_min_width(200.0);
                    if ui.button(format!("💾  {}", tr("Guardar diseño"))).clicked() {
                        ui.close_menu();
                        self.save_design();
                    }
                    if ui.button(format!("📂  {}", tr("Cargar diseño"))).clicked() {
                        ui.close_menu();
                        self.ask_load_design();
                    }
                    ui.separator();
                    if ui.button(format!("↺  {}", tr("Empezar de nuevo"))).clicked() {
                        ui.close_menu();
                        self.style = preset_style(&PRESETS[0]);
                        self.rules = Rules::default();
                        self.logo = None;
                        self.logo_gen += 1;
                    }
                });
            });
        });
    }

    /// Menú vertical de iconos (pantallas anchas).
    fn nav_rail(&mut self, ui: &mut egui::Ui) {
        let p = pal();
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.spacing_mut().item_spacing.y = 4.0;
            for t in Tab::ALL {
                let sel = self.tab == t;
                let (rect, resp) = ui.allocate_exact_size(Vec2::new(76.0, 62.0), egui::Sense::click());
                let painter = ui.painter();
                if sel {
                    painter.rect_filled(rect.shrink2(Vec2::new(6.0, 2.0)), 12.0, p.sel_fill);
                } else if resp.hovered() {
                    painter.rect_filled(rect.shrink2(Vec2::new(6.0, 2.0)), 12.0, p.hover);
                }
                let col = if sel { ACCENT } else { p.icon };
                painter.text(rect.center() - Vec2::new(0.0, 9.0), egui::Align2::CENTER_CENTER, t.icon(), FontId::proportional(20.0), col);
                painter.text(
                    rect.center() + Vec2::new(0.0, 15.0),
                    egui::Align2::CENTER_CENTER,
                    t.label(),
                    FontId::proportional(11.5),
                    if sel { ACCENT } else { p.hint },
                );
                if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                    self.tab = t;
                }
            }
        });
    }

    /// Pestañas en fila (pantallas estrechas).
    fn tab_strip(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::horizontal().id_salt("pestanas").show(ui, |ui| {
            ui.horizontal(|ui| {
                for t in Tab::ALL {
                    if pill(ui, &format!("{} {}", t.icon(), t.label()), self.tab == t).clicked() {
                        self.tab = t;
                    }
                }
            });
        });
    }

    /// El contenido de la pestaña elegida.
    fn options(&mut self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        ui.label(RichText::new(self.tab.label()).heading());
        ui.label(RichText::new(self.tab.subtitle()).color(pal().hint));
        ui.add_space(6.0);
        match self.tab {
            Tab::Gallery => self.tab_gallery(ui),
            Tab::Content => self.tab_content(ui),
            Tab::Shapes => self.tab_shapes(ui),
            Tab::Colors => self.tab_colors(ui),
            Tab::Logo => self.tab_logo(ui),
            Tab::Frame => self.tab_frame(ui),
            Tab::Rules => self.tab_rules(ui),
        }
        ui.add_space(24.0);
    }

    // ─────────────────────────────────────────────────── pestañas

    fn tab_gallery(&mut self, ui: &mut egui::Ui) {
        if self.qr.is_err() {
            warn(ui, tr("Escribe un contenido válido para ver las miniaturas."));
            return;
        }
        self.refresh_thumbs(ui.ctx());
        let p = pal();
        let gap = 10.0;
        let avail = ui.available_width();
        let cols = ((avail + gap) / 118.0).floor().max(2.0) as usize;
        let cell = (avail - gap * (cols as f32 - 1.0)) / cols as f32;
        let mut chosen = None;
        egui::Grid::new("galeria").spacing([gap, gap]).show(ui, |ui| {
            for (i, preset) in PRESETS.iter().enumerate() {
                let st = preset_style(preset);
                let sel = self.style == st;
                let (rect, resp) = ui.allocate_exact_size(Vec2::new(cell, cell + 24.0), egui::Sense::click());
                let hover = resp.hovered();
                let painter = ui.painter_at(rect);
                painter.rect_filled(rect, 12.0, if sel { p.sel_fill } else if hover { p.hover } else { p.field });
                if sel || hover {
                    painter.rect_stroke(rect, 12.0, Stroke::new(if sel { 2.0f32 } else { 1.0 }, ACCENT), egui::StrokeKind::Inside);
                }
                if let Some(tex) = self.thumbs.get(i) {
                    let ts = tex.size_vec2();
                    let room = Vec2::new(cell - 16.0, cell - 12.0);
                    let k = (room.x / ts.x).min(room.y / ts.y);
                    let r = egui::Rect::from_center_size(egui::pos2(rect.center().x, rect.top() + 8.0 + room.y * 0.5), ts * k);
                    painter.image(tex.id(), r, full_uv(), Color32::WHITE);
                }
                painter.text(
                    egui::pos2(rect.center().x, rect.bottom() - 13.0),
                    egui::Align2::CENTER_CENTER,
                    tr(preset.name),
                    FontId::proportional(13.0),
                    if sel { ACCENT } else { p.text },
                );
                if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                    chosen = Some(st);
                }
                if (i + 1) % cols == 0 {
                    ui.end_row();
                }
            }
        });
        if let Some(st) = chosen {
            self.style = st;
        }
    }

    fn tab_content(&mut self, ui: &mut egui::Ui) {
        card(ui, tr("¿Qué quieres que abra el QR?"), |ui| {
            ui.horizontal_wrapped(|ui| {
                for k in Kind::ALL {
                    if pill(ui, &format!("{} {}", k.icon(), tr(k.label())), self.content.kind == k).clicked() {
                        self.content.kind = k;
                    }
                }
            });
        });
        let c = &mut self.content;
        card(ui, tr("Datos"), |ui| {
            let w = ui.available_width().min(420.0);
            // Un campo de una línea con su nombre encima.
            let field = |ui: &mut egui::Ui, label: &str, v: &mut String, hint: &str| {
                ui.label(RichText::new(label).color(pal().hint).small());
                ui.add(egui::TextEdit::singleline(v).hint_text(hint).desired_width(w).margin(Vec2::new(8.0, 6.0)));
            };
            let area = |ui: &mut egui::Ui, label: &str, v: &mut String, rows: usize| {
                ui.label(RichText::new(label).color(pal().hint).small());
                ui.add(egui::TextEdit::multiline(v).desired_rows(rows).desired_width(w).margin(Vec2::new(8.0, 6.0)));
            };
            match c.kind {
                Kind::Text => area(ui, tr("Texto"), &mut c.text, 5),
                Kind::Url => field(ui, tr("Dirección web"), &mut c.url, "https://..."),
                Kind::Wifi => {
                    field(ui, tr("Nombre de la red"), &mut c.wifi_ssid, "SSID");
                    ui.label(RichText::new(tr("Seguridad")).color(pal().hint).small());
                    ui.horizontal_wrapped(|ui| {
                        for (v, l) in [(WifiSec::Wpa, "WPA/WPA2/WPA3"), (WifiSec::Wep, "WEP"), (WifiSec::Open, tr("Abierta"))] {
                            if pill(ui, l, c.wifi_sec == v).clicked() {
                                c.wifi_sec = v;
                            }
                        }
                    });
                    if c.wifi_sec != WifiSec::Open {
                        field(ui, tr("Contraseña"), &mut c.wifi_pass, "");
                    }
                    ui.checkbox(&mut c.wifi_hidden, tr("Red oculta"));
                }
                Kind::Contact => {
                    field(ui, tr("Nombre"), &mut c.c_name, "");
                    field(ui, tr("Apellidos"), &mut c.c_surname, "");
                    field(ui, tr("Teléfono"), &mut c.c_phone, "+34 ...");
                    field(ui, tr("Correo"), &mut c.c_email, "");
                    field(ui, tr("Empresa"), &mut c.c_org, tr("opcional"));
                    field(ui, tr("Cargo"), &mut c.c_title, tr("opcional"));
                    field(ui, tr("Web"), &mut c.c_web, tr("opcional"));
                    field(ui, tr("Dirección"), &mut c.c_address, tr("opcional"));
                }
                Kind::Email => {
                    field(ui, tr("Para"), &mut c.mail_to, "correo@...");
                    field(ui, tr("Asunto"), &mut c.mail_subject, tr("opcional"));
                    area(ui, tr("Mensaje"), &mut c.mail_body, 3);
                }
                Kind::Phone => field(ui, tr("Número"), &mut c.phone, "+34 ..."),
                Kind::Sms => {
                    field(ui, tr("Número"), &mut c.phone, "+34 ...");
                    area(ui, tr("Mensaje"), &mut c.sms_msg, 3);
                }
                Kind::WhatsApp => {
                    field(ui, tr("Número (con prefijo)"), &mut c.phone, "34600000000");
                    area(ui, tr("Mensaje"), &mut c.wa_msg, 3);
                }
                Kind::Location => {
                    field(ui, tr("Latitud"), &mut c.lat, "40.4168");
                    field(ui, tr("Longitud"), &mut c.lon, "-3.7038");
                }
                Kind::Event => {
                    field(ui, tr("Título"), &mut c.ev_title, "");
                    field(ui, tr("Lugar"), &mut c.ev_place, tr("opcional"));
                    field(ui, tr("Empieza"), &mut c.ev_start, tr("AAAA-MM-DD HH:MM"));
                    field(ui, tr("Termina"), &mut c.ev_end, tr("AAAA-MM-DD HH:MM"));
                    area(ui, tr("Descripción"), &mut c.ev_desc, 2);
                }
            }
        });
        egui::CollapsingHeader::new(tr("Ver el texto exacto que llevará el QR")).show(ui, |ui| {
            let mut p = self.content.payload();
            ui.add(
                egui::TextEdit::multiline(&mut p)
                    .desired_rows(4)
                    .desired_width(f32::INFINITY)
                    .interactive(false)
                    .code_editor(),
            );
            hint(ui, &trf("{0} caracteres · {1} bytes", &[&p.chars().count(), &p.len()]));
        });
    }

    fn tab_shapes(&mut self, ui: &mut egui::Ui) {
        let s = &mut self.style;
        let icons = &self.icons;
        card(ui, tr("Forma de los módulos"), |ui| {
            let items: Vec<_> = ModuleShape::ALL
                .iter()
                .zip(&icons.modules)
                .map(|(m, t)| (*m, t.id(), tr(m.label())))
                .collect();
            tile_picker(ui, &items, &mut s.module_shape);
            ui.add_space(4.0);
            slider_row(ui, tr("Tamaño del módulo"), |ui| {
                ui.add(egui::Slider::new(&mut s.module_scale, 0.5..=1.0).custom_formatter(|v, _| format!("{:.0} %", v * 100.0)))
            });
            hint(ui, tr("Más pequeño deja huecos entre módulos (más ligero), pero cuesta un poco más de leer."));
        });
        card(ui, tr("Marco de los ojos (esquinas)"), |ui| {
            let items: Vec<_> = EyeFrame::ALL.iter().zip(&icons.frames).map(|(m, t)| (*m, t.id(), tr(m.label()))).collect();
            tile_picker(ui, &items, &mut s.eye_frame);
        });
        card(ui, tr("Centro de los ojos"), |ui| {
            let items: Vec<_> = EyeBall::ALL.iter().zip(&icons.balls).map(|(m, t)| (*m, t.id(), tr(m.label()))).collect();
            tile_picker(ui, &items, &mut s.eye_ball);
        });
        card(ui, tr("Esquinas del fondo"), |ui| {
            slider_row(ui, tr("Redondeo"), |ui| ui.add(egui::Slider::new(&mut s.bg_radius, 0.0..=8.0)));
        });
    }

    fn tab_colors(&mut self, ui: &mut egui::Ui) {
        let s = &mut self.style;
        card(ui, tr("Color del código"), |ui| {
            fill_editor(ui, "fg", &mut s.fg, false, &FG_SWATCHES);
        });
        card(ui, tr("Color de los ojos"), |ui| {
            ui.checkbox(&mut s.eyes_custom, tr("Usar colores propios para los ojos"));
            if s.eyes_custom {
                ui.horizontal(|ui| {
                    ui.label(tr("Marco"));
                    color_btn(ui, &mut s.eye_frame_color);
                    ui.add_space(16.0);
                    ui.label(tr("Centro"));
                    color_btn(ui, &mut s.eye_ball_color);
                });
            }
        });
        card(ui, tr("Fondo"), |ui| {
            fill_editor(ui, "bg", &mut s.bg, true, &BG_SWATCHES);
        });
        card(ui, tr("Legibilidad"), |ui| {
            if ui.button(format!("⇄  {}", tr("Intercambiar código y fondo"))).clicked() && s.bg.mode != FillMode::Transparent {
                std::mem::swap(&mut s.fg, &mut s.bg);
            }
            contrast_notes(ui, s);
        });
    }

    fn tab_logo(&mut self, ui: &mut egui::Ui) {
        let mut ask = false;
        let mut sample = false;
        let mut remove = false;
        card(ui, tr("Logo en el centro"), |ui| {
            ui.horizontal_wrapped(|ui| {
                ask = ui.add(primary(&format!("📂 {}", tr("Elegir imagen…")))).clicked();
                sample = ui.button(format!("⭐ {}", tr("Logo de ejemplo"))).clicked();
                if self.logo.is_some() {
                    remove = ui.button(format!("✖ {}", tr("Quitar"))).clicked();
                }
            });
            if self.logo.is_some() {
                hint(ui, &trf("Imagen: {0}", &[&self.logo_name]));
            } else {
                hint(ui, tr("Puedes poner tu logo o una imagen (PNG con transparencia queda mejor)."));
            }
            if platform::IS_MOBILE {
                hint(ui, &trf("En el móvil, copia tus imágenes a esta carpeta: {0}", &[&platform::sub_dir("Logos").display()]));
            }
        });
        if ask {
            self.ask_logo();
        }
        if sample {
            self.set_logo(render::sample_logo(), tr("ejemplo").into());
        }
        if remove {
            self.logo = None;
            self.logo_gen += 1;
        }
        if self.logo.is_none() {
            return;
        }
        let s = &mut self.style;
        card(ui, tr("Tamaño y espacio"), |ui| {
            slider_row(ui, tr("Tamaño"), |ui| {
                ui.add(egui::Slider::new(&mut s.logo_size, 0.1..=0.35).custom_formatter(|v, _| format!("{:.0} %", v * 100.0)))
            });
            ui.checkbox(&mut s.logo_clear, tr("Quitar los módulos que quedan debajo del logo"));
            slider_row(ui, tr("Espacio alrededor"), |ui| ui.add(egui::Slider::new(&mut s.logo_padding, 0.0..=3.0)));
        });
        card(ui, tr("Fondo del logo"), |ui| {
            ui.horizontal_wrapped(|ui| {
                for b in LogoBg::ALL {
                    if pill(ui, tr(b.label()), s.logo_bg == b).clicked() {
                        s.logo_bg = b;
                    }
                }
            });
            if s.logo_bg != LogoBg::None {
                ui.horizontal(|ui| {
                    ui.label(tr("Color"));
                    color_btn(ui, &mut s.logo_bg_color);
                });
            }
        });
        if let Ok(q) = &self.qr {
            let pad = if s.logo_clear { s.logo_padding * 2.0 } else { 0.0 };
            let covered = ((s.logo_size * q.size as f32 + pad) / q.size as f32).powi(2);
            card(ui, tr("Legibilidad"), |ui| {
                ui.label(trf(
                    "El logo tapa ≈{0} % del código. Con corrección {1} se aguanta hasta ≈{2} %.",
                    &[&format!("{:.0}", covered * 100.0), &q.ecc.short(), &format!("{:.0}", q.ecc.recovery() * 100.0)],
                ));
                if covered > q.ecc.recovery() * 0.75 {
                    warn(ui, tr("El logo es grande para este nivel de corrección: hazlo más pequeño o sube la corrección a H."));
                }
            });
        }
    }

    fn tab_frame(&mut self, ui: &mut egui::Ui) {
        let s = &mut self.style;
        card(ui, tr("Estilo del marco"), |ui| {
            ui.horizontal_wrapped(|ui| {
                for f in FrameStyle::ALL {
                    if pill(ui, tr(f.label()), s.frame == f).clicked() {
                        s.frame = f;
                    }
                }
            });
            if s.frame == FrameStyle::None {
                hint(ui, tr("Añade una etiqueta como «ESCANÉAME» para invitar a la gente a usarlo."));
            }
        });
        if s.frame == FrameStyle::None {
            return;
        }
        card(ui, tr("Texto"), |ui| {
            ui.add(egui::TextEdit::singleline(&mut s.frame_text).desired_width(ui.available_width().min(360.0)).margin(Vec2::new(8.0, 6.0)));
            ui.horizontal_wrapped(|ui| {
                for t in FRAME_TEXTS {
                    let t = tr(t);
                    if ui.small_button(t).clicked() {
                        s.frame_text = t.into();
                    }
                }
            });
            if s.frame_text.trim().is_empty() && s.frame != FrameStyle::Border {
                hint(ui, tr("Escribe un texto para que aparezca la etiqueta."));
            }
            slider_row(ui, tr("Tamaño del texto"), |ui| ui.add(egui::Slider::new(&mut s.frame_text_size, 0.04..=0.16).show_value(false)));
        });
        card(ui, tr("Colores"), |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(tr("Marco"));
                color_btn(ui, &mut s.frame_color);
                ui.add_space(16.0);
                ui.label(tr("Texto"));
                color_btn(ui, &mut s.frame_text_color);
            });
            if s.frame == FrameStyle::Border {
                slider_row(ui, tr("Grosor del borde"), |ui| ui.add(egui::Slider::new(&mut s.frame_width, 0.5..=6.0)));
            }
        });
    }

    fn tab_rules(&mut self, ui: &mut egui::Ui) {
        let r = &mut self.rules;
        card(ui, tr("Corrección de errores"), |ui| {
            hint(ui, tr("Cuanto más alta, más daño o decoración aguanta el código, pero tendrá más cuadritos."));
            for e in Ecc::ALL {
                ui.radio_value(&mut r.ecc, e, tr(e.label()));
            }
            ui.checkbox(&mut r.boost_ecc, tr("Subirla automáticamente si cabe sin agrandar el código"));
        });
        card(ui, tr("Versión (tamaño)"), |ui| {
            hint(ui, tr("La versión 1 tiene 21×21 módulos y cada versión suma 4 (la 40 tiene 177×177). Se usa la más pequeña donde quepa el contenido."));
            slider_row(ui, tr("Mínima"), |ui| ui.add(egui::Slider::new(&mut r.min_version, 1..=40)));
            slider_row(ui, tr("Máxima"), |ui| ui.add(egui::Slider::new(&mut r.max_version, 1..=40)));
            if r.min_version > r.max_version {
                r.max_version = r.min_version;
            }
        });
        card(ui, tr("Patrón de máscara"), |ui| {
            hint(ui, tr("La norma prueba 8 máscaras y elige la que evita zonas confusas. Puedes fijar una a mano."));
            ui.horizontal_wrapped(|ui| {
                if pill(ui, tr("Automática"), r.mask.is_none()).clicked() {
                    r.mask = None;
                }
                for m in 0..8u8 {
                    if pill(ui, &m.to_string(), r.mask == Some(m)).clicked() {
                        r.mask = Some(m);
                    }
                }
            });
        });
        card(ui, tr("Modo de codificación"), |ui| {
            egui::ComboBox::from_id_salt("modo")
                .width(ui.available_width().min(330.0))
                .selected_text(tr(r.mode.label()))
                .show_ui(ui, |ui| {
                    for m in EncMode::ALL {
                        ui.selectable_value(&mut r.mode, m, tr(m.label()));
                    }
                });
            ui.checkbox(&mut r.eci_utf8, tr("Marcar el texto como UTF-8 (ECI 26) para tildes y emojis"))
                .on_hover_text(tr("Ayuda a algunos lectores con caracteres especiales. Casi todos los móviles ya entienden UTF-8 sin esto."));
        });
        card(ui, tr("Zona silenciosa (margen)"), |ui| {
            hint(ui, tr("La norma pide 4 módulos de margen libre alrededor. Menos puede fallar en algunos lectores."));
            slider_row(ui, tr("Margen"), |ui| ui.add(egui::Slider::new(&mut r.quiet_zone, 0..=10)));
            if r.quiet_zone < 2 {
                warn(ui, tr("Con menos de 2 módulos de margen algunos lectores no lo detectan."));
            }
        });
        if ui.button(format!("↺  {}", tr("Volver a las reglas de la norma"))).clicked() {
            *r = Rules::default();
        }
    }

    // ─────────────────────────────────────────────────── vista previa, información y guardar

    /// Dibuja la vista previa del QR dentro de un rectángulo de `size` puntos.
    fn preview(&self, ui: &mut egui::Ui, size: Vec2) {
        let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
        match (&self.qr, &self.tex) {
            (Ok(_), Some(tex)) => {
                let ts = tex.size_vec2();
                let k = ((size.x - 48.0) / ts.x).min((size.y - 48.0) / ts.y).min(1.3).max(0.05);
                let img_rect = egui::Rect::from_center_size(rect.center(), ts * k);
                let p = ui.painter();
                let pl = pal();
                // Sombra suave, "papel" blanco y tablero (visible con fondo transparente).
                for i in 1..=4 {
                    let f = i as f32;
                    p.rect_filled(img_rect.expand(10.0 + f * 2.0).translate(Vec2::new(0.0, 4.0 + f)), 18.0 + f, pl.shadow.gamma_multiply(0.35));
                }
                p.rect_filled(img_rect.expand(12.0), 16.0, Color32::WHITE);
                // El tablero solo se ve cuando el fondo es transparente.
                let transparent = self.style.bg.mode == FillMode::Transparent;
                let cell = 10.0;
                let mut y = img_rect.top();
                let mut row = 0;
                while transparent && y < img_rect.bottom() {
                    let mut x = img_rect.left();
                    let mut col = row;
                    while x < img_rect.right() {
                        if col % 2 == 0 {
                            let r = egui::Rect::from_min_max(
                                egui::pos2(x, y),
                                egui::pos2((x + cell).min(img_rect.right()), (y + cell).min(img_rect.bottom())),
                            );
                            p.rect_filled(r, 0.0, pl.checker);
                        }
                        x += cell;
                        col += 1;
                    }
                    y += cell;
                    row += 1;
                }
                p.image(tex.id(), img_rect, full_uv(), Color32::WHITE);
            }
            (Err(e), _) => {
                let p = ui.painter();
                let pl = pal();
                let r = egui::Rect::from_center_size(rect.center(), Vec2::new((size.x - 48.0).min(420.0), 120.0));
                p.rect_filled(r, 16.0, pl.card);
                p.rect_stroke(r, 16.0, Stroke::new(1.0f32, pl.card_stroke), egui::StrokeKind::Inside);
                let galley = ui.painter().layout(
                    format!("⚠  {}", e.message()),
                    FontId::proportional(15.0),
                    pl.bad,
                    r.width() - 32.0,
                );
                let pos = r.center() - galley.size() * 0.5;
                ui.painter().galley(pos, galley, pl.bad);
            }
            _ => {}
        }
    }

    /// Fichas con la versión, la corrección, la máscara, el modo, la capacidad y la lectura.
    fn info_bar(&self, ui: &mut egui::Ui) {
        let Ok(q) = &self.qr else { return };
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);
            chip(ui, &trf("Versión {0}", &[&q.version]), &trf("{0}×{0} módulos", &[&q.size]));
            chip(ui, &trf("Corrección {0}", &[&q.ecc.short()]), &trf("aguanta ≈{0} % de daño", &[&format!("{:.0}", q.ecc.recovery() * 100.0)]));
            chip(
                ui,
                &trf("Máscara {0}", &[&q.mask]),
                if self.rules.mask.is_none() { tr("elegida sola") } else { tr("fijada a mano") },
            );
            let fill = q.used_bits as f32 / q.capacity_bits as f32;
            let cap = trf("Capacidad usada {0} %", &[&format!("{:.0}", fill * 100.0)]);
            let w = text_width(ui, &cap, 12.0).max(130.0) + 26.0;
            fixed_box(ui, Vec2::new(w, 46.0), |ui| {
                egui::Frame::new().fill(pal().field).corner_radius(10).inner_margin(egui::Margin::symmetric(12, 7)).show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 4.0;
                        ui.add(egui::Label::new(RichText::new(cap).small().color(pal().hint)).extend());
                        ui.add(egui::ProgressBar::new(fill.min(1.0)).desired_width(w - 26.0).desired_height(8.0).fill(ACCENT));
                    });
                });
            });
            let (txt, col, tip): (String, Color32, String) = match &self.check {
                Check::Ok(k) => (
                    format!("✔ {}", tr("Se puede leer")),
                    pal().good,
                    match k {
                        render::ReadKind::Camera => tr("Lo encontramos y leímos en la imagen completa, como haría la cámara de un móvil."),
                        render::ReadKind::Modules => tr("Todos los módulos se distinguen bien y el contenido se recupera. Los ojos decorados los reconocen las cámaras de los móviles actuales."),
                    }
                    .to_string(),
                ),
                Check::Fail(e) => (format!("✖ {}", tr("Lectura dudosa")), pal().bad, e.message().to_string()),
                Check::Pending(_) => (tr("Comprobando lectura…").to_string(), pal().hint, String::new()),
                Check::None => (String::new(), pal().hint, String::new()),
            };
            if !txt.is_empty() {
                let w = text_width(ui, &txt, 14.5) + 32.0;
                fixed_box(ui, Vec2::new(w, 46.0), |ui| {
                    let r = egui::Frame::new()
                        .fill(col.gamma_multiply(0.14))
                        .corner_radius(20)
                        .inner_margin(egui::Margin::symmetric(14, 12))
                        .show(ui, |ui| ui.add(egui::Label::new(RichText::new(txt).color(col).strong()).extend()));
                    if !tip.is_empty() {
                        r.response.on_hover_text(tip);
                    }
                });
            }
        });
    }

    /// Tamaño de exportación y botones para guardar o copiar.
    fn export_bar(&mut self, ui: &mut egui::Ui) {
        let ok = self.qr.is_ok();
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(tr("Tamaño")).color(pal().hint));
            egui::ComboBox::from_id_salt("tam")
                .width(110.0)
                .selected_text(format!("{} px", self.export_px))
                .show_ui(ui, |ui| {
                    for px in [256u32, 512, 768, 1024, 2048, 3000, 4096] {
                        ui.selectable_value(&mut self.export_px, px, format!("{px} px"));
                    }
                });
            if ui.add_enabled(ok, primary(&format!("⬇ {}", tr("Guardar PNG")))).clicked() {
                self.save_png();
            }
            if ui
                .add_enabled(ok, egui::Button::new(format!("⬇ {}", tr("Guardar SVG"))))
                .on_hover_text(tr("Ideal para imprenta: se puede agrandar sin perder calidad"))
                .clicked()
            {
                self.save_svg();
            }
            if platform::CAN_COPY_IMAGE && ui.add_enabled(ok, egui::Button::new(format!("📋 {}", tr("Copiar")))).clicked() {
                self.copy_image();
            }
        });
        if platform::IS_MOBILE {
            hint(ui, &trf("Se guarda en: {0}", &[&platform::sub_dir("QR").display()]));
        }
    }

    /// Mensaje breve flotante.
    fn show_toast(&mut self, ctx: &egui::Context) {
        let Some(t) = &self.toast else { return };
        let age = t.since.elapsed();
        if age > Duration::from_secs(6) {
            self.toast = None;
            return;
        }
        ctx.request_repaint_after(Duration::from_millis(200));
        let col = if t.ok { pal().good } else { pal().bad };
        egui::Area::new(egui::Id::new("aviso"))
            .anchor(egui::Align2::RIGHT_BOTTOM, Vec2::new(-20.0, -20.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(pal().card)
                    .stroke(Stroke::new(1.0f32, col.gamma_multiply(0.6)))
                    .corner_radius(12)
                    .shadow(egui::Shadow { offset: [0, 6], blur: 18, spread: 0, color: pal().shadow })
                    .inner_margin(egui::Margin::symmetric(16, 12))
                    .show(ui, |ui| {
                        ui.set_max_width(420.0);
                        ui.label(RichText::new(format!("{}  {}", if t.ok { "✔" } else { "⚠" }, t.text)).color(col));
                    });
            });
    }

    /// En el móvil: ventana con los archivos de la carpeta de la app para elegir uno.
    fn file_list_window(&mut self, ctx: &egui::Context) {
        let Some(kind) = self.file_list else { return };
        let files = platform::list_files(kind);
        let mut open = true;
        let mut chosen: Option<PathBuf> = None;
        let title = if kind == FileKind::Image { tr("Elegir logo") } else { tr("Cargar diseño") };
        egui::Window::new(title)
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.set_max_width(360.0);
                if files.is_empty() {
                    hint(ui, &trf("No hay archivos todavía. Cópialos a: {0}", &[&platform::sub_dir(kind.mobile_folder()).display()]));
                }
                egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
                    for f in &files {
                        let name = f.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                        if ui.add(egui::Button::new(name).min_size(Vec2::new(ui.available_width(), 34.0))).clicked() {
                            chosen = Some(f.clone());
                        }
                    }
                });
            });
        if let Some(f) = chosen {
            self.file_list = None;
            if kind == FileKind::Image {
                self.open_logo(&f);
            } else {
                self.load_design(&f);
            }
        } else if !open {
            self.file_list = None;
        }
    }

    /// Con `--screenshot`: tras ~120 cuadros guarda una foto de la ventana y la cierra.
    fn handle_screenshot(&mut self, ctx: &egui::Context) {
        let Some(shot) = &mut self.shot else { return };
        shot.frames += 1;
        ctx.request_repaint();
        if shot.frames >= 120 && !shot.requested {
            shot.requested = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
        }
        let mut got = None;
        ctx.input(|i| {
            for e in &i.events {
                if let egui::Event::Screenshot { image, .. } = e {
                    got = Some(image.clone());
                }
            }
        });
        if let Some(img) = got {
            let [w, h] = img.size;
            let mut buf = RgbaImage::new(w as u32, h as u32);
            for (i, c) in img.pixels.iter().enumerate() {
                let [r, g, b, _] = c.to_srgba_unmultiplied();
                buf.as_mut()[i * 4..i * 4 + 4].copy_from_slice(&[r, g, b, 255]);
            }
            if let Some(dir) = shot.path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            if let Err(e) = buf.save_with_format(&shot.path, image::ImageFormat::Png) {
                eprintln!("No se pudo guardar la captura: {e}");
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

impl eframe::App for QrApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.refresh(ctx);
        self.run_check(ctx);
        let p = pal();
        let compact = ctx.screen_rect().width() < COMPACT_WIDTH;

        egui::TopBottomPanel::top("arriba")
            .show_separator_line(false)
            .frame(egui::Frame::new().fill(p.panel).inner_margin(egui::Margin::symmetric(16, 10)).stroke(Stroke::new(1.0f32, p.card_stroke)))
            .show(ctx, |ui| self.top_bar(ui, compact));

        if compact {
            // Una sola columna: vista previa, información, guardar, pestañas y opciones.
            egui::CentralPanel::default()
                .frame(egui::Frame::new().fill(p.canvas).inner_margin(egui::Margin::symmetric(12, 8)))
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                        let w = ui.available_width();
                        let h = (w * 1.05).min(520.0);
                        self.preview(ui, Vec2::new(w, h));
                        card_frame().show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            self.info_bar(ui);
                            self.export_bar(ui);
                        });
                        ui.add_space(8.0);
                        self.tab_strip(ui);
                        self.options(ui);
                    });
                });
        } else {
            egui::SidePanel::left("menu")
                .show_separator_line(false)
                .resizable(false)
                .exact_width(88.0)
                .frame(egui::Frame::new().fill(p.panel).inner_margin(egui::Margin::symmetric(0, 8)))
                .show(ctx, |ui| self.nav_rail(ui));
            egui::SidePanel::left("opciones")
                .show_separator_line(false)
                .resizable(true)
                .default_width(400.0)
                .width_range(340.0..=560.0)
                .frame(egui::Frame::new().fill(p.canvas).inner_margin(egui::Margin { left: 16, right: 12, top: 12, bottom: 0 }))
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                        ui.set_width(ui.available_width() - 6.0);
                        self.options(ui);
                    });
                });
            egui::CentralPanel::default()
                .frame(egui::Frame::new().fill(p.canvas).inner_margin(egui::Margin { left: 8, right: 20, top: 12, bottom: 16 }))
                .show(ctx, |ui| {
                    let avail = ui.available_size();
                    let bottom = 128.0;
                    self.preview(ui, Vec2::new(avail.x, (avail.y - bottom).max(220.0)));
                    card_frame().show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        self.info_bar(ui);
                        ui.add_space(2.0);
                        self.export_bar(ui);
                    });
                });
        }

        self.file_list_window(ctx);
        self.show_toast(ctx);
        self.handle_screenshot(ctx);
    }
}

// ───────────────────────────────────────────────────────────── piezas de interfaz

/// Colores rápidos para el código (oscuros, para que se lea bien).
const FG_SWATCHES: [Rgba; 12] = [
    [0, 0, 0, 255],
    [16, 42, 92, 255],
    [0, 92, 151, 255],
    [0, 118, 118, 255],
    [20, 110, 60, 255],
    [88, 28, 135, 255],
    [99, 91, 255, 255],
    [190, 30, 110, 255],
    [200, 40, 50, 255],
    [200, 85, 20, 255],
    [78, 52, 36, 255],
    [44, 52, 64, 255],
];

/// Colores rápidos para el fondo (claros).
const BG_SWATCHES: [Rgba; 10] = [
    [255, 255, 255, 255],
    [250, 247, 240, 255],
    [243, 244, 248, 255],
    [235, 245, 255, 255],
    [236, 250, 244, 255],
    [246, 242, 255, 255],
    [255, 240, 247, 255],
    [255, 250, 230, 255],
    [255, 244, 236, 255],
    [240, 242, 245, 255],
];

/// Textos sugeridos para la etiqueta del marco (se traducen con `tr`).
const FRAME_TEXTS: [&str; 7] = ["ESCANÉAME", "¡ESCANÉAME!", "MENÚ", "WIFI GRATIS", "SÍGUENOS", "MÁS INFO", "PAGA AQUÍ"];

/// Coordenadas de textura completas (0,0)–(1,1).
fn full_uv() -> egui::Rect {
    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0))
}

/// Pequeño logotipo del programa, dibujado con figuras.
fn app_logo(ui: &mut egui::Ui, size: f32) {
    let (r, _) = ui.allocate_exact_size(Vec2::splat(size), egui::Sense::hover());
    let p = ui.painter();
    p.rect_filled(r, size * 0.26, ACCENT);
    let u = r.width() / 10.0;
    for (cx, cy) in [(1.5, 1.5), (5.5, 1.5), (1.5, 5.5)] {
        let eye = egui::Rect::from_min_size(r.min + Vec2::new(cx * u, cy * u), Vec2::splat(3.0 * u));
        p.rect_stroke(eye, u, Stroke::new(u * 0.75, Color32::WHITE), egui::StrokeKind::Inside);
    }
    for (cx, cy) in [(6.0, 6.0), (7.5, 7.5), (6.0, 7.5), (7.5, 5.5)] {
        p.circle_filled(r.min + Vec2::new((cx + 0.5) * u, (cy + 0.5) * u), u * 0.55, Color32::WHITE);
    }
}

/// Botón cuadrado con solo un icono.
fn icon_button(ui: &mut egui::Ui, icon: &str) -> egui::Response {
    ui.add(egui::Button::new(RichText::new(icon).size(16.0)).min_size(Vec2::splat(34.0)).corner_radius(10))
}

/// Botón principal (relleno con el color de acento).
fn primary(text: &str) -> egui::Button<'static> {
    egui::Button::new(RichText::new(text.to_string()).color(Color32::WHITE).strong()).fill(ACCENT).corner_radius(10)
}

/// Botón "pastilla" que se puede elegir.
fn pill(ui: &mut egui::Ui, text: &str, selected: bool) -> egui::Response {
    let p = pal();
    let txt = if selected { RichText::new(text).color(Color32::WHITE) } else { RichText::new(text).color(p.text) };
    ui.add(
        egui::Button::new(txt)
            .fill(if selected { ACCENT } else { p.field })
            .stroke(Stroke::NONE)
            .corner_radius(18),
    )
}

/// Marco de una tarjeta.
fn card_frame() -> egui::Frame {
    let p = pal();
    egui::Frame::new()
        .fill(p.card)
        .stroke(Stroke::new(1.0f32, p.card_stroke))
        .corner_radius(14)
        .inner_margin(egui::Margin::same(14))
}

/// Tarjeta con título que agrupa unas opciones.
fn card(ui: &mut egui::Ui, title: &str, add: impl FnOnce(&mut egui::Ui)) {
    card_frame().show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(RichText::new(title).family(bold()).size(15.0));
        ui.add_space(2.0);
        add(ui);
    });
    ui.add_space(10.0);
}

/// Una fila con un nombre y un control (normalmente un deslizador).
fn slider_row(ui: &mut egui::Ui, label: &str, add: impl FnOnce(&mut egui::Ui) -> egui::Response) {
    ui.horizontal(|ui| {
        ui.add_sized([120.0, 20.0], egui::Label::new(RichText::new(label).color(pal().hint)).truncate());
        add(ui);
    });
}

fn hint(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).small().color(pal().hint));
}

fn warn(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(format!("⚠ {text}")).color(pal().warn));
}

/// Ficha pequeña con un dato y una explicación debajo.
fn chip(ui: &mut egui::Ui, title: &str, sub: &str) {
    // Se mide antes para reservar su sitio: así, en pantallas estrechas, la ficha entera
    // pasa a la línea siguiente en vez de partirse letra a letra.
    let w = text_width(ui, title, 14.5).max(text_width(ui, sub, 12.0)) + 28.0;
    fixed_box(ui, Vec2::new(w, 46.0), |ui| {
        egui::Frame::new().fill(pal().field).corner_radius(10).inner_margin(egui::Margin::symmetric(12, 6)).show(ui, |ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 1.0;
                ui.add(egui::Label::new(RichText::new(title).strong()).extend());
                ui.add(egui::Label::new(RichText::new(sub).small().color(pal().hint)).extend());
            });
        });
    });
}

/// Ancho que ocupa un texto de una línea.
fn text_width(ui: &egui::Ui, text: &str, size: f32) -> f32 {
    ui.fonts(|f| f.layout_no_wrap(text.to_string(), FontId::proportional(size), Color32::WHITE).size().x)
}

/// Reserva un hueco de tamaño fijo (respeta el salto de línea de las filas que se ajustan).
fn fixed_box(ui: &mut egui::Ui, size: Vec2, add: impl FnOnce(&mut egui::Ui)) {
    ui.allocate_ui_with_layout(size, egui::Layout::top_down(egui::Align::Min), |ui| {
        ui.set_min_size(size);
        add(ui);
    });
}

/// Rejilla de fichas con icono para elegir una opción.
fn tile_picker<T: Copy + PartialEq>(ui: &mut egui::Ui, items: &[(T, egui::TextureId, &str)], current: &mut T) {
    let p = pal();
    let tile = Vec2::new(80.0, 84.0);
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);
        for (v, tex, label) in items {
            let sel = *current == *v;
            let (rect, resp) = ui.allocate_exact_size(tile, egui::Sense::click());
            let hov = resp.hovered();
            let painter = ui.painter();
            painter.rect_filled(rect, 12.0, if sel { p.sel_fill } else if hov { p.hover } else { p.field });
            if sel {
                painter.rect_stroke(rect, 12.0, Stroke::new(2.0f32, ACCENT), egui::StrokeKind::Inside);
            }
            let icon = egui::Rect::from_center_size(egui::pos2(rect.center().x, rect.top() + 32.0), Vec2::splat(40.0));
            painter.image(*tex, icon, full_uv(), if sel { ACCENT } else { p.icon });
            let galley = painter.layout(label.to_string(), FontId::proportional(11.5), if sel { ACCENT } else { p.hint }, tile.x - 6.0);
            let pos = egui::pos2(rect.center().x - galley.size().x * 0.5, rect.bottom() - 8.0 - galley.size().y);
            painter.galley(pos, galley, p.hint);
            if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                *current = *v;
            }
        }
    });
}

/// Botón de color que abre el selector.
fn color_btn(ui: &mut egui::Ui, c: &mut Rgba) {
    let mut col = Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3]);
    if egui::color_picker::color_edit_button_srgba(ui, &mut col, egui::color_picker::Alpha::OnlyBlend).changed() {
        *c = col.to_srgba_unmultiplied();
    }
}

/// Fila de colores rápidos: un clic pone ese color.
fn swatches(ui: &mut egui::Ui, list: &[Rgba], target: &mut Rgba) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);
        for c in list {
            let (rect, resp) = ui.allocate_exact_size(Vec2::splat(24.0), egui::Sense::click());
            let col = Color32::from_rgb(c[0], c[1], c[2]);
            let sel = target[..3] == c[..3];
            ui.painter().circle_filled(rect.center(), 11.0, col);
            ui.painter().circle_stroke(rect.center(), 11.0, Stroke::new(1.0f32, pal().card_stroke));
            if sel {
                ui.painter().circle_stroke(rect.center(), 13.5, Stroke::new(2.0f32, ACCENT));
            }
            if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                *target = *c;
            }
        }
    });
}

/// Editor de un relleno: sólido, degradado lineal o radial (y transparente para el fondo).
fn fill_editor(ui: &mut egui::Ui, id: &str, fill: &mut Fill, allow_transparent: bool, quick: &[Rgba]) {
    let modes: &[FillMode] = if allow_transparent {
        &[FillMode::Solid, FillMode::Linear, FillMode::Radial, FillMode::Transparent]
    } else {
        &[FillMode::Solid, FillMode::Linear, FillMode::Radial]
    };
    ui.push_id(id, |ui| {
        ui.horizontal_wrapped(|ui| {
            for m in modes {
                if pill(ui, tr(m.label()), fill.mode == *m).clicked() {
                    fill.mode = *m;
                }
            }
        });
        ui.add_space(4.0);
        match fill.mode {
            FillMode::Solid => {
                ui.horizontal(|ui| {
                    ui.label(tr("Color"));
                    color_btn(ui, &mut fill.c1);
                });
                swatches(ui, quick, &mut fill.c1);
            }
            FillMode::Linear | FillMode::Radial => {
                ui.horizontal_wrapped(|ui| {
                    ui.label(if fill.mode == FillMode::Radial { tr("Centro") } else { tr("Inicio") });
                    color_btn(ui, &mut fill.c1);
                    ui.add_space(12.0);
                    ui.label(if fill.mode == FillMode::Radial { tr("Borde") } else { tr("Final") });
                    color_btn(ui, &mut fill.c2);
                    if ui.small_button("⇄").on_hover_text(tr("Intercambiar")).clicked() {
                        std::mem::swap(&mut fill.c1, &mut fill.c2);
                    }
                });
                swatches(ui, quick, &mut fill.c1);
                if fill.mode == FillMode::Linear {
                    slider_row(ui, tr("Ángulo"), |ui| ui.add(egui::Slider::new(&mut fill.angle, 0.0..=360.0).suffix("°")));
                }
            }
            FillMode::Transparent => {
                hint(ui, tr("El PNG y el SVG quedarán sin fondo. Ponlo siempre sobre algo claro."));
            }
        }
    });
}

/// Avisos de contraste entre el código y el fondo.
fn contrast_notes(ui: &mut egui::Ui, s: &Style) {
    let fg = s.fg.average();
    let bg = if s.bg.mode == FillMode::Transparent { [255, 255, 255, 255] } else { s.bg.average() };
    let c = contrast(fg, bg);
    ui.label(trf("Contraste entre código y fondo: {0} : 1", &[&format!("{c:.1}")]));
    if c < 3.0 {
        warn(ui, tr("Contraste bajo: muchos móviles no podrán leerlo. Oscurece el código o aclara el fondo."));
    } else if c < 4.5 {
        hint(ui, tr("Contraste aceptable. Por encima de 4.5 : 1 es más seguro."));
    }
    if luminance(fg) > luminance(bg) {
        warn(ui, tr("Colores invertidos (código claro sobre fondo oscuro): algunos lectores antiguos no lo leen."));
    }
}
