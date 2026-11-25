//! Dibujo del QR decorado.
//!
//! El proceso tiene tres pasos:
//! 1. [`build_scene`] convierte el código QR y el estilo en una *escena*: capas de figuras
//!    geométricas ([`Prim`]) medidas en módulos (1 módulo = 1 unidad).
//! 2. [`rasterize`] pinta la escena en una imagen PNG del tamaño pedido, con bordes suaves
//!    (cada píxel se cubre según su distancia al borde de la figura), degradados, logo y texto.
//! 3. [`to_svg`] escribe la misma escena como SVG (vectorial, para imprenta).
//!
//! Así la vista previa, el PNG y el SVG salen siempre iguales. Además, [`verify`] intenta
//! leer la imagen como lo haría un móvil para avisar si el diseño es difícil de escanear.

use crate::qr::QrResult;
use crate::style::*;
use image::RgbaImage;

/// Una figura geométrica básica, en unidades de módulo.
#[derive(Clone, Debug)]
pub enum Prim {
    /// Rectángulo con radio por esquina: [arriba-izq, arriba-der, abajo-der, abajo-izq].
    RRect { x: f32, y: f32, w: f32, h: f32, r: [f32; 4] },
    Circle { cx: f32, cy: f32, r: f32 },
    Poly(Vec<(f32, f32)>),
    /// Forma con un agujero (el marco de los ojos, el borde).
    Ring(Box<Prim>, Box<Prim>),
}

impl Prim {
    fn bounds(&self) -> (f32, f32, f32, f32) {
        match self {
            Prim::RRect { x, y, w, h, .. } => (*x, *y, x + w, y + h),
            Prim::Circle { cx, cy, r } => (cx - r, cy - r, cx + r, cy + r),
            Prim::Poly(v) => {
                let mut b = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
                for &(x, y) in v {
                    b.0 = b.0.min(x);
                    b.1 = b.1.min(y);
                    b.2 = b.2.max(x);
                    b.3 = b.3.max(y);
                }
                b
            }
            Prim::Ring(o, _) => o.bounds(),
        }
    }

    /// Distancia con signo (negativa dentro), en unidades de módulo.
    fn sd(&self, px: f32, py: f32) -> f32 {
        match self {
            Prim::RRect { x, y, w, h, r } => {
                let hx = w * 0.5;
                let hy = h * 0.5;
                let qx0 = px - (x + hx);
                let qy0 = py - (y + hy);
                let rad = if qx0 > 0.0 {
                    if qy0 > 0.0 { r[2] } else { r[1] }
                } else if qy0 > 0.0 {
                    r[3]
                } else {
                    r[0]
                };
                let rad = rad.min(hx).min(hy).max(0.0);
                let qx = qx0.abs() - hx + rad;
                let qy = qy0.abs() - hy + rad;
                let outside = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt();
                qx.max(qy).min(0.0) + outside - rad
            }
            Prim::Circle { cx, cy, r } => ((px - cx).powi(2) + (py - cy).powi(2)).sqrt() - r,
            Prim::Poly(v) => sd_poly(px, py, v),
            Prim::Ring(o, i) => o.sd(px, py).max(-i.sd(px, py)),
        }
    }
}

/// Distancia con signo a un polígono (negativa dentro). Algoritmo de Inigo Quilez.
fn sd_poly(px: f32, py: f32, v: &[(f32, f32)]) -> f32 {
    let n = v.len();
    let mut d = (px - v[0].0).powi(2) + (py - v[0].1).powi(2);
    let mut s = 1.0f32;
    let mut j = n - 1;
    for i in 0..n {
        let (ex, ey) = (v[j].0 - v[i].0, v[j].1 - v[i].1);
        let (wx, wy) = (px - v[i].0, py - v[i].1);
        let t = ((wx * ex + wy * ey) / (ex * ex + ey * ey)).clamp(0.0, 1.0);
        let (bx, by) = (wx - ex * t, wy - ey * t);
        d = d.min(bx * bx + by * by);
        let c1 = py >= v[i].1;
        let c2 = py < v[j].1;
        let c3 = ex * wy > ey * wx;
        if (c1 && c2 && c3) || (!c1 && !c2 && !c3) {
            s = -s;
        }
        j = i;
    }
    s * d.sqrt()
}

/// Una capa: figuras pintadas con el mismo relleno.
#[derive(Clone)]
pub struct Layer {
    pub prims: Vec<Prim>,
    pub fill: Fill,
    /// Caja de referencia del degradado (x, y, ancho, alto).
    pub gbox: (f32, f32, f32, f32),
}

/// El texto del marco: contenido, centro, altura, ancho máximo y color.
#[derive(Clone)]
pub struct TextSpec {
    pub text: String,
    pub cx: f32,
    pub cy: f32,
    pub size: f32,
    pub max_w: f32,
    pub color: Rgba,
}

/// Todo lo que hay que dibujar, en unidades de módulo (`w` × `h`).
#[derive(Clone)]
pub struct Scene {
    pub w: f32,
    pub h: f32,
    pub layers: Vec<Layer>,
    /// Caja donde va el logo (x, y, lado).
    pub logo_box: Option<(f32, f32, f32)>,
    pub text: Option<TextSpec>,
    /// Recortar el logo en círculo.
    pub logo_round: bool,
    /// Esquina del primer módulo y número de módulos por lado.
    pub qr_origin: (f32, f32),
    pub qr_size: i32,
}

/// Refleja los radios de las esquinas en horizontal (`mx`) y/o vertical (`my`).
fn mirror(r: [f32; 4], mx: bool, my: bool) -> [f32; 4] {
    let mut r = r;
    if mx {
        r = [r[1], r[0], r[3], r[2]];
    }
    if my {
        r = [r[3], r[2], r[1], r[0]];
    }
    r
}

/// Estrella de cinco puntas (radio exterior `ro`, interior `ri`).
fn star(cx: f32, cy: f32, ro: f32, ri: f32) -> Prim {
    let mut v = Vec::with_capacity(10);
    for k in 0..10 {
        let a = -std::f32::consts::FRAC_PI_2 + k as f32 * std::f32::consts::PI / 5.0;
        let r = if k % 2 == 0 { ro } else { ri };
        v.push((cx + r * a.cos(), cy + r * a.sin()));
    }
    Prim::Poly(v)
}

/// Corazón (curva paramétrica clásica) de tamaño `s`.
fn heart(cx: f32, cy: f32, s: f32) -> Prim {
    let k = 0.5 * s / 16.0 * 1.06;
    let mut v = Vec::with_capacity(32);
    for i in 0..32 {
        let t = i as f32 / 32.0 * std::f32::consts::TAU;
        let x = 16.0 * t.sin().powi(3);
        let y = 13.0 * t.cos() - 5.0 * (2.0 * t).cos() - 2.0 * (3.0 * t).cos() - (4.0 * t).cos();
        v.push((cx + x * k, cy - (y + 2.5) * k));
    }
    Prim::Poly(v)
}

/// Rombo con semidiagonal `e`.
fn diamond(cx: f32, cy: f32, e: f32) -> Prim {
    Prim::Poly(vec![(cx, cy - e), (cx + e, cy), (cx, cy + e), (cx - e, cy)])
}

/// Cruz "+" con brazos de largo `e` y medio grosor `a`.
fn plus(cx: f32, cy: f32, e: f32, a: f32) -> Prim {
    Prim::Poly(vec![
        (cx - a, cy - e),
        (cx + a, cy - e),
        (cx + a, cy - a),
        (cx + e, cy - a),
        (cx + e, cy + a),
        (cx + a, cy + a),
        (cx + a, cy + e),
        (cx - a, cy + e),
        (cx - a, cy + a),
        (cx - e, cy + a),
        (cx - e, cy - a),
        (cx - a, cy - a),
    ])
}

/// Octágono dentro de un cuadrado de lado `size`, con las esquinas cortadas `c`.
fn octagon(x: f32, y: f32, size: f32, c: f32) -> Prim {
    Prim::Poly(vec![
        (x + c, y),
        (x + size - c, y),
        (x + size, y + c),
        (x + size, y + size - c),
        (x + size - c, y + size),
        (x + c, y + size),
        (x, y + size - c),
        (x, y + c),
    ])
}

/// Hexágono regular de radio `r`.
fn hexagon(cx: f32, cy: f32, r: f32) -> Prim {
    let mut v = Vec::with_capacity(6);
    for k in 0..6 {
        let a = std::f32::consts::FRAC_PI_6 + k as f32 * std::f32::consts::FRAC_PI_3;
        v.push((cx + r * a.cos(), cy + r * a.sin()));
    }
    Prim::Poly(v)
}

