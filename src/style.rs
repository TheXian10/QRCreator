//! Opciones de diseño y reglas del código QR.
//!
//! - [`Rules`]: las reglas técnicas de la norma (corrección, versión, máscara...).
//! - [`Style`]: todo lo visual (formas, colores, logo, marco).
//! - [`PRESETS`]: los diseños listos de la galería.
//! - [`DesignFile`]: lo que se guarda al pulsar «Guardar diseño» (JSON).
//!
//! Los nombres que devuelven las funciones `label()` están en español; la interfaz los
//! traduce con `i18n::tr`.

use serde::{Deserialize, Serialize};

/// Un color con transparencia: rojo, verde, azul y opacidad (0-255).
pub type Rgba = [u8; 4];

/// Nivel de corrección de errores: cuánto del código se puede perder (dañado o tapado por
/// un logo) y aun así leerse.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Ecc {
    L,
    M,
    Q,
    H,
}

impl Ecc {
    pub const ALL: [Ecc; 4] = [Ecc::L, Ecc::M, Ecc::Q, Ecc::H];
    pub fn label(self) -> &'static str {
        match self {
            Ecc::L => "L · Baja (≈7 %)",
            Ecc::M => "M · Media (≈15 %)",
            Ecc::Q => "Q · Alta (≈25 %)",
            Ecc::H => "H · Máxima (≈30 %)",
        }
    }
    pub fn short(self) -> &'static str {
        match self {
            Ecc::L => "L",
            Ecc::M => "M",
            Ecc::Q => "Q",
            Ecc::H => "H",
        }
    }
    /// Fracción aproximada de módulos que se pueden perder y seguir leyéndose.
    pub fn recovery(self) -> f32 {
        match self {
            Ecc::L => 0.07,
            Ecc::M => 0.15,
            Ecc::Q => 0.25,
            Ecc::H => 0.30,
        }
    }
}

/// Cómo se guardan los caracteres dentro del QR. Los modos numérico y alfanumérico ocupan
/// menos, pero solo admiten algunos caracteres.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum EncMode {
    Auto,
    Numeric,
    Alphanumeric,
    Byte,
}

impl EncMode {
    pub const ALL: [EncMode; 4] = [EncMode::Auto, EncMode::Numeric, EncMode::Alphanumeric, EncMode::Byte];
    pub fn label(self) -> &'static str {
        match self {
            EncMode::Auto => "Automático (el más compacto)",
            EncMode::Numeric => "Numérico (solo 0-9)",
            EncMode::Alphanumeric => "Alfanumérico (0-9 A-Z espacio $%*+-./:)",
            EncMode::Byte => "Bytes (UTF-8, cualquier texto)",
        }
    }
}

/// Las reglas técnicas con las que se crea el código.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Rules {
    /// Nivel de corrección de errores.
    pub ecc: Ecc,
    /// Subir la corrección si cabe sin agrandar el código.
    pub boost_ecc: bool,
    /// Versión (tamaño) mínima y máxima permitidas, de 1 a 40.
    pub min_version: u8,
    pub max_version: u8,
    /// None = elegir la mejor máscara automáticamente.
    pub mask: Option<u8>,
    /// Modo de codificación.
    pub mode: EncMode,
    /// Añadir la marca ECI 26 (texto en UTF-8).
    pub eci_utf8: bool,
    /// Margen libre alrededor del código, en módulos (la norma pide 4).
    pub quiet_zone: u32,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            ecc: Ecc::M,
            boost_ecc: true,
            min_version: 1,
            max_version: 40,
            mask: None,
            mode: EncMode::Auto,
            eci_utf8: false,
            quiet_zone: 4,
        }
    }
}

/// Forma de cada "punto" (módulo) del código.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ModuleShape {
    Square,
    Rounded,
    Circle,
    SmallDots,
    Liquid,
    Diamond,
    VBars,
    HBars,
    Star,
    Plus,
    Heart,
    Hexagon,
    Petal,
    Cross,
    Mosaic,
}

