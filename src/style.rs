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
    Drop,
    Octagon,
    Network,
    Organic,
}

impl ModuleShape {
    pub const ALL: [ModuleShape; 19] = [
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
        ModuleShape::Drop,
        ModuleShape::Octagon,
        ModuleShape::Network,
        ModuleShape::Organic,
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
            ModuleShape::Drop => "Gotas",
            ModuleShape::Octagon => "Octágonos",
            ModuleShape::Network => "Red",
            ModuleShape::Organic => "Orgánico",
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
    Petal,
}

impl EyeFrame {
    pub const ALL: [EyeFrame; 9] = [
        EyeFrame::Square,
        EyeFrame::Rounded,
        EyeFrame::ExtraRounded,
        EyeFrame::Circle,
        EyeFrame::Leaf,
        EyeFrame::Drop,
        EyeFrame::Octagon,
        EyeFrame::Dotted,
        EyeFrame::Petal,
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
            EyeFrame::Petal => "Pétalo",
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
    Plus,
    Flower,
    Dots,
}

impl EyeBall {
    pub const ALL: [EyeBall; 13] = [
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
        EyeBall::Plus,
        EyeBall::Flower,
        EyeBall::Dots,
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
            EyeBall::Plus => "Cruz",
            EyeBall::Flower => "Flor",
            EyeBall::Dots => "Nueve puntos",
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
    /// Burbuja de diálogo debajo del código, con un pico que lo señala.
    Bubble,
    /// Esquinas de "visor" de cámara alrededor del código.
    Visor,
    /// Como una foto instantánea: borde fino y una franja ancha abajo para el texto.
    Polaroid,
}

impl FrameStyle {
    pub const ALL: [FrameStyle; 7] = [
        FrameStyle::None,
        FrameStyle::LabelBottom,
        FrameStyle::LabelTop,
        FrameStyle::Border,
        FrameStyle::Bubble,
        FrameStyle::Visor,
        FrameStyle::Polaroid,
    ];
    pub fn label(self) -> &'static str {
        match self {
            FrameStyle::None => "Sin marco",
            FrameStyle::LabelBottom => "Etiqueta abajo",
            FrameStyle::LabelTop => "Etiqueta arriba",
            FrameStyle::Border => "Borde con texto",
            FrameStyle::Bubble => "Burbuja",
            FrameStyle::Visor => "Visor",
            FrameStyle::Polaroid => "Polaroid",
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
    /// Recortar el logo en forma de círculo.
    pub logo_round: bool,
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
            logo_round: false,
        }
    }
}

/// Categoría de un diseño de la galería.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cat {
    Classic,
    Colorful,
    Elegant,
    Fun,
    Framed,
}

impl Cat {
    pub const ALL: [Cat; 5] = [Cat::Classic, Cat::Colorful, Cat::Elegant, Cat::Fun, Cat::Framed];
    pub fn label(self) -> &'static str {
        match self {
            Cat::Classic => "Clásicos",
            Cat::Colorful => "Coloridos",
            Cat::Elegant => "Elegantes",
            Cat::Fun => "Divertidos",
            Cat::Framed => "Con marco",
        }
    }
}

/// Un diseño de la galería: su nombre (en español), su categoría y la función que lo crea.
pub struct Preset {
    pub name: &'static str,
    pub cat: Cat,
    pub style: fn() -> Style,
}