/// Cruz en forma de "X" (la cruz "+" girada 45°).
fn cross(cx: f32, cy: f32, e: f32, a: f32) -> Prim {
    if let Prim::Poly(v) = plus(0.0, 0.0, e, a) {
        let (sn, cs) = std::f32::consts::FRAC_PI_4.sin_cos();
        Prim::Poly(v.into_iter().map(|(x, y)| (cx + x * cs - y * sn, cy + x * sn + y * cs)).collect())
    } else {
        unreachable!()
    }
}

/// Rectángulo con el mismo radio en las cuatro esquinas.
fn rr(x: f32, y: f32, w: f32, h: f32, r: f32) -> Prim {
    Prim::RRect { x, y, w, h, r: [r; 4] }
}

/// La figura de un módulo oscuro cuya casilla empieza en (fx, fy).
/// `s` es la escala del módulo (1.0 llena la casilla) y `nb` dice si hay módulos oscuros
/// arriba, abajo, a la izquierda y a la derecha (para las formas que se unen entre sí).
fn module_prim(shape: ModuleShape, s: f32, fx: f32, fy: f32, nb: [bool; 4]) -> Prim {
    let inset = (1.0 - s) * 0.5;
    let (cx, cy) = (fx + 0.5, fy + 0.5);
    let [t, b, l, r] = nb;
    match shape {
        ModuleShape::Square => rr(fx + inset, fy + inset, s, s, 0.0),
        ModuleShape::Rounded => rr(fx + inset, fy + inset, s, s, 0.32 * s),
        ModuleShape::Circle => Prim::Circle { cx, cy, r: 0.5 * s },
        ModuleShape::SmallDots => Prim::Circle { cx, cy, r: 0.36 * s },
        ModuleShape::Liquid => {
            // Se redondea una esquina solo si no hay vecino en ninguno de sus dos lados:
            // así los grupos de módulos parecen gotas unidas.
            let rad = 0.5 * s;
            let c = |a: bool, b: bool| if !a && !b { rad } else { 0.0 };
            let x0 = if l { fx } else { fx + inset };
            let x1 = if r { fx + 1.0 } else { fx + 1.0 - inset };
            let y0 = if t { fy } else { fy + inset };
            let y1 = if b { fy + 1.0 } else { fy + 1.0 - inset };
            Prim::RRect { x: x0, y: y0, w: x1 - x0, h: y1 - y0, r: [c(t, l), c(t, r), c(b, r), c(b, l)] }
        }
        ModuleShape::Diamond => diamond(cx, cy, 0.5 * s + 0.04),
        ModuleShape::VBars => {
            let y0 = if t { fy } else { fy + inset * 0.5 };
            let y1 = if b { fy + 1.0 } else { fy + 1.0 - inset * 0.5 };
            let rt = if t { 0.0 } else { 0.5 * s };
            let rb = if b { 0.0 } else { 0.5 * s };
            Prim::RRect { x: fx + inset, y: y0, w: s, h: y1 - y0, r: [rt, rt, rb, rb] }
        }
        ModuleShape::HBars => {
            let x0 = if l { fx } else { fx + inset * 0.5 };
            let x1 = if r { fx + 1.0 } else { fx + 1.0 - inset * 0.5 };
            let rl = if l { 0.0 } else { 0.5 * s };
            let rr_ = if r { 0.0 } else { 0.5 * s };
            Prim::RRect { x: x0, y: fy + inset, w: x1 - x0, h: s, r: [rl, rr_, rr_, rl] }
        }
        ModuleShape::Star => star(cx, cy, 0.62 * s, 0.3 * s),
        ModuleShape::Plus => plus(cx, cy, 0.52 * s, 0.22 * s),
        ModuleShape::Heart => heart(cx, cy, s),
        ModuleShape::Hexagon => hexagon(cx, cy, 0.58 * s),
        ModuleShape::Petal => {
            let r = 0.5 * s;
            Prim::RRect { x: fx + inset, y: fy + inset, w: s, h: s, r: [r, 0.06, r, 0.06] }
        }
        ModuleShape::Cross => cross(cx, cy, 0.66 * s, 0.2 * s),
        ModuleShape::Mosaic => {
            let m = 0.84 * s;
            rr(cx - m * 0.5, cy - m * 0.5, m, m, 0.14 * m)
        }
        ModuleShape::Drop => {
            let r = 0.5 * s;
            Prim::RRect { x: fx + inset, y: fy + inset, w: s, h: s, r: [r, r, 0.08, r] }
        }
        ModuleShape::Octagon => octagon(fx + inset, fy + inset, s, 0.29 * s),
        // El nodo; las uniones con los vecinos las añade `module_prims`.
        ModuleShape::Network => Prim::Circle { cx, cy, r: 0.36 * s },
        ModuleShape::Organic => {
            // Tamaño "al azar" pero siempre igual para la misma casilla.
            let hsh = ((fx * 73.0).to_bits() ^ (fy * 151.0).to_bits().rotate_left(13)).wrapping_mul(0x9E37_79B1);
            let t = (hsh >> 24) as f32 / 255.0;
            Prim::Circle { cx, cy, r: s * (0.36 + 0.15 * t) }
        }
    }
}

/// Como `module_prim`, pero añade a `out` todas las piezas del módulo (la forma "Red"
/// dibuja además las uniones con los vecinos de la derecha y de abajo).
fn module_prims(shape: ModuleShape, s: f32, fx: f32, fy: f32, nb: [bool; 4], out: &mut Vec<Prim>) {
    out.push(module_prim(shape, s, fx, fy, nb));
    if shape == ModuleShape::Network {
        let (cx, cy) = (fx + 0.5, fy + 0.5);
        let t = 0.26 * s;
        if nb[3] {
            out.push(rr(cx, cy - t * 0.5, 1.0, t, 0.0));
        }
        if nb[1] {
            out.push(rr(cx - t * 0.5, cy, t, 1.0, 0.0));
        }
    }
}

/// Las figuras de un ojo de 7×7 módulos con su esquina en (x, y): el marco (una o varias
/// piezas) y el centro de 3×3. `mx`/`my` reflejan las formas asimétricas.
fn eye_prims(frame: EyeFrame, ball: EyeBall, x: f32, y: f32, mx: bool, my: bool) -> (Vec<Prim>, Vec<Prim>) {
    let fr = |outer: [f32; 4], inner: [f32; 4]| {
        Prim::Ring(
            Box::new(Prim::RRect { x, y, w: 7.0, h: 7.0, r: mirror(outer, mx, my) }),
            Box::new(Prim::RRect { x: x + 1.0, y: y + 1.0, w: 5.0, h: 5.0, r: mirror(inner, mx, my) }),
        )
    };
    let frames = match frame {
        EyeFrame::Square => vec![fr([0.0; 4], [0.0; 4])],
        EyeFrame::Rounded => vec![fr([1.8; 4], [1.0; 4])],
        EyeFrame::ExtraRounded => vec![fr([3.0; 4], [2.1; 4])],
        EyeFrame::Circle => vec![Prim::Ring(
            Box::new(Prim::Circle { cx: x + 3.5, cy: y + 3.5, r: 3.5 }),
            Box::new(Prim::Circle { cx: x + 3.5, cy: y + 3.5, r: 2.5 }),
        )],
        EyeFrame::Leaf => vec![fr([3.2, 0.0, 3.2, 0.0], [2.2, 0.0, 2.2, 0.0])],
        EyeFrame::Drop => vec![fr([3.2, 3.2, 0.0, 3.2], [2.2, 2.2, 0.0, 2.2])],
        EyeFrame::Petal => vec![fr([3.4, 0.7, 0.7, 0.7], [2.4, 0.3, 0.3, 0.3])],
        EyeFrame::Octagon => vec![Prim::Ring(
            Box::new(octagon(x, y, 7.0, 2.1)),
            Box::new(octagon(x + 1.0, y + 1.0, 5.0, 1.5)),
        )],
        // Un círculo por cada módulo del borde del ojo.
        EyeFrame::Dotted => {
            let mut v = Vec::with_capacity(24);
            for i in 0..7 {
                for j in 0..7 {
                    if i == 0 || j == 0 || i == 6 || j == 6 {
                        v.push(Prim::Circle { cx: x + i as f32 + 0.5, cy: y + j as f32 + 0.5, r: 0.5 });
                    }
                }
            }
            v
        }
    };
    let (bx, by) = (x + 2.0, y + 2.0);
    let sq = |r: [f32; 4]| Prim::RRect { x: bx, y: by, w: 3.0, h: 3.0, r: mirror(r, mx, my) };
    let (bcx, bcy) = (bx + 1.5, by + 1.5);
    let ball = match ball {
        // Nueve puntos (uno por módulo del centro).
        EyeBall::Dots => {
            let mut v = Vec::with_capacity(9);
            for i in 0..3 {
                for j in 0..3 {
                    v.push(Prim::Circle { cx: bx + i as f32 + 0.5, cy: by + j as f32 + 0.5, r: 0.47 });
                }
            }
            return (frames, v);
        }
        EyeBall::Plus => plus(bcx, bcy, 1.62, 0.6),
        EyeBall::Flower => {
            // Flor de seis pétalos: el radio sube y baja con el ángulo.
            let mut v = Vec::with_capacity(60);
            for k in 0..60 {
                let a = k as f32 / 60.0 * std::f32::consts::TAU;
                let r = 1.18 + 0.48 * (6.0 * a).cos();
                v.push((bcx + r * a.cos(), bcy + r * a.sin()));
            }
            Prim::Poly(v)
        }
        EyeBall::Square => sq([0.0; 4]),
        EyeBall::Rounded => sq([0.8; 4]),
        EyeBall::Circle => Prim::Circle { cx: bx + 1.5, cy: by + 1.5, r: 1.5 },
        EyeBall::Leaf => sq([1.5, 0.0, 1.5, 0.0]),
        EyeBall::Drop => sq([1.5, 1.5, 0.0, 1.5]),
        EyeBall::Diamond => diamond(bx + 1.5, by + 1.5, 1.85),
        EyeBall::Star => star(bx + 1.5, by + 1.5, 2.0, 0.95),
        EyeBall::Heart => heart(bx + 1.5, by + 1.5, 3.4),
        EyeBall::Hexagon => hexagon(bx + 1.5, by + 1.5, 1.72),
        EyeBall::Octagon => octagon(bx, by, 3.0, 0.9),
    };
    (frames, vec![ball])
}