impl ModuleShape {
    pub const ALL: [ModuleShape; 15] = [
        ModuleShape::Square,
        ModuleShape::Rounded,
        ModuleShape::Circle,
        ModuleShape::SmallDots,
        ModuleShape::Liquid,
        ModuleShape::Diamond,
        ModuleShape::VBars,
        ModuleShape::HBars,
        ModuleShape::Star,
        ModuleShape::Plus,
        ModuleShape::Heart,
        ModuleShape::Hexagon,
        ModuleShape::Petal,
        ModuleShape::Cross,
        ModuleShape::Mosaic,
    ];
    pub fn label(self) -> &'static str {
        match self {
            ModuleShape::Square => "Cuadrado",
            ModuleShape::Rounded => "Redondeado",
            ModuleShape::Circle => "Círculos",
            ModuleShape::SmallDots => "Puntitos",
            ModuleShape::Liquid => "Líquido",
            ModuleShape::Diamond => "Rombos",
            ModuleShape::VBars => "Barras vert.",
            ModuleShape::HBars => "Barras horiz.",
            ModuleShape::Star => "Estrellas",
            ModuleShape::Plus => "Cruces",
            ModuleShape::Heart => "Corazones",
            ModuleShape::Hexagon => "Hexágonos",
            ModuleShape::Petal => "Pétalos",
            ModuleShape::Cross => "Equis",
            ModuleShape::Mosaic => "Mosaico",
        }
    }
}

/// Forma del marco exterior de los tres "ojos" (los cuadros de las esquinas).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum EyeFrame {
    Square,
    Rounded,
    ExtraRounded,
    Circle,
    Leaf,
    Drop,
    Octagon,
    Dotted,
}

impl EyeFrame {
    pub const ALL: [EyeFrame; 8] = [
        EyeFrame::Square,
        EyeFrame::Rounded,
        EyeFrame::ExtraRounded,
        EyeFrame::Circle,
        EyeFrame::Leaf,
        EyeFrame::Drop,
        EyeFrame::Octagon,
        EyeFrame::Dotted,
    ];
    pub fn label(self) -> &'static str {
        match self {
            EyeFrame::Square => "Cuadrado",
            EyeFrame::Rounded => "Redondeado",
            EyeFrame::ExtraRounded => "Muy redondeado",
            EyeFrame::Circle => "Círculo",
            EyeFrame::Leaf => "Hoja",
            EyeFrame::Drop => "Gota",
            EyeFrame::Octagon => "Octágono",
            EyeFrame::Dotted => "Punteado",
        }
    }
}

/// Forma del centro de los ojos.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum EyeBall {
    Square,
    Rounded,
    Circle,
    Leaf,
    Drop,
    Diamond,
    Star,
    Heart,
    Hexagon,
    Octagon,
}

impl EyeBall {
    pub const ALL: [EyeBall; 10] = [
        EyeBall::Square,
        EyeBall::Rounded,
        EyeBall::Circle,
        EyeBall::Leaf,
        EyeBall::Drop,
        EyeBall::Diamond,
        EyeBall::Star,
        EyeBall::Heart,
        EyeBall::Hexagon,
        EyeBall::Octagon,
    ];
    pub fn label(self) -> &'static str {
        match self {
            EyeBall::Square => "Cuadrado",
            EyeBall::Rounded => "Redondeado",
            EyeBall::Circle => "Círculo",
            EyeBall::Leaf => "Hoja",
            EyeBall::Drop => "Gota",
            EyeBall::Diamond => "Rombo",
            EyeBall::Star => "Estrella",
            EyeBall::Heart => "Corazón",
            EyeBall::Hexagon => "Hexágono",
            EyeBall::Octagon => "Octágono",
        }
    }
}

/// Tipo de relleno de un color.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum FillMode {
    Solid,
    Linear,
    Radial,
    Transparent,
}

impl FillMode {
    pub fn label(self) -> &'static str {
        match self {
            FillMode::Solid => "Color sólido",
            FillMode::Linear => "Degradado lineal",
            FillMode::Radial => "Degradado radial",
            FillMode::Transparent => "Transparente",
        }
    }
}

/// Un relleno: color sólido, degradado entre dos colores o transparente.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Fill {
    pub mode: FillMode,
    pub c1: Rgba,
    pub c2: Rgba,
    /// Ángulo del degradado lineal en grados (0 = de izquierda a derecha).
    pub angle: f32,
}