/// Los diseños de la galería, en el orden en que se enseñan.
pub const PRESETS: &[Preset] = &[
    Preset { name: "Clásico", cat: Cat::Classic, style: Style::default },
    Preset { name: "Océano", cat: Cat::Framed, style: preset_ocean },
    Preset { name: "Atardecer", cat: Cat::Framed, style: preset_sunset },
    Preset { name: "Bosque", cat: Cat::Colorful, style: preset_forest },
    Preset { name: "Uva", cat: Cat::Framed, style: preset_grape },
    Preset { name: "Elegante", cat: Cat::Elegant, style: preset_elegant },
    Preset { name: "Noche", cat: Cat::Classic, style: preset_night },
    Preset { name: "Caramelo", cat: Cat::Fun, style: preset_candy },
    Preset { name: "Menta", cat: Cat::Colorful, style: preset_mint },
    Preset { name: "Coral", cat: Cat::Framed, style: preset_coral },
    Preset { name: "Cielo", cat: Cat::Fun, style: preset_sky },
    Preset { name: "Dorado", cat: Cat::Elegant, style: preset_gold },
    Preset { name: "Lava", cat: Cat::Colorful, style: preset_lava },
    Preset { name: "Café", cat: Cat::Elegant, style: preset_coffee },
    Preset { name: "Fiesta", cat: Cat::Fun, style: preset_party },
    Preset { name: "Pizarra", cat: Cat::Classic, style: preset_slate },
    Preset { name: "Empresa", cat: Cat::Framed, style: preset_corporate },
    Preset { name: "Lavanda", cat: Cat::Colorful, style: preset_lavender },
    Preset { name: "Visor", cat: Cat::Framed, style: preset_visor },
    Preset { name: "Polaroid", cat: Cat::Framed, style: preset_polaroid },
    Preset { name: "Bocadillo", cat: Cat::Fun, style: preset_bubble },
    Preset { name: "Conexiones", cat: Cat::Colorful, style: preset_network },
    Preset { name: "Orgánico", cat: Cat::Colorful, style: preset_organic },
    Preset { name: "Lluvia", cat: Cat::Colorful, style: preset_rain },
    Preset { name: "Panal", cat: Cat::Colorful, style: preset_honey },
    Preset { name: "Flores", cat: Cat::Fun, style: preset_flowers },
    Preset { name: "Minimalista", cat: Cat::Classic, style: preset_minimal },
    Preset { name: "Arcoíris", cat: Cat::Fun, style: preset_rainbow },
    Preset { name: "Esmeralda", cat: Cat::Elegant, style: preset_emerald },
    Preset { name: "Tinta", cat: Cat::Classic, style: preset_ink },
    Preset { name: "Medianoche", cat: Cat::Elegant, style: preset_midnight },
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

fn preset_visor() -> Style {
    Style {
        module_shape: ModuleShape::Rounded,
        module_scale: 0.94,
        eye_frame: EyeFrame::Rounded,
        eye_ball: EyeBall::Rounded,
        fg: Fill::solid([24, 26, 40, 255]),
        bg: Fill::solid([255, 255, 255, 255]),
        bg_radius: 3.0,
        frame: FrameStyle::Visor,
        frame_color: [99, 91, 255, 255],
        ..Style::default()
    }
}

fn preset_polaroid() -> Style {
    Style {
        module_shape: ModuleShape::Square,
        eye_frame: EyeFrame::Square,
        eye_ball: EyeBall::Square,
        fg: Fill::solid([40, 40, 44, 255]),
        bg: Fill::solid([252, 250, 245, 255]),
        bg_radius: 0.6,
        frame: FrameStyle::Polaroid,
        frame_color: [255, 255, 255, 255],
        frame_text_color: [60, 60, 70, 255],
        frame_text: "BIENVENIDOS".into(),
        ..Style::default()
    }
}

fn preset_bubble() -> Style {
    Style {
        module_shape: ModuleShape::Circle,
        module_scale: 0.88,
        eye_frame: EyeFrame::ExtraRounded,
        eye_ball: EyeBall::Circle,
        fg: Fill::linear([80, 40, 190, 255], [150, 60, 220, 255], 45.0),
        bg: Fill::solid([248, 245, 255, 255]),
        bg_radius: 4.0,
        frame: FrameStyle::Bubble,
        frame_color: [99, 60, 220, 255],
        frame_text: "¡ESCANÉAME!".into(),
        ..Style::default()
    }
}

fn preset_network() -> Style {
    Style {
        module_shape: ModuleShape::Network,
        eye_frame: EyeFrame::Circle,
        eye_ball: EyeBall::Circle,
        fg: Fill::linear([0, 90, 120, 255], [0, 150, 140, 255], 45.0),
        bg: Fill::solid([240, 252, 250, 255]),
        bg_radius: 3.0,
        ..Style::default()
    }
}

fn preset_organic() -> Style {
    Style {
        module_shape: ModuleShape::Organic,
        eye_frame: EyeFrame::Leaf,
        eye_ball: EyeBall::Circle,
        fg: Fill::radial([30, 100, 40, 255], [80, 130, 30, 255]),
        bg: Fill::solid([246, 250, 236, 255]),
        bg_radius: 3.5,
        ..Style::default()
    }
}

fn preset_rain() -> Style {
    Style {
        module_shape: ModuleShape::Drop,
        module_scale: 0.95,
        eye_frame: EyeFrame::Drop,
        eye_ball: EyeBall::Drop,
        fg: Fill::linear([10, 60, 150, 255], [30, 130, 200, 255], 90.0),
        bg: Fill::solid([238, 246, 255, 255]),
        bg_radius: 3.0,
        ..Style::default()
    }
}

fn preset_honey() -> Style {
    Style {
        module_shape: ModuleShape::Octagon,
        eye_frame: EyeFrame::Octagon,
        eye_ball: EyeBall::Hexagon,
        fg: Fill::linear([120, 70, 0, 255], [170, 100, 10, 255], 60.0),
        bg: Fill::solid([255, 248, 225, 255]),
        bg_radius: 2.0,
        frame: FrameStyle::LabelBottom,
        frame_color: [120, 70, 0, 255],
        frame_text: "MÁS INFO".into(),
        ..Style::default()
    }
}

fn preset_flowers() -> Style {
    Style {
        module_shape: ModuleShape::Petal,
        eye_frame: EyeFrame::Petal,
        eye_ball: EyeBall::Flower,
        fg: Fill::linear([200, 30, 100, 255], [230, 90, 60, 255], 45.0),
        eyes_custom: true,
        eye_frame_color: [170, 20, 90, 255],
        eye_ball_color: [230, 120, 20, 255],
        bg: Fill::solid([255, 244, 248, 255]),
        bg_radius: 4.0,
        ..Style::default()
    }
}

fn preset_minimal() -> Style {
    Style {
        module_shape: ModuleShape::Square,
        module_scale: 0.78,
        eye_frame: EyeFrame::Square,
        eye_ball: EyeBall::Square,
        fg: Fill::solid([50, 54, 62, 255]),
        bg: Fill::solid([255, 255, 255, 255]),
        ..Style::default()
    }
}

fn preset_rainbow() -> Style {
    Style {
        module_shape: ModuleShape::Circle,
        module_scale: 0.92,
        eye_frame: EyeFrame::ExtraRounded,
        eye_ball: EyeBall::Star,
        fg: Fill::linear([200, 30, 60, 255], [40, 60, 200, 255], 0.0),
        bg: Fill::solid([255, 255, 255, 255]),
        bg_radius: 3.0,
        frame: FrameStyle::Bubble,
        frame_color: [120, 40, 160, 255],
        frame_text: "SÍGUENOS".into(),
        ..Style::default()
    }
}

fn preset_emerald() -> Style {
    Style {
        module_shape: ModuleShape::Diamond,
        eye_frame: EyeFrame::Octagon,
        eye_ball: EyeBall::Diamond,
        fg: Fill::linear([0, 90, 60, 255], [10, 120, 80, 255], 45.0),
        eyes_custom: true,
        eye_frame_color: [0, 80, 55, 255],
        eye_ball_color: [170, 130, 40, 255],
        bg: Fill::solid([248, 252, 248, 255]),
        bg_radius: 1.5,
        frame: FrameStyle::Border,
        frame_color: [0, 70, 50, 255],
        frame_text_color: [225, 195, 120, 255],
        frame_text: "PREMIUM".into(),
        frame_width: 1.5,
        ..Style::default()
    }
}

fn preset_ink() -> Style {
    Style {
        module_shape: ModuleShape::Rounded,
        eye_frame: EyeFrame::Petal,
        eye_ball: EyeBall::Dots,
        fg: Fill::solid([20, 20, 24, 255]),
        bg: Fill::solid([255, 255, 255, 255]),
        bg_radius: 2.0,
        ..Style::default()
    }
}

fn preset_midnight() -> Style {
    Style {
        module_shape: ModuleShape::Liquid,
        eye_frame: EyeFrame::Rounded,
        eye_ball: EyeBall::Plus,
        fg: Fill::linear([16, 24, 60, 255], [40, 50, 110, 255], 135.0),
        eyes_custom: true,
        eye_frame_color: [16, 24, 60, 255],
        eye_ball_color: [120, 130, 160, 255],
        bg: Fill::solid([246, 247, 252, 255]),
        bg_radius: 2.0,
        frame: FrameStyle::Polaroid,
        frame_color: [16, 24, 60, 255],
        frame_text_color: [230, 232, 245, 255],
        frame_text: "VISÍTANOS".into(),
        ..Style::default()
    }
}

/// Paletas que combinan bien: color del código (inicio, final), fondo y color de acento
/// (ojos y marco). Las usa el botón «Sorpréndeme».
const PALETTES: [(Rgba, Rgba, Rgba, Rgba); 12] = [
    ([0, 92, 151, 255], [0, 160, 190, 255], [255, 255, 255, 255], [0, 60, 120, 255]),
    ([200, 40, 90, 255], [240, 120, 40, 255], [255, 248, 242, 255], [180, 30, 80, 255]),
    ([20, 100, 60, 255], [70, 150, 70, 255], [244, 250, 240, 255], [10, 80, 50, 255]),
    ([88, 28, 135, 255], [180, 60, 170, 255], [250, 245, 255, 255], [70, 20, 120, 255]),
    ([30, 30, 34, 255], [70, 70, 80, 255], [250, 247, 240, 255], [176, 141, 87, 255]),
    ([16, 42, 92, 255], [0, 120, 200, 255], [255, 255, 255, 255], [16, 42, 92, 255]),
    ([150, 20, 20, 255], [210, 80, 20, 255], [255, 247, 238, 255], [130, 20, 20, 255]),
    ([0, 100, 100, 255], [20, 140, 120, 255], [236, 250, 246, 255], [0, 80, 80, 255]),
    ([78, 52, 36, 255], [130, 90, 60, 255], [246, 238, 226, 255], [78, 52, 36, 255]),
    ([60, 40, 160, 255], [120, 80, 230, 255], [246, 244, 255, 255], [99, 91, 255, 255]),
    ([190, 30, 110, 255], [120, 40, 190, 255], [255, 240, 247, 255], [150, 20, 100, 255]),
    ([44, 52, 64, 255], [80, 90, 110, 255], [240, 242, 245, 255], [44, 52, 64, 255]),
];

/// Un diseño al azar a partir de un número cualquiera (`seed`). Solo usa formas que se
/// leen bien; la interfaz además comprueba que el resultado se pueda escanear.
pub fn random_style(seed: u64) -> Style {
    let mut x = seed ^ 0x9E37_79B9_7F4A_7C15;
    let mut next = |n: usize| -> usize {
        // xorshift64*: suficiente para elegir opciones al azar.
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        (x.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 33) as usize % n.max(1)
    };
    let shapes = [
        ModuleShape::Square, ModuleShape::Rounded, ModuleShape::Circle, ModuleShape::Liquid,
        ModuleShape::Diamond, ModuleShape::VBars, ModuleShape::HBars, ModuleShape::Hexagon,
        ModuleShape::Petal, ModuleShape::Mosaic, ModuleShape::Drop, ModuleShape::Octagon,
        ModuleShape::Network, ModuleShape::Organic, ModuleShape::Heart, ModuleShape::Star,
    ];
    let frames = [FrameStyle::None, FrameStyle::None, FrameStyle::LabelBottom, FrameStyle::Bubble, FrameStyle::Visor, FrameStyle::Polaroid, FrameStyle::Border];
    let texts = ["ESCANÉAME", "¡ESCANÉAME!", "MÁS INFO", "SÍGUENOS", "BIENVENIDOS", "VISÍTANOS"];
    let (c1, c2, bg, accent) = PALETTES[next(PALETTES.len())];
    let fg = match next(3) {
        0 => Fill::solid(c1),
        1 => Fill::linear(c1, c2, [0.0, 45.0, 90.0, 135.0][next(4)]),
        _ => Fill::radial(c1, c2),
    };
    let frame = frames[next(frames.len())];
    let eyes_custom = next(2) == 0;
    let (frame_color, frame_text_color) = match frame {
        FrameStyle::Polaroid => ([255, 255, 255, 255], [60, 60, 70, 255]),
        _ => (accent, [255, 255, 255, 255]),
    };
    Style {
        module_shape: shapes[next(shapes.len())],
        module_scale: [1.0, 0.95, 0.9][next(3)],
        eye_frame: EyeFrame::ALL[next(EyeFrame::ALL.len())],
        eye_ball: EyeBall::ALL[next(EyeBall::ALL.len())],
        fg,
        eyes_custom,
        eye_frame_color: accent,
        eye_ball_color: c2,
        bg: Fill::solid(bg),
        bg_radius: [0.0, 1.5, 3.0, 4.0][next(4)],
        frame,
        frame_color,
        frame_text_color,
        frame_text: texts[next(texts.len())].into(),
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