/// Dibuja unas figuras en blanco sobre fondo transparente (para los iconos de la interfaz,
/// que luego se tiñen del color del tema).
fn icon_from(prims: Vec<Prim>, w: f32, font: &ab_glyph::FontArc, px: u32) -> RgbaImage {
    let scene = Scene {
        w,
        h: w,
        layers: vec![Layer { prims, fill: Fill::solid([255, 255, 255, 255]), gbox: (0.0, 0.0, w, w) }],
        logo_box: None,
        logo_round: false,
        text: None,
        qr_origin: (0.0, 0.0),
        qr_size: 1,
    };
    rasterize(&scene, None, font, px)
}

/// El icono de la aplicación: un cuadrado redondeado con degradado violeta y un QR blanco
/// estilizado (tres ojos redondeados y algunos puntos). Se usa para la ventana, los
/// instaladores y las apps de móvil.
pub fn app_icon(px: u32) -> RgbaImage {
    icon_impl(px, false)
}

/// Como `app_icon`, pero `full` llena todo el cuadrado sin esquinas transparentes (iOS
/// redondea él mismo las esquinas y no admite transparencia).
fn icon_impl(px: u32, full: bool) -> RgbaImage {
    let w = 24.0;
    let mut white = Vec::new();
    for (x, y) in [(4.0, 4.0), (13.0, 4.0), (4.0, 13.0)] {
        let (f, b) = eye_prims(EyeFrame::Rounded, EyeBall::Rounded, x, y, false, false);
        white.extend(f);
        white.extend(b);
    }
    // Puntos del cuadrante de abajo a la derecha.
    for (x, y) in [(13, 13), (15, 13), (17, 14), (13, 15), (14, 17), (16, 16), (18, 18), (16, 19), (13, 19), (19, 16)] {
        white.push(Prim::Circle { cx: x as f32 + 0.5, cy: y as f32 + 0.5, r: 0.62 });
    }
    let scene = Scene {
        w,
        h: w,
        layers: vec![
            Layer {
                prims: vec![if full { rr(0.0, 0.0, w, w, 0.0) } else { rr(0.6, 0.6, w - 1.2, w - 1.2, 5.4) }],
                fill: Fill::linear([110, 96, 255, 255], [72, 52, 212, 255], 60.0),
                gbox: (0.0, 0.0, w, w),
            },
            Layer { prims: white, fill: Fill::solid([255, 255, 255, 255]), gbox: (0.0, 0.0, w, w) },
        ],
        logo_box: None,
        logo_round: false,
        text: None,
        qr_origin: (0.0, 0.0),
        qr_size: 1,
    };
    let font = ab_glyph::FontArc::try_from_slice(epaint_default_fonts::UBUNTU_LIGHT).expect("fuente interna");
    rasterize(&scene, None, &font, px)
}

/// Escribe todos los iconos que necesitan los instaladores en una carpeta:
/// `icon.png` (1024 px), `icon.ico` (Windows, varios tamaños), los `mipmap-*` de Android
/// y los iconos de iOS. Se usa con `creador-qr --exportar-iconos <carpeta>`.
pub fn export_icons(dir: &std::path::Path) -> Result<(), String> {
    use image::codecs::ico::{IcoEncoder, IcoFrame};
    let e = |x: image::ImageError| x.to_string();
    std::fs::create_dir_all(dir).map_err(|x| x.to_string())?;
    app_icon(1024).save(dir.join("icon.png")).map_err(e)?;
    // Windows: un .ico con varios tamaños.
    let mut frames = Vec::new();
    for s in [16u32, 24, 32, 48, 64, 128, 256] {
        let img = app_icon(s);
        frames.push(IcoFrame::as_png(img.as_raw(), s, s, image::ExtendedColorType::Rgba8).map_err(e)?);
    }
    let file = std::fs::File::create(dir.join("icon.ico")).map_err(|x| x.to_string())?;
    IcoEncoder::new(file).encode_images(&frames).map_err(e)?;
    // Android: res/mipmap-*/ic_launcher.png
    for (name, s) in [("mdpi", 48u32), ("hdpi", 72), ("xhdpi", 96), ("xxhdpi", 144), ("xxxhdpi", 192)] {
        let d = dir.join("android").join("res").join(format!("mipmap-{name}"));
        std::fs::create_dir_all(&d).map_err(|x| x.to_string())?;
        app_icon(s).save(d.join("ic_launcher.png")).map_err(e)?;
    }
    // iOS: iconos de la pantalla de inicio (iPhone y iPad).
    let d = dir.join("ios");
    std::fs::create_dir_all(&d).map_err(|x| x.to_string())?;
    for (name, s) in [("AppIcon60x60@2x", 120u32), ("AppIcon60x60@3x", 180), ("AppIcon76x76@2x~ipad", 152), ("AppIcon83.5x83.5@2x~ipad", 167)] {
        icon_impl(s, true).save(d.join(format!("{name}.png"))).map_err(e)?;
    }
    Ok(())
}

/// Icono de una forma de módulo: un pequeño trozo de patrón de 4×4.
pub fn module_icon(shape: ModuleShape, font: &ab_glyph::FontArc, px: u32) -> RgbaImage {
    const P: [[u8; 4]; 4] = [[1, 1, 0, 1], [1, 0, 0, 1], [0, 1, 1, 1], [1, 1, 0, 0]];
    let on = |x: i32, y: i32| x >= 0 && y >= 0 && x < 4 && y < 4 && P[y as usize][x as usize] == 1;
    let mut prims = Vec::new();
    for y in 0..4 {
        for x in 0..4 {
            if on(x, y) {
                let nb = [on(x, y - 1), on(x, y + 1), on(x - 1, y), on(x + 1, y)];
                module_prims(shape, 0.92, 0.3 + x as f32, 0.3 + y as f32, nb, &mut prims);
            }
        }
    }
    icon_from(prims, 4.6, font, px)
}

/// Icono del marco de un ojo (con un centro cuadrado).
pub fn eye_frame_icon(frame: EyeFrame, font: &ab_glyph::FontArc, px: u32) -> RgbaImage {
    let (mut prims, _) = eye_prims(frame, EyeBall::Square, 0.6, 0.6, false, false);
    prims.push(Prim::RRect { x: 2.6, y: 2.6, w: 3.0, h: 3.0, r: [0.0; 4] });
    icon_from(prims, 8.2, font, px)
}

/// Icono del centro de un ojo (solo el centro, más grande).
pub fn eye_ball_icon(ball: EyeBall, font: &ab_glyph::FontArc, px: u32) -> RgbaImage {
    let (_, b) = eye_prims(EyeFrame::Square, ball, -1.4, -1.4, false, false);
    icon_from(b, 4.2, font, px)
}