impl Default for Fill {
    fn default() -> Self {
        Self { mode: FillMode::Solid, c1: [0, 0, 0, 255], c2: [0, 0, 0, 255], angle: 45.0 }
    }
}

impl Fill {
    pub fn solid(c: Rgba) -> Self {
        Self { mode: FillMode::Solid, c1: c, c2: c, angle: 45.0 }
    }
    pub fn linear(c1: Rgba, c2: Rgba, angle: f32) -> Self {
        Self { mode: FillMode::Linear, c1, c2, angle }
    }
    pub fn radial(c1: Rgba, c2: Rgba) -> Self {
        Self { mode: FillMode::Radial, c1, c2, angle: 0.0 }
    }
    /// Color "medio" para comprobar el contraste.
    pub fn average(&self) -> Rgba {
        match self.mode {
            FillMode::Solid => self.c1,
            FillMode::Transparent => [255, 255, 255, 0],
            _ => [
                ((self.c1[0] as u16 + self.c2[0] as u16) / 2) as u8,
                ((self.c1[1] as u16 + self.c2[1] as u16) / 2) as u8,
                ((self.c1[2] as u16 + self.c2[2] as u16) / 2) as u8,
                ((self.c1[3] as u16 + self.c2[3] as u16) / 2) as u8,
            ],
        }
    }
}

/// Marco alrededor del código, con o sin texto.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum FrameStyle {
    None,
    LabelBottom,
    LabelTop,
    Border,
}

impl FrameStyle {
    pub const ALL: [FrameStyle; 4] =
        [FrameStyle::None, FrameStyle::LabelBottom, FrameStyle::LabelTop, FrameStyle::Border];
    pub fn label(self) -> &'static str {
        match self {
            FrameStyle::None => "Sin marco",
            FrameStyle::LabelBottom => "Etiqueta abajo",
            FrameStyle::LabelTop => "Etiqueta arriba",
            FrameStyle::Border => "Borde con texto",
        }
    }
}

/// Forma del fondo que se pone detrás del logo.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum LogoBg {
    None,
    Square,
    Rounded,
    Circle,
}

impl LogoBg {
    pub const ALL: [LogoBg; 4] = [LogoBg::None, LogoBg::Square, LogoBg::Rounded, LogoBg::Circle];
    pub fn label(self) -> &'static str {
        match self {
            LogoBg::None => "Sin fondo",
            LogoBg::Square => "Cuadrado",
            LogoBg::Rounded => "Redondeado",
            LogoBg::Circle => "Círculo",
        }
    }
}

/// Todas las opciones visuales del código. `#[serde(default)]` permite abrir diseños
/// guardados con versiones anteriores (lo que falte toma su valor por defecto).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Style {
    pub module_shape: ModuleShape,
    /// Tamaño de cada módulo dentro de su casilla (0.5 – 1.0).
    pub module_scale: f32,
    pub eye_frame: EyeFrame,
    pub eye_ball: EyeBall,
    pub fg: Fill,
    pub eyes_custom: bool,
    pub eye_frame_color: Rgba,
    pub eye_ball_color: Rgba,
    pub bg: Fill,
    /// Radio de las esquinas del fondo, en módulos.
    pub bg_radius: f32,
    pub frame: FrameStyle,
    pub frame_color: Rgba,
    pub frame_text: String,
    pub frame_text_color: Rgba,
    /// Tamaño del texto en relación con el ancho del QR (0.04 – 0.16).
    pub frame_text_size: f32,
    /// Grosor del borde, en módulos.
    pub frame_width: f32,
    /// Tamaño del logo en relación con el ancho del QR (0.10 – 0.35).
    pub logo_size: f32,
    pub logo_clear: bool,
    pub logo_padding: f32,
    pub logo_bg: LogoBg,
    pub logo_bg_color: Rgba,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            module_shape: ModuleShape::Square,
            module_scale: 1.0,
            eye_frame: EyeFrame::Square,
            eye_ball: EyeBall::Square,
            fg: Fill::solid([0, 0, 0, 255]),
            eyes_custom: false,
            eye_frame_color: [0, 0, 0, 255],
            eye_ball_color: [0, 0, 0, 255],
            bg: Fill::solid([255, 255, 255, 255]),
            bg_radius: 0.0,
            frame: FrameStyle::None,
            frame_color: [0, 0, 0, 255],
            frame_text: "ESCANÉAME".into(),
            frame_text_color: [255, 255, 255, 255],
            frame_text_size: 0.09,
            frame_width: 2.0,
            logo_size: 0.22,
            logo_clear: true,
            logo_padding: 0.6,
            logo_bg: LogoBg::Rounded,
            logo_bg_color: [255, 255, 255, 255],
        }
    }
}