/// Convierte un código QR y un estilo en una escena lista para dibujar.
///
/// Orden de las capas (de abajo arriba): fondo, marco, módulos, marcos de los ojos,
/// centros de los ojos y fondo del logo. El logo y el texto se añaden al pintar.
pub fn build_scene(qr: &QrResult, style: &Style, rules: &Rules, has_logo: bool) -> Scene {
    let n = qr.size as f32;
    let q = rules.quiet_zone as f32;
    let qb = n + 2.0 * q;
    let ts = style.frame_text_size * qb;
    let text_on = !style.frame_text.trim().is_empty();
    // Las etiquetas y la burbuja sin texto no tienen sentido: se quitan.
    let needs_text = matches!(style.frame, FrameStyle::LabelBottom | FrameStyle::LabelTop | FrameStyle::Bubble);
    let frame = if needs_text && !text_on { FrameStyle::None } else { style.frame };

    let (w, h, ox, oy);
    let mut frame_prims: Vec<Prim> = Vec::new();
    let mut text: Option<TextSpec> = None;
    match frame {
        FrameStyle::None => {
            w = qb;
            h = qb;
            ox = q;
            oy = q;
        }
        FrameStyle::LabelBottom | FrameStyle::LabelTop => {
            let gap = (q * 0.5).max(1.0);
            let bh = ts * 1.9;
            let margin = (q * 0.7).max(1.0);
            w = qb;
            let bx = q.max(1.0).min(qb * 0.2);
            let bw = w - 2.0 * bx;
            let by;
            if frame == FrameStyle::LabelBottom {
                ox = q;
                oy = q;
                by = q + n + gap;
                h = by + bh + margin;
            } else {
                by = margin;
                ox = q;
                oy = margin + bh + gap;
                h = oy + n + q;
            }
            frame_prims.push(rr(bx, by, bw, bh, bh * 0.32));
            text = Some(TextSpec {
                text: style.frame_text.trim().to_string(),
                cx: bx + bw * 0.5,
                cy: by + bh * 0.5,
                size: ts,
                max_w: bw * 0.9,
                color: style.frame_text_color,
            });
        }
        FrameStyle::Border => {
            let bw = style.frame_width.max(0.3);
            let tb = if text_on { ts * 1.9 } else { 0.0 };
            w = qb + 2.0 * bw;
            h = qb + 2.0 * bw + tb;
            ox = bw + q;
            oy = bw + q;
            let ir = (style.bg_radius - bw * 0.5).max(0.0);
            frame_prims.push(Prim::Ring(
                Box::new(rr(0.0, 0.0, w, h, style.bg_radius)),
                Box::new(rr(bw, bw, qb, qb, ir)),
            ));
            if text_on {
                text = Some(TextSpec {
                    text: style.frame_text.trim().to_string(),
                    cx: w * 0.5,
                    cy: bw + qb + (tb + bw) * 0.5,
                    size: ts,
                    max_w: w * 0.88,
                    color: style.frame_text_color,
                });
            }
        }
        FrameStyle::Bubble => {
            // Como la etiqueta de abajo, pero en forma de pastilla con un pico hacia el código.
            let gap = (q * 0.5).max(1.0) + 0.9;
            let bh = ts * 1.9;
            let margin = (q * 0.7).max(1.0);
            w = qb;
            ox = q;
            oy = q;
            let bx = q.max(1.0).min(qb * 0.2);
            let bw = w - 2.0 * bx;
            let by = q + n + gap;
            h = by + bh + margin;
            frame_prims.push(rr(bx, by, bw, bh, bh * 0.5));
            let (cx, pw) = (w * 0.5, 1.5);
            frame_prims.push(Prim::Poly(vec![(cx - pw, by + 0.3), (cx, by - 1.15), (cx + pw, by + 0.3)]));
            text = Some(TextSpec {
                text: style.frame_text.trim().to_string(),
                cx: bx + bw * 0.5,
                cy: by + bh * 0.5,
                size: ts,
                max_w: bw * 0.86,
                color: style.frame_text_color,
            });
        }
        FrameStyle::Visor => {
            // Esquinas de visor por fuera de la zona silenciosa; el texto (opcional) va
            // debajo con el color del marco.
            let e = (q * 0.25).max(1.6);
            let tb = if text_on { ts * 1.8 } else { 0.0 };
            w = qb + 2.0 * e;
            h = qb + 2.0 * e + tb;
            ox = e + q;
            oy = e + q;
            let t = 0.9;
            let len = (qb * 0.2).max(3.0);
            let (x0, y0, x1, y1) = (e * 0.35, e * 0.35, w - e * 0.35, qb + 2.0 * e - e * 0.35);
            for (cx, cy, sx, sy) in [(x0, y0, 1.0, 1.0), (x1, y0, -1.0, 1.0), (x0, y1, 1.0, -1.0), (x1, y1, -1.0, -1.0)] {
                let hx = if sx > 0.0 { cx } else { cx - len };
                let hy = if sy > 0.0 { cy } else { cy - t };
                frame_prims.push(rr(hx, hy, len, t, t * 0.5));
                let vx = if sx > 0.0 { cx } else { cx - t };
                let vy = if sy > 0.0 { cy } else { cy - len };
                frame_prims.push(rr(vx, vy, t, len, t * 0.5));
            }
            if text_on {
                text = Some(TextSpec {
                    text: style.frame_text.trim().to_string(),
                    cx: w * 0.5,
                    cy: qb + 2.0 * e + tb * 0.42,
                    size: ts,
                    max_w: w * 0.85,
                    color: style.frame_color,
                });
            }
        }
        FrameStyle::Polaroid => {
            // Borde fino arriba y a los lados y una franja ancha abajo, como una foto.
            let b = (q * 0.35).max(1.4);
            let bottom = if text_on { ts * 3.2 } else { b * 2.6 };
            w = qb + 2.0 * b;
            h = qb + b + bottom;
            ox = b + q;
            oy = b + q;
            frame_prims.push(Prim::Ring(
                Box::new(rr(0.0, 0.0, w, h, style.bg_radius.min(1.5))),
                Box::new(rr(b, b, qb, qb, 0.3)),
            ));
            if text_on {
                text = Some(TextSpec {
                    text: style.frame_text.trim().to_string(),
                    cx: w * 0.5,
                    cy: b + qb + bottom * 0.5,
                    size: ts * 1.05,
                    max_w: w * 0.85,
                    color: style.frame_text_color,
                });
            }
        }
    }

    let mut layers = Vec::new();
    // Fondo
    if style.bg.mode != FillMode::Transparent {
        layers.push(Layer {
            prims: vec![rr(0.0, 0.0, w, h, style.bg_radius)],
            fill: style.bg.clone(),
            gbox: (0.0, 0.0, w, h),
        });
    }
    if !frame_prims.is_empty() {
        layers.push(Layer { prims: frame_prims, fill: Fill::solid(style.frame_color), gbox: (0.0, 0.0, w, h) });
    }

    // Zona despejada para el logo
    let logo_box = if has_logo {
        let side = (style.logo_size.clamp(0.05, 0.4) * n).max(1.0);
        Some((ox + (n - side) * 0.5, oy + (n - side) * 0.5, side))
    } else {
        None
    };
    let cleared = |x: i32, y: i32| -> bool {
        let Some((lx, ly, side)) = logo_box else { return false };
        if !style.logo_clear {
            return false;
        }
        let p = style.logo_padding.max(0.0);
        let (cx, cy) = (x as f32 + 0.5 + ox, y as f32 + 0.5 + oy);
        match style.logo_bg {
            LogoBg::Circle => {
                let r = side * 0.5 * std::f32::consts::SQRT_2.min(1.15) + p;
                let (mx, my) = (lx + side * 0.5, ly + side * 0.5);
                ((cx - mx).powi(2) + (cy - my).powi(2)).sqrt() < r
            }
            _ => cx > lx - p && cx < lx + side + p && cy > ly - p && cy < ly + side + p,
        }
    };

    let size = qr.size;
    let on = |x: i32, y: i32| -> bool { qr.get(x, y) && !qr.in_eye(x, y) && !cleared(x, y) };

    let s = style.module_scale.clamp(0.4, 1.0);
    let mut mods = Vec::new();
    for y in 0..size {
        for x in 0..size {
            if !on(x, y) {
                continue;
            }
            // Vecinos (arriba, abajo, izquierda, derecha): las formas "conectadas" los necesitan.
            let nb = [on(x, y - 1), on(x, y + 1), on(x - 1, y), on(x + 1, y)];
            module_prims(style.module_shape, s, ox + x as f32, oy + y as f32, nb, &mut mods);
        }
    }
    let qbox = (ox, oy, n, n);
    layers.push(Layer { prims: mods, fill: style.fg.clone(), gbox: qbox });

    // Ojos (los tres patrones de posición de las esquinas). El de arriba a la derecha y el de
    // abajo a la izquierda se reflejan para que las formas asimétricas "miren" hacia fuera.
    let mut frames = Vec::new();
    let mut balls = Vec::new();
    let eyes = [(0.0, 0.0, false, false), (n - 7.0, 0.0, true, false), (0.0, n - 7.0, false, true)];
    for &(ex, ey, mx, my) in &eyes {
        let (f, b) = eye_prims(style.eye_frame, style.eye_ball, ox + ex, oy + ey, mx, my);
        frames.extend(f);
        balls.extend(b);
    }
    let (ff, bf) = if style.eyes_custom {
        (Fill::solid(style.eye_frame_color), Fill::solid(style.eye_ball_color))
    } else {
        (style.fg.clone(), style.fg.clone())
    };
    layers.push(Layer { prims: frames, fill: ff, gbox: qbox });
    layers.push(Layer { prims: balls, fill: bf, gbox: qbox });

    // Fondo del logo
    if let Some((lx, ly, side)) = logo_box {
        let p = style.logo_padding.max(0.0) * 0.6;
        let prim = match style.logo_bg {
            LogoBg::None => None,
            LogoBg::Square => Some(rr(lx - p, ly - p, side + 2.0 * p, side + 2.0 * p, 0.0)),
            LogoBg::Rounded => Some(rr(lx - p, ly - p, side + 2.0 * p, side + 2.0 * p, (side + 2.0 * p) * 0.22)),
            LogoBg::Circle => Some(Prim::Circle {
                cx: lx + side * 0.5,
                cy: ly + side * 0.5,
                r: side * 0.5 * 1.15 + p,
            }),
        };
        if let Some(prim) = prim {
            layers.push(Layer { prims: vec![prim], fill: Fill::solid(style.logo_bg_color), gbox: qbox });
        }
    }

    // El logo con fondo circular se encoge un poco para quedar dentro del círculo.
    let logo_box = logo_box.map(|(lx, ly, side)| {
        if style.logo_bg == LogoBg::Circle {
            let k = 0.84;
            let ns = side * k;
            (lx + (side - ns) * 0.5, ly + (side - ns) * 0.5, ns)
        } else {
            (lx, ly, side)
        }
    });

    Scene { w, h, layers, logo_box, logo_round: style.logo_round, text, qr_origin: (ox, oy), qr_size: size }
}