/// Un diseño de la galería: su nombre (en español) y la función que lo crea.
pub struct Preset {
    pub name: &'static str,
    pub style: fn() -> Style,
}

/// Los diseños de la galería, en el orden en que se enseñan.
pub const PRESETS: &[Preset] = &[
    Preset { name: "Clásico", style: Style::default },
    Preset { name: "Océano", style: preset_ocean },
    Preset { name: "Atardecer", style: preset_sunset },
    Preset { name: "Bosque", style: preset_forest },
    Preset { name: "Uva", style: preset_grape },
    Preset { name: "Elegante", style: preset_elegant },
    Preset { name: "Noche", style: preset_night },
    Preset { name: "Caramelo", style: preset_candy },
    Preset { name: "Menta", style: preset_mint },
    Preset { name: "Coral", style: preset_coral },
    Preset { name: "Cielo", style: preset_sky },
    Preset { name: "Dorado", style: preset_gold },
    Preset { name: "Lava", style: preset_lava },
    Preset { name: "Café", style: preset_coffee },
    Preset { name: "Fiesta", style: preset_party },
    Preset { name: "Pizarra", style: preset_slate },
    Preset { name: "Empresa", style: preset_corporate },
    Preset { name: "Lavanda", style: preset_lavender },
];

fn preset_ocean() -> Style {
    Style {
        module_shape: ModuleShape::Liquid,
        eye_frame: EyeFrame::ExtraRounded,
        eye_ball: EyeBall::Rounded,
        fg: Fill::linear([0, 92, 151, 255], [0, 180, 200, 255], 45.0),
        eyes_custom: true,
        eye_frame_color: [0, 70, 130, 255],
        eye_ball_color: [0, 150, 190, 255],
        bg: Fill::solid([255, 255, 255, 255]),
        bg_radius: 3.0,
        frame: FrameStyle::LabelBottom,
        frame_color: [0, 92, 151, 255],
        ..Style::default()
    }
}

fn preset_sunset() -> Style {
    Style {
        module_shape: ModuleShape::Circle,
        module_scale: 0.9,
        eye_frame: EyeFrame::Leaf,
        eye_ball: EyeBall::Leaf,
        fg: Fill::linear([214, 40, 90, 255], [245, 130, 32, 255], 90.0),
        bg: Fill::solid([255, 248, 240, 255]),
        bg_radius: 4.0,
        frame: FrameStyle::Border,
        frame_color: [214, 40, 90, 255],
        frame_text: "¡ESCANÉAME!".into(),
        ..Style::default()
    }
}

fn preset_forest() -> Style {
    Style {
        module_shape: ModuleShape::Rounded,
        module_scale: 0.92,
        eye_frame: EyeFrame::Rounded,
        eye_ball: EyeBall::Circle,
        fg: Fill::radial([20, 90, 50, 255], [60, 140, 70, 255]),
        eyes_custom: true,
        eye_frame_color: [20, 70, 40, 255],
        eye_ball_color: [120, 170, 60, 255],
        bg: Fill::solid([244, 250, 240, 255]),
        bg_radius: 2.0,
        ..Style::default()
    }
}

fn preset_grape() -> Style {
    Style {
        module_shape: ModuleShape::Diamond,
        eye_frame: EyeFrame::Circle,
        eye_ball: EyeBall::Circle,
        fg: Fill::linear([88, 28, 135, 255], [190, 60, 170, 255], 135.0),
        bg: Fill::solid([255, 255, 255, 255]),
        bg_radius: 3.0,
        frame: FrameStyle::LabelTop,
        frame_color: [88, 28, 135, 255],
        frame_text: "MI MENÚ".into(),
        ..Style::default()
    }
}