// ---------------------------------------------------------------- rasterizado

/// Mezcla dos colores (t = 0 → `a`, t = 1 → `b`) y lo devuelve en 0..1.
fn lerp_color(a: Rgba, b: Rgba, t: f32) -> [f32; 4] {
    let t = t.clamp(0.0, 1.0);
    let mut o = [0.0; 4];
    for i in 0..4 {
        o[i] = (a[i] as f32 + (b[i] as f32 - a[i] as f32) * t) / 255.0;
    }
    o
}

struct Sampler {
    fill: Fill,
    cx: f32,
    cy: f32,
    dx: f32,
    dy: f32,
    ext: f32,
}

impl Sampler {
    fn new(fill: &Fill, g: (f32, f32, f32, f32)) -> Self {
        let a = fill.angle.to_radians();
        let (dx, dy) = (a.cos(), a.sin());
        let ext = match fill.mode {
            FillMode::Radial => (g.2 * g.2 + g.3 * g.3).sqrt() * 0.5,
            _ => (g.2 * dx).abs() + (g.3 * dy).abs(),
        };
        Self { fill: fill.clone(), cx: g.0 + g.2 * 0.5, cy: g.1 + g.3 * 0.5, dx, dy, ext: ext.max(1e-3) }
    }
    fn at(&self, x: f32, y: f32) -> [f32; 4] {
        match self.fill.mode {
            FillMode::Solid | FillMode::Transparent => lerp_color(self.fill.c1, self.fill.c1, 0.0),
            FillMode::Linear => {
                let t = ((x - self.cx) * self.dx + (y - self.cy) * self.dy) / self.ext + 0.5;
                lerp_color(self.fill.c1, self.fill.c2, t)
            }
            FillMode::Radial => {
                let t = ((x - self.cx).powi(2) + (y - self.cy).powi(2)).sqrt() / self.ext;
                lerp_color(self.fill.c1, self.fill.c2, t)
            }
        }
    }
}

/// Lienzo de píxeles en coma flotante donde se van mezclando las capas.
pub struct Canvas {
    pub w: usize,
    pub h: usize,
    /// Color premultiplicado.
    px: Vec<[f32; 4]>,
}

impl Canvas {
    fn blend(&mut self, i: usize, c: [f32; 4], cov: f32) {
        let a = c[3] * cov;
        if a <= 0.0 {
            return;
        }
        let d = &mut self.px[i];
        d[0] = c[0] * a + d[0] * (1.0 - a);
        d[1] = c[1] * a + d[1] * (1.0 - a);
        d[2] = c[2] * a + d[2] * (1.0 - a);
        d[3] = a + d[3] * (1.0 - a);
    }

    fn into_image(self) -> RgbaImage {
        let mut img = RgbaImage::new(self.w as u32, self.h as u32);
        for (i, p) in self.px.iter().enumerate() {
            let a = p[3].clamp(0.0, 1.0);
            let px = if a > 0.0 {
                [
                    (p[0] / a * 255.0 + 0.5).clamp(0.0, 255.0) as u8,
                    (p[1] / a * 255.0 + 0.5).clamp(0.0, 255.0) as u8,
                    (p[2] / a * 255.0 + 0.5).clamp(0.0, 255.0) as u8,
                    (a * 255.0 + 0.5) as u8,
                ]
            } else {
                [0, 0, 0, 0]
            };
            img.as_mut()[i * 4..i * 4 + 4].copy_from_slice(&px);
        }
        img
    }
}

/// Pinta una escena en una imagen de `px_width` píxeles de ancho (el alto sale de la
/// proporción de la escena).
#[allow(non_snake_case)]
pub fn rasterize(
    scene: &Scene,
    logo: Option<&RgbaImage>,
    font: &ab_glyph::FontArc,
    px_width: u32,
) -> RgbaImage {
    let scale = px_width as f32 / scene.w;
    let w = px_width.max(1) as usize;
    let h = ((scene.h * scale).round() as usize).max(1);
    let mut cv = Canvas { w, h, px: vec![[0.0; 4]; w * h] };
    let mut mask = vec![0.0f32; w * h];

    for layer in &scene.layers {
        if layer.prims.is_empty() {
            continue;
        }
        let (mut bx0, mut by0, mut bx1, mut by1) = (w, h, 0usize, 0usize);
        for p in &layer.prims {
            let b = p.bounds();
            let x0 = ((b.0 * scale).floor() as i64 - 1).clamp(0, w as i64) as usize;
            let y0 = ((b.1 * scale).floor() as i64 - 1).clamp(0, h as i64) as usize;
            let x1 = ((b.2 * scale).ceil() as i64 + 1).clamp(0, w as i64) as usize;
            let y1 = ((b.3 * scale).ceil() as i64 + 1).clamp(0, h as i64) as usize;
            if x0 >= x1 || y0 >= y1 {
                continue;
            }
            bx0 = bx0.min(x0);
            by0 = by0.min(y0);
            bx1 = bx1.max(x1);
            by1 = by1.max(y1);
            for yy in y0..y1 {
                let uy = (yy as f32 + 0.5) / scale;
                let row = yy * w;
                for xx in x0..x1 {
                    let ux = (xx as f32 + 0.5) / scale;
                    let d = p.sd(ux, uy) * scale;
                    let c = (0.5 - d).clamp(0.0, 1.0);
                    if c > 0.0 {
                        let m = &mut mask[row + xx];
                        *m = (*m + c).min(1.0);
                    }
                }
            }
        }
        if bx0 >= bx1 || by0 >= by1 {
            continue;
        }
        let sampler = Sampler::new(&layer.fill, layer.gbox);
        for yy in by0..by1 {
            let uy = (yy as f32 + 0.5) / scale;
            for xx in bx0..bx1 {
                let i = yy * w + xx;
                let m = mask[i];
                if m > 0.0 {
                    let c = sampler.at((xx as f32 + 0.5) / scale, uy);
                    cv.blend(i, c, m);
                    mask[i] = 0.0;
                }
            }
        }
    }

    // Logo
    if let (Some(img), Some((lx, ly, side))) = (logo, scene.logo_box) {
        let box_px = (side * scale).max(1.0);
        let (iw, ih) = (img.width() as f32, img.height() as f32);
        let k = box_px / iw.max(ih);
        let tw = ((iw * k).round() as u32).max(1);
        let th = ((ih * k).round() as u32).max(1);
        let mut resized = image::imageops::resize(img, tw, th, image::imageops::FilterType::Triangle);
        if scene.logo_round {
            resized = round_mask(&resized);
        }
        let x0 = (lx * scale + (box_px - tw as f32) * 0.5).round() as i64;
        let y0 = (ly * scale + (box_px - th as f32) * 0.5).round() as i64;
        for (x, y, p) in resized.enumerate_pixels() {
            let (X, Y) = (x0 + x as i64, y0 + y as i64);
            if X < 0 || Y < 0 || X >= w as i64 || Y >= h as i64 {
                continue;
            }
            let c = [p[0] as f32 / 255.0, p[1] as f32 / 255.0, p[2] as f32 / 255.0, p[3] as f32 / 255.0];
            cv.blend(Y as usize * w + X as usize, c, 1.0);
        }
    }

    // Texto del marco
    if let Some(t) = &scene.text {
        draw_text(&mut cv, font, t, scale);
    }

    cv.into_image()
}