fn preset_elegant() -> Style {
    Style {
        module_shape: ModuleShape::VBars,
        module_scale: 0.8,
        eye_frame: EyeFrame::Drop,
        eye_ball: EyeBall::Drop,
        fg: Fill::solid([30, 30, 34, 255]),
        eyes_custom: true,
        eye_frame_color: [30, 30, 34, 255],
        eye_ball_color: [176, 141, 87, 255],
        bg: Fill::solid([250, 247, 240, 255]),
        bg_radius: 1.5,
        frame: FrameStyle::LabelBottom,
        frame_color: [30, 30, 34, 255],
        frame_text_color: [222, 190, 130, 255],
        frame_text: "BIENVENIDOS".into(),
        ..Style::default()
    }
}

fn preset_night() -> Style {
    Style {
        module_shape: ModuleShape::SmallDots,
        eye_frame: EyeFrame::Rounded,
        eye_ball: EyeBall::Rounded,
        fg: Fill::linear([20, 24, 48, 255], [60, 40, 110, 255], 45.0),
        bg: Fill::linear([235, 240, 255, 255], [255, 255, 255, 255], 90.0),
        bg_radius: 3.0,
        ..Style::default()
    }
}

fn preset_candy() -> Style {
    Style {
        module_shape: ModuleShape::Heart,
        module_scale: 1.0,
        eye_frame: EyeFrame::ExtraRounded,
        eye_ball: EyeBall::Star,
        fg: Fill::linear([220, 30, 110, 255], [140, 40, 200, 255], 60.0),
        bg: Fill::solid([255, 240, 247, 255]),
        bg_radius: 4.0,
        frame: FrameStyle::LabelBottom,
        frame_color: [220, 30, 110, 255],
        frame_text: "CON CARIÑO".into(),
        ..Style::default()
    }
}

fn preset_mint() -> Style {
    Style {
        module_shape: ModuleShape::Petal,
        module_scale: 0.95,
        eye_frame: EyeFrame::Leaf,
        eye_ball: EyeBall::Leaf,
        fg: Fill::linear([0, 110, 90, 255], [30, 160, 120, 255], 45.0),
        bg: Fill::solid([236, 250, 244, 255]),
        bg_radius: 3.0,
        ..Style::default()
    }
}

fn preset_coral() -> Style {
    Style {
        module_shape: ModuleShape::Hexagon,
        eye_frame: EyeFrame::Octagon,
        eye_ball: EyeBall::Hexagon,
        fg: Fill::linear([200, 50, 60, 255], [235, 110, 70, 255], 30.0),
        bg: Fill::solid([255, 246, 242, 255]),
        bg_radius: 2.5,
        frame: FrameStyle::LabelBottom,
        frame_color: [200, 50, 60, 255],
        frame_text: "MÁS INFO".into(),
        ..Style::default()
    }
}

fn preset_sky() -> Style {
    Style {
        module_shape: ModuleShape::Circle,
        module_scale: 0.85,
        eye_frame: EyeFrame::Circle,
        eye_ball: EyeBall::Heart,
        fg: Fill::radial([20, 80, 170, 255], [40, 140, 210, 255]),
        eyes_custom: true,
        eye_frame_color: [20, 70, 160, 255],
        eye_ball_color: [230, 60, 110, 255],
        bg: Fill::linear([235, 245, 255, 255], [255, 255, 255, 255], 90.0),
        bg_radius: 4.0,
        ..Style::default()
    }
}

fn preset_gold() -> Style {
    Style {
        module_shape: ModuleShape::Mosaic,
        eye_frame: EyeFrame::Square,
        eye_ball: EyeBall::Diamond,
        fg: Fill::linear([110, 80, 20, 255], [150, 110, 30, 255], 135.0),
        bg: Fill::solid([252, 247, 232, 255]),
        bg_radius: 1.0,
        frame: FrameStyle::Border,
        frame_color: [40, 32, 20, 255],
        frame_text_color: [225, 190, 110, 255],
        frame_text: "PREMIUM".into(),
        frame_width: 1.5,
        ..Style::default()
    }
}