#[allow(non_snake_case)]
/// Escribe el texto del marco centrado (se encoge si no cabe).
fn draw_text(cv: &mut Canvas, font: &ab_glyph::FontArc, t: &TextSpec, scale: f32) {
    use ab_glyph::{Font, Glyph, PxScale, ScaleFont};
    let measure = |px: f32| -> f32 {
        let sf = font.as_scaled(PxScale::from(px));
        let mut w = 0.0;
        let mut prev = None;
        for c in t.text.chars() {
            let id = sf.glyph_id(c);
            if let Some(p) = prev {
                w += sf.kern(p, id);
            }
            w += sf.h_advance(id);
            prev = Some(id);
        }
        w
    };
    let mut px = t.size * scale * 1.12;
    let maxw = t.max_w * scale;
    let tw = measure(px);
    if tw > maxw && tw > 0.0 {
        px *= maxw / tw;
    }
    let tw = measure(px);
    let sf = font.as_scaled(PxScale::from(px));
    let cap = font
        .outline_glyph(sf.scaled_glyph('H'))
        .map(|g| g.px_bounds().height())
        .unwrap_or(px * 0.7);
    let baseline = t.cy * scale + cap * 0.5;
    let mut x = t.cx * scale - tw * 0.5;
    let col = [t.color[0] as f32 / 255.0, t.color[1] as f32 / 255.0, t.color[2] as f32 / 255.0, t.color[3] as f32 / 255.0];
    let mut prev = None;
    for c in t.text.chars() {
        let id = sf.glyph_id(c);
        if let Some(p) = prev {
            x += sf.kern(p, id);
        }
        let g: Glyph = id.with_scale_and_position(PxScale::from(px), ab_glyph::point(x, baseline));
        if let Some(og) = font.outline_glyph(g) {
            let b = og.px_bounds();
            og.draw(|gx, gy, cov| {
                let X = b.min.x as i64 + gx as i64;
                let Y = b.min.y as i64 + gy as i64;
                if X >= 0 && Y >= 0 && (X as usize) < cv.w && (Y as usize) < cv.h {
                    cv.blend(Y as usize * cv.w + X as usize, col, cov.min(1.0));
                }
            });
        }
        x += sf.h_advance(id);
        prev = Some(id);
    }
}

// ---------------------------------------------------------------- SVG

/// Número corto para el SVG (sin ceros de más).
fn f(v: f32) -> String {
    let s = format!("{:.3}", v);
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" { "0".into() } else { s.to_string() }
}

fn hex(c: Rgba) -> String {
    format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2])
}

/// Añade la figura al trazado SVG (`d="..."`).
fn path_of(p: &Prim, out: &mut String) {
    match p {
        Prim::RRect { x, y, w, h, r } => {
            let lim = (w * 0.5).min(h * 0.5);
            let r: Vec<f32> = r.iter().map(|v| v.clamp(0.0, lim)).collect();
            let (x, y, w, h) = (*x, *y, *w, *h);
            out.push_str(&format!("M{} {}", f(x + r[0]), f(y)));
            out.push_str(&format!("H{}", f(x + w - r[1])));
            if r[1] > 0.0 {
                out.push_str(&format!("A{0} {0} 0 0 1 {1} {2}", f(r[1]), f(x + w), f(y + r[1])));
            }
            out.push_str(&format!("V{}", f(y + h - r[2])));
            if r[2] > 0.0 {
                out.push_str(&format!("A{0} {0} 0 0 1 {1} {2}", f(r[2]), f(x + w - r[2]), f(y + h)));
            }
            out.push_str(&format!("H{}", f(x + r[3])));
            if r[3] > 0.0 {
                out.push_str(&format!("A{0} {0} 0 0 1 {1} {2}", f(r[3]), f(x), f(y + h - r[3])));
            }
            out.push_str(&format!("V{}", f(y + r[0])));
            if r[0] > 0.0 {
                out.push_str(&format!("A{0} {0} 0 0 1 {1} {2}", f(r[0]), f(x + r[0]), f(y)));
            }
            out.push('Z');
        }
        Prim::Circle { r, .. } if *r <= 0.0 => {}
        Prim::Circle { cx, cy, r } => {
            out.push_str(&format!(
                "M{} {}A{2} {2} 0 1 0 {3} {1}A{2} {2} 0 1 0 {0} {1}Z",
                f(cx - r),
                f(*cy),
                f(*r),
                f(cx + r)
            ));
        }
        Prim::Poly(v) => {
            for (i, (x, y)) in v.iter().enumerate() {
                out.push_str(&format!("{}{} {}", if i == 0 { "M" } else { "L" }, f(*x), f(*y)));
            }
            out.push('Z');
        }
        Prim::Ring(o, i) => {
            path_of(o, out);
            path_of(i, out);
        }
    }
}

fn xml_esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// Codificación base64 (para incrustar el logo dentro del SVG).
pub fn base64(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len() * 4 / 3 + 4);
    for ch in data.chunks(3) {
        let b = [ch[0], *ch.get(1).unwrap_or(&0), *ch.get(2).unwrap_or(&0)];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        out.push(T[(n >> 18) as usize & 63] as char);
        out.push(T[(n >> 12) as usize & 63] as char);
        out.push(if ch.len() > 1 { T[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if ch.len() > 2 { T[n as usize & 63] as char } else { '=' });
    }
    out
}

/// Escribe la escena como un archivo SVG con degradados y el logo incrustado.
pub fn to_svg(scene: &Scene, logo: Option<&RgbaImage>, px_width: u32) -> String {
    let px_h = (scene.h / scene.w * px_width as f32).round();
    let mut s = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {} {}\" width=\"{}\" height=\"{}\" shape-rendering=\"geometricPrecision\">\n",
        f(scene.w),
        f(scene.h),
        px_width,
        px_h
    );
    let mut defs = String::new();
    let mut body = String::new();
    for (i, layer) in scene.layers.iter().enumerate() {
        if layer.prims.is_empty() {
            continue;
        }
        let fl = &layer.fill;
        let fill_attr = match fl.mode {
            FillMode::Solid | FillMode::Transparent => {
                let mut a = format!("fill=\"{}\"", hex(fl.c1));
                if fl.c1[3] < 255 {
                    a.push_str(&format!(" fill-opacity=\"{}\"", f(fl.c1[3] as f32 / 255.0)));
                }
                a
            }
            FillMode::Linear | FillMode::Radial => {
                let g = layer.gbox;
                let (cx, cy) = (g.0 + g.2 * 0.5, g.1 + g.3 * 0.5);
                let stops = format!(
                    "<stop offset=\"0\" stop-color=\"{}\" stop-opacity=\"{}\"/><stop offset=\"1\" stop-color=\"{}\" stop-opacity=\"{}\"/>",
                    hex(fl.c1),
                    f(fl.c1[3] as f32 / 255.0),
                    hex(fl.c2),
                    f(fl.c2[3] as f32 / 255.0)
                );
                if fl.mode == FillMode::Linear {
                    let a = fl.angle.to_radians();
                    let (dx, dy) = (a.cos(), a.sin());
                    let ext = (g.2 * dx).abs() + (g.3 * dy).abs();
                    defs.push_str(&format!(
                        "<linearGradient id=\"g{i}\" gradientUnits=\"userSpaceOnUse\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\">{stops}</linearGradient>\n",
                        f(cx - dx * ext * 0.5),
                        f(cy - dy * ext * 0.5),
                        f(cx + dx * ext * 0.5),
                        f(cy + dy * ext * 0.5)
                    ));
                } else {
                    let r = (g.2 * g.2 + g.3 * g.3).sqrt() * 0.5;
                    defs.push_str(&format!(
                        "<radialGradient id=\"g{i}\" gradientUnits=\"userSpaceOnUse\" cx=\"{}\" cy=\"{}\" r=\"{}\">{stops}</radialGradient>\n",
                        f(cx),
                        f(cy),
                        f(r)
                    ));
                }
                format!("fill=\"url(#g{i})\"")
            }
        };
        let mut plain = String::new();
        for p in &layer.prims {
            if let Prim::Ring(..) = p {
                let mut d = String::new();
                path_of(p, &mut d);
                body.push_str(&format!("<path fill-rule=\"evenodd\" {fill_attr} d=\"{d}\"/>\n"));
            } else {
                path_of(p, &mut plain);
            }
        }
        if !plain.is_empty() {
            body.push_str(&format!("<path {fill_attr} d=\"{plain}\"/>\n"));
        }
    }
    if let (Some(img), Some((lx, ly, side))) = (logo, scene.logo_box) {
        let mut png = Vec::new();
        let img = if scene.logo_round { round_mask(img) } else { img.clone() };
        if image::DynamicImage::ImageRgba8(img)
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .is_ok()
        {
            body.push_str(&format!(
                "<image x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" preserveAspectRatio=\"xMidYMid meet\" href=\"data:image/png;base64,{}\"/>\n",
                f(lx),
                f(ly),
                f(side),
                f(side),
                base64(&png)
            ));
        }
    }
    if let Some(t) = &scene.text {
        body.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" font-family=\"Segoe UI, Helvetica, Arial, sans-serif\" font-weight=\"700\" font-size=\"{}\" text-anchor=\"middle\" dominant-baseline=\"central\" fill=\"{}\" textLength=\"{}\" lengthAdjust=\"spacingAndGlyphs\">{}</text>\n",
            f(t.cx),
            f(t.cy),
            f(t.size * 1.12),
            hex(t.color),
            f((t.text.chars().count() as f32 * t.size * 0.78).min(t.max_w)),
            xml_esc(&t.text)
        ));
    }
    if !defs.is_empty() {
        s.push_str("<defs>\n");
        s.push_str(&defs);
        s.push_str("</defs>\n");
    }
    s.push_str(&body);
    s.push_str("</svg>\n");
    s
}

/// Recorta una imagen en círculo (lo de fuera queda transparente, con borde suave).
pub fn round_mask(img: &RgbaImage) -> RgbaImage {
    let mut out = img.clone();
    let (w, h) = (img.width() as f32, img.height() as f32);
    let r = w.min(h) * 0.5;
    for (x, y, p) in out.enumerate_pixels_mut() {
        let d = ((x as f32 + 0.5 - w * 0.5).powi(2) + (y as f32 + 0.5 - h * 0.5).powi(2)).sqrt();
        let k = (r - d + 0.5).clamp(0.0, 1.0);
        p[3] = (p[3] as f32 * k) as u8;
    }
    out
}

/// Iconos listos para poner en el centro del QR (útiles cuando no se tiene un logo).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LogoIcon {
    Star,
    Heart,
    Wifi,
    Pin,
    Mail,
    Phone,
    Play,
    Chat,
    Music,
}

impl LogoIcon {
    pub const ALL: [LogoIcon; 9] = [
        LogoIcon::Wifi,
        LogoIcon::Pin,
        LogoIcon::Mail,
        LogoIcon::Phone,
        LogoIcon::Chat,
        LogoIcon::Play,
        LogoIcon::Music,
        LogoIcon::Heart,
        LogoIcon::Star,
    ];
    /// Nombre en español (la interfaz lo traduce).
    pub fn label(self) -> &'static str {
        match self {
            LogoIcon::Star => "Estrella",
            LogoIcon::Heart => "Corazón",
            LogoIcon::Wifi => "Wi-Fi",
            LogoIcon::Pin => "Ubicación",
            LogoIcon::Mail => "Correo",
            LogoIcon::Phone => "Teléfono",
            LogoIcon::Play => "Vídeo",
            LogoIcon::Chat => "Chat",
            LogoIcon::Music => "Música",
        }
    }
}

/// Un trozo de anillo entre los ángulos `a0` y `a1` (en grados; 0 = derecha, -90 = arriba).
fn arc(cx: f32, cy: f32, r0: f32, r1: f32, a0: f32, a1: f32) -> Prim {
    let steps = 24;
    let mut v = Vec::with_capacity(steps * 2 + 2);
    for k in 0..=steps {
        let a = (a0 + (a1 - a0) * k as f32 / steps as f32).to_radians();
        v.push((cx + r1 * a.cos(), cy + r1 * a.sin()));
    }
    for k in (0..=steps).rev() {
        let a = (a0 + (a1 - a0) * k as f32 / steps as f32).to_radians();
        v.push((cx + r0 * a.cos(), cy + r0 * a.sin()));
    }
    Prim::Poly(v)
}

/// Las figuras blancas de un icono, en un cuadro de 24×24 unidades.
fn icon_glyph(icon: LogoIcon) -> Vec<Prim> {
    match icon {
        LogoIcon::Star => vec![star(12.0, 12.6, 7.4, 3.1)],
        LogoIcon::Heart => vec![heart(12.0, 12.6, 13.5)],
        LogoIcon::Wifi => vec![
            Prim::Circle { cx: 12.0, cy: 16.6, r: 1.6 },
            arc(12.0, 16.6, 3.4, 5.2, -138.0, -42.0),
            arc(12.0, 16.6, 6.6, 8.4, -138.0, -42.0),
        ],
        LogoIcon::Pin => vec![
            Prim::Ring(
                Box::new(Prim::Circle { cx: 12.0, cy: 10.0, r: 5.4 }),
                Box::new(Prim::Circle { cx: 12.0, cy: 10.0, r: 2.2 }),
            ),
            Prim::Poly(vec![(7.4, 12.8), (16.6, 12.8), (12.0, 20.0)]),
        ],
        LogoIcon::Mail => vec![
            Prim::Ring(Box::new(rr(4.8, 7.2, 14.4, 10.0, 1.3)), Box::new(rr(6.2, 8.6, 11.6, 7.2, 0.4))),
            Prim::Poly(vec![(6.0, 8.4), (7.6, 8.4), (12.0, 11.9), (16.4, 8.4), (18.0, 8.4), (12.0, 13.6)]),
        ],
        LogoIcon::Phone => vec![
            Prim::Ring(Box::new(rr(8.0, 4.4, 8.0, 15.2, 1.7)), Box::new(rr(9.3, 6.4, 5.4, 10.0, 0.4))),
        ],
        LogoIcon::Play => vec![Prim::Poly(vec![(9.6, 7.2), (17.6, 12.0), (9.6, 16.8)])],
        LogoIcon::Chat => vec![
            rr(5.2, 6.2, 13.6, 9.8, 3.2),
            Prim::Poly(vec![(8.2, 15.0), (12.0, 15.0), (7.6, 19.4)]),
        ],
        LogoIcon::Music => vec![
            Prim::Circle { cx: 9.6, cy: 16.4, r: 2.5 },
            rr(11.2, 6.2, 1.5, 10.4, 0.2),
            Prim::Poly(vec![(12.5, 6.2), (17.2, 8.2), (17.2, 10.8), (12.5, 9.0)]),
        ],
    }
}

/// Un icono listo para el centro: figura blanca sobre un círculo del color dado (512 px).
pub fn builtin_logo(icon: LogoIcon, color: Rgba) -> RgbaImage {
    let w = 24.0;
    let scene = Scene {
        w,
        h: w,
        layers: vec![
            Layer { prims: vec![Prim::Circle { cx: 12.0, cy: 12.0, r: 12.0 }], fill: Fill::solid(color), gbox: (0.0, 0.0, w, w) },
            Layer { prims: icon_glyph(icon), fill: Fill::solid([255, 255, 255, 255]), gbox: (0.0, 0.0, w, w) },
        ],
        logo_box: None,
        logo_round: false,
        text: None,
        qr_origin: (0.0, 0.0),
        qr_size: 1,
    };
    let font = ab_glyph::FontArc::try_from_slice(epaint_default_fonts::UBUNTU_LIGHT).expect("fuente interna");
    rasterize(&scene, None, &font, 512)
}

/// Icono blanco (sin círculo) para los botones de la interfaz.
pub fn logo_icon_preview(icon: LogoIcon, font: &ab_glyph::FontArc, px: u32) -> RgbaImage {
    icon_from(icon_glyph(icon), 24.0, font, px)
}

/// Un logo de ejemplo (estrella blanca sobre un círculo con degradado) para probar la función.
pub fn sample_logo() -> RgbaImage {
    let n = 256u32;
    let k = n as f32;
    let circle = Prim::Circle { cx: 0.5, cy: 0.5, r: 0.5 };
    let st = star(0.5, 0.53, 0.36, 0.15);
    let mut img = RgbaImage::new(n, n);
    for (x, y, p) in img.enumerate_pixels_mut() {
        let (u, v) = ((x as f32 + 0.5) / k, (y as f32 + 0.5) / k);
        let ca = (0.5 - circle.sd(u, v) * k).clamp(0.0, 1.0);
        let sa = (0.5 - st.sd(u, v) * k).clamp(0.0, 1.0);
        let t = (u + v) * 0.5;
        let c = lerp_color([255, 140, 40, 255], [230, 40, 120, 255], t);
        let r = c[0] * (1.0 - sa) + sa;
        let g = c[1] * (1.0 - sa) + sa;
        let b = c[2] * (1.0 - sa) + sa;
        *p = image::Rgba([(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, (ca * 255.0) as u8]);
    }
    img
}

/// Cómo se comprobó que el QR se puede leer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ReadKind {
    /// Encontrado y leído en la imagen completa, como una cámara.
    Camera,
    /// Los módulos se distinguen bien y la corrección de errores recupera el contenido.
    Modules,
}