fn preset_lava() -> Style {
    Style {
        module_shape: ModuleShape::Liquid,
        eye_frame: EyeFrame::Drop,
        eye_ball: EyeBall::Drop,
        fg: Fill::linear([150, 20, 20, 255], [220, 80, 20, 255], 90.0),
        bg: Fill::solid([255, 247, 238, 255]),
        bg_radius: 3.0,
        ..Style::default()
    }
}

fn preset_coffee() -> Style {
    Style {
        module_shape: ModuleShape::HBars,
        module_scale: 0.82,
        eye_frame: EyeFrame::Rounded,
        eye_ball: EyeBall::Rounded,
        fg: Fill::solid([78, 52, 36, 255]),
        eyes_custom: true,
        eye_frame_color: [78, 52, 36, 255],
        eye_ball_color: [160, 110, 70, 255],
        bg: Fill::solid([246, 238, 226, 255]),
        bg_radius: 2.0,
        frame: FrameStyle::LabelTop,
        frame_color: [78, 52, 36, 255],
        frame_text_color: [246, 238, 226, 255],
        frame_text: "NUESTRA CARTA".into(),
        ..Style::default()
    }
}

fn preset_party() -> Style {
    Style {
        module_shape: ModuleShape::Star,
        eye_frame: EyeFrame::Dotted,
        eye_ball: EyeBall::Star,
        fg: Fill::linear([110, 30, 170, 255], [220, 40, 100, 255], 45.0),
        eyes_custom: true,
        eye_frame_color: [110, 30, 170, 255],
        eye_ball_color: [230, 140, 0, 255],
        bg: Fill::solid([255, 252, 240, 255]),
        bg_radius: 4.0,
        frame: FrameStyle::LabelBottom,
        frame_color: [110, 30, 170, 255],
        frame_text: "¡ESTÁS INVITADO!".into(),
        ..Style::default()
    }
}

fn preset_slate() -> Style {
    Style {
        module_shape: ModuleShape::Cross,
        eye_frame: EyeFrame::Square,
        eye_ball: EyeBall::Square,
        fg: Fill::solid([44, 52, 64, 255]),
        bg: Fill::solid([240, 242, 245, 255]),
        bg_radius: 1.0,
        ..Style::default()
    }
}

fn preset_corporate() -> Style {
    Style {
        module_shape: ModuleShape::Square,
        eye_frame: EyeFrame::Rounded,
        eye_ball: EyeBall::Rounded,
        fg: Fill::solid([16, 42, 92, 255]),
        eyes_custom: true,
        eye_frame_color: [16, 42, 92, 255],
        eye_ball_color: [0, 120, 200, 255],
        bg: Fill::solid([255, 255, 255, 255]),
        bg_radius: 2.0,
        frame: FrameStyle::LabelBottom,
        frame_color: [16, 42, 92, 255],
        frame_text: "VISÍTANOS".into(),
        ..Style::default()
    }
}

fn preset_lavender() -> Style {
    Style {
        module_shape: ModuleShape::Rounded,
        module_scale: 0.9,
        eye_frame: EyeFrame::Dotted,
        eye_ball: EyeBall::Octagon,
        fg: Fill::linear([90, 60, 160, 255], [140, 90, 190, 255], 120.0),
        bg: Fill::solid([246, 242, 255, 255]),
        bg_radius: 3.5,
        ..Style::default()
    }
}

/// Lo que se guarda en un archivo de diseño (.qrdiseno.json).
#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DesignFile {
    pub style: Style,
    pub rules: Rules,
}

/// Luminancia relativa de un color (fórmula de las pautas de accesibilidad WCAG).
pub fn luminance(c: Rgba) -> f32 {
    fn ch(v: u8) -> f32 {
        let v = v as f32 / 255.0;
        if v <= 0.03928 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    }
    0.2126 * ch(c[0]) + 0.7152 * ch(c[1]) + 0.0722 * ch(c[2])
}

/// Contraste entre dos colores, de 1:1 (iguales) a 21:1 (negro sobre blanco).
pub fn contrast(a: Rgba, b: Rgba) -> f32 {
    let (la, lb) = (luminance(a), luminance(b));
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}