/// Por qué no se pudo leer el QR dibujado.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ReadError {
    LowContrast,
    Mismatch,
    Unreadable,
}

impl ReadError {
    /// Explicación para la persona, en el idioma elegido.
    pub fn message(self) -> &'static str {
        use crate::i18n::tr;
        match self {
            ReadError::LowContrast => tr("Contraste insuficiente entre el código y el fondo."),
            ReadError::Mismatch => tr("Se lee, pero el contenido no coincide."),
            ReadError::Unreadable => tr("No se pudo leer. Prueba más contraste, un logo más pequeño o más corrección."),
        }
    }
}

/// Intenta leer el QR dibujado, como haría un teléfono.
pub fn verify(img: &RgbaImage, scene: &Scene, expected: &str) -> Result<ReadKind, ReadError> {
    let (w, h) = (img.width() as usize, img.height() as usize);
    let gray: Vec<f32> = img
        .pixels()
        .map(|p| {
            let a = p[3] as f32 / 255.0;
            let l = 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32;
            l * a + 255.0 * (1.0 - a)
        })
        .collect();
    let mut wrong = false;
    for invert in [false, true] {
        let mut prep = rqrr::PreparedImage::prepare_from_greyscale(w, h, |x, y| {
            let v = gray[y * w + x] as u8;
            if invert { 255 - v } else { v }
        });
        for g in prep.detect_grids() {
            if let Ok((_, content)) = g.decode() {
                if content == expected {
                    return Ok(ReadKind::Camera);
                }
                wrong = true;
            }
        }
    }
    // Segunda prueba: mirar el centro de cada módulo. Los ojos decorados confunden al detector
    // sencillo que usamos, pero no a las cámaras de los móviles actuales.
    let n = scene.qr_size.max(1) as usize;
    let scale = w as f32 / scene.w;
    let (ox, oy) = scene.qr_origin;
    let r = (scale * 0.12) as i64;
    let mut samples = vec![0.0f32; n * n];
    for y in 0..n {
        for x in 0..n {
            let cx = ((ox + x as f32 + 0.5) * scale) as i64;
            let cy = ((oy + y as f32 + 0.5) * scale) as i64;
            let (mut sum, mut cnt) = (0.0, 0.0);
            for yy in cy - r..=cy + r {
                for xx in cx - r..=cx + r {
                    if xx >= 0 && yy >= 0 && (xx as usize) < w && (yy as usize) < h {
                        sum += gray[yy as usize * w + xx as usize];
                        cnt += 1.0;
                    }
                }
            }
            samples[y * n + x] = if cnt > 0.0 { sum / cnt } else { 255.0 };
        }
    }
    // Umbral por dos grupos (oscuros y claros), sin usar el resultado esperado.
    let mut lo = samples.iter().cloned().fold(f32::MAX, f32::min);
    let mut hi = samples.iter().cloned().fold(f32::MIN, f32::max);
    for _ in 0..12 {
        let t = (lo + hi) * 0.5;
        let (mut a, mut na, mut b, mut nb) = (0.0, 0.0, 0.0, 0.0);
        for &v in &samples {
            if v < t {
                a += v;
                na += 1.0;
            } else {
                b += v;
                nb += 1.0;
            }
        }
        if na > 0.0 {
            lo = a / na;
        }
        if nb > 0.0 {
            hi = b / nb;
        }
    }
    if hi - lo < 40.0 {
        return Err(ReadError::LowContrast);
    }
    let t = (lo + hi) * 0.5;
    // El centro de un ojo siempre es un módulo "oscuro" del código (las esquinas pueden ir redondeadas).
    // Se vota con el centro y los lados del marco de los tres ojos.
    let mut votes = 0i32;
    for (ex, ey) in [(0usize, 0usize), (n - 7, 0), (0, n - 7)] {
        for (dx, dy) in [(3usize, 3usize), (3, 0), (0, 3), (6, 3), (3, 6)] {
            votes += if samples[(ey + dy) * n + ex + dx] < t { 1 } else { -1 };
        }
    }
    let dark_is_low = votes >= 0;
    let grid = rqrr::SimpleGrid::from_func(n, |x, y| (samples[y * n + x] < t) == dark_is_low);
    if let Ok((_, content)) = rqrr::Grid::new(grid).decode() {
        if content == expected {
            return Ok(ReadKind::Modules);
        }
        wrong = true;
    }
    if wrong {
        return Err(ReadError::Mismatch);
    }
    Err(ReadError::Unreadable)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shapes_are_readable() {
        let font = ab_glyph::FontArc::try_from_slice(epaint_default_fonts::UBUNTU_LIGHT).unwrap();
        let text = "https://www.ejemplo.com";
        let rules = Rules { ecc: Ecc::Q, ..Rules::default() };
        let q = crate::qr::encode(text, &rules).unwrap();
        let _ = std::fs::create_dir_all("shots");
        for m in ModuleShape::ALL {
            let style = Style { module_shape: m, ..Style::default() };
            let scene = build_scene(&q, &style, &rules, false);
            let img = rasterize(&scene, None, &font, 720);
            let r = verify(&img, &scene, text);
            println!("{:?}: {:?}", m, r);
            assert!(r.is_ok(), "{:?} no se lee", m);
        }
        // Cada marco y centro de ojo, y cada tipo de marco con texto.
        for e in EyeFrame::ALL {
            let style = Style { eye_frame: e, ..Style::default() };
            let scene = build_scene(&q, &style, &rules, false);
            let img = rasterize(&scene, None, &font, 720);
            assert!(verify(&img, &scene, text).is_ok(), "{:?} no se lee", e);
        }
        for b in EyeBall::ALL {
            let style = Style { eye_ball: b, ..Style::default() };
            let scene = build_scene(&q, &style, &rules, false);
            let img = rasterize(&scene, None, &font, 720);
            assert!(verify(&img, &scene, text).is_ok(), "{:?} no se lee", b);
        }
        for f in FrameStyle::ALL {
            let style = Style { frame: f, frame_color: [20, 20, 60, 255], ..Style::default() };
            let scene = build_scene(&q, &style, &rules, false);
            let img = rasterize(&scene, None, &font, 720);
            assert!(verify(&img, &scene, text).is_ok(), "{:?} no se lee", f);
            img.save(format!("shots/frame_{:?}.png", f)).unwrap();
        }
        // Los iconos listos, recortados en círculo, también dejan leer el código.
        let hrules = Rules { ecc: Ecc::H, ..Rules::default() };
        let hq = crate::qr::encode(text, &hrules).unwrap();
        for icon in LogoIcon::ALL {
            let logo = builtin_logo(icon, [99, 91, 255, 255]);
            let style = Style { logo_round: true, ..Style::default() };
            let scene = build_scene(&hq, &style, &hrules, true);
            let img = rasterize(&scene, Some(&logo), &font, 720);
            assert!(verify(&img, &scene, text).is_ok(), "{:?} no se lee", icon);
            img.save(format!("shots/icon_{:?}.png", icon)).unwrap();
        }
        // Diseños al azar.
        let mut fails = 0;
        for seed in 0..40u64 {
            let style = random_style(seed);
            let scene = build_scene(&q, &style, &rules, false);
            let img = rasterize(&scene, None, &font, 480);
            if verify(&img, &scene, text).is_err() {
                fails += 1;
            }
        }
        println!("al azar ilegibles: {fails}/40");
        let logo = sample_logo();
        let hq = crate::qr::encode(text, &hrules).unwrap();
        for (i, p) in PRESETS.iter().enumerate() {
            let style = (p.style)();
            let scene = build_scene(&q, &style, &rules, false);
            let img = rasterize(&scene, None, &font, 720);
            let r = verify(&img, &scene, text);
            println!("preset {}: {:?}", p.name, r);
            img.save(format!("shots/plain_{i}.png")).unwrap();
            assert!(r.is_ok(), "{} no se lee", p.name);
            let scene = build_scene(&hq, &style, &hrules, true);
            let img = rasterize(&scene, Some(&logo), &font, 720);
            let r = verify(&img, &scene, text);
            println!("preset {} + logo: {:?}", p.name, r);
            assert!(r.is_ok(), "{} con logo no se lee", p.name);
            img.save(format!("shots/preset_{i}.png")).unwrap();
            let svg = to_svg(&scene, Some(&logo), 512);
            assert!(svg.starts_with("<svg") && svg.trim_end().ends_with("</svg>"));
        }
    }
}
