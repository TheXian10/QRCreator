//! Codificación del QR siguiendo la norma ISO/IEC 18004 con todas sus reglas:
//! nivel de corrección de errores, versión (tamaño), patrón de máscara, modo de
//! codificación, ECI y cálculo de la capacidad usada.
//!
//! El trabajo pesado (Reed-Solomon, colocación de módulos, elección de máscara) lo hace el
//! crate `qrcodegen`; aquí se preparan los segmentos y se calcula la información que se
//! enseña en la interfaz.

use crate::style::{Ecc, EncMode, Rules};
use qrcodegen::{Mask, QrCode, QrCodeEcc, QrSegment, QrSegmentMode, Version};

#[derive(Clone)]
pub struct QrResult {
    pub size: i32,
    pub modules: Vec<bool>,
    pub version: u8,
    pub ecc: Ecc,
    pub mask: u8,
    pub used_bits: usize,
    pub capacity_bits: usize,
    pub mode_desc: String,
    pub bytes: usize,
}

impl QrResult {
    pub fn get(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= self.size || y >= self.size {
            return false;
        }
        self.modules[(y * self.size + x) as usize]
    }

    /// ¿Este módulo forma parte de uno de los tres "ojos" (patrones de posición)?
    pub fn in_eye(&self, x: i32, y: i32) -> bool {
        let n = self.size;
        (x < 7 && y < 7) || (x >= n - 7 && y < 7) || (x < 7 && y >= n - 7)
    }
}

fn to_ecc(e: Ecc) -> QrCodeEcc {
    match e {
        Ecc::L => QrCodeEcc::Low,
        Ecc::M => QrCodeEcc::Medium,
        Ecc::Q => QrCodeEcc::Quartile,
        Ecc::H => QrCodeEcc::High,
    }
}

fn from_ecc(e: QrCodeEcc) -> Ecc {
    match e {
        QrCodeEcc::Low => Ecc::L,
        QrCodeEcc::Medium => Ecc::M,
        QrCodeEcc::Quartile => Ecc::Q,
        QrCodeEcc::High => Ecc::H,
    }
}

static ECC_CODEWORDS_PER_BLOCK: [[i16; 41]; 4] = [
    [-1, 7, 10, 15, 20, 26, 18, 20, 24, 30, 18, 20, 24, 26, 30, 22, 24, 28, 30, 28, 28, 28, 28, 30, 30, 26, 28, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30],
    [-1, 10, 16, 26, 18, 24, 16, 18, 22, 22, 26, 30, 22, 22, 24, 24, 28, 28, 26, 26, 26, 26, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28],
    [-1, 13, 22, 18, 26, 18, 24, 18, 22, 20, 24, 28, 26, 24, 20, 30, 24, 28, 28, 26, 30, 28, 30, 30, 30, 30, 28, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30],
    [-1, 17, 28, 22, 16, 22, 28, 26, 26, 24, 28, 24, 28, 22, 24, 24, 30, 28, 28, 26, 28, 30, 24, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30],
];

static NUM_ERROR_CORRECTION_BLOCKS: [[i16; 41]; 4] = [
    [-1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 4, 4, 4, 4, 4, 6, 6, 6, 6, 7, 8, 8, 9, 9, 10, 12, 12, 12, 13, 14, 15, 16, 17, 18, 19, 19, 20, 21, 22, 24, 25],
    [-1, 1, 1, 1, 2, 2, 4, 4, 4, 5, 5, 5, 8, 9, 9, 10, 10, 11, 13, 14, 16, 17, 17, 18, 20, 21, 23, 25, 26, 28, 29, 31, 33, 35, 37, 38, 40, 43, 45, 47, 49],
    [-1, 1, 1, 2, 2, 4, 4, 6, 6, 8, 8, 8, 10, 12, 16, 12, 17, 16, 18, 21, 20, 23, 23, 25, 27, 29, 34, 34, 35, 38, 40, 43, 45, 48, 51, 53, 56, 59, 62, 65, 68],
    [-1, 1, 1, 2, 4, 4, 4, 5, 6, 8, 8, 11, 11, 16, 16, 18, 16, 19, 21, 25, 25, 25, 34, 30, 32, 35, 37, 40, 42, 45, 48, 51, 54, 57, 60, 63, 66, 70, 74, 77, 81],
];

fn raw_data_modules(ver: usize) -> usize {
    let mut result = (16 * ver + 128) * ver + 64;
    if ver >= 2 {
        let numalign = ver / 7 + 2;
        result -= (25 * numalign - 10) * numalign - 55;
        if ver >= 7 {
            result -= 36;
        }
    }
    result
}

pub fn data_codewords(ver: u8, ecc: Ecc) -> usize {
    let e = ecc as usize;
    let v = ver as usize;
    raw_data_modules(v) / 8
        - ECC_CODEWORDS_PER_BLOCK[e][v] as usize * NUM_ERROR_CORRECTION_BLOCKS[e][v] as usize
}

fn char_count_bits(mode: QrSegmentMode, ver: u8) -> usize {
    let i = ((ver + 7) / 17) as usize;
    (match mode {
        QrSegmentMode::Numeric => [10, 12, 14],
        QrSegmentMode::Alphanumeric => [9, 11, 13],
        QrSegmentMode::Byte => [8, 16, 16],
        QrSegmentMode::Kanji => [8, 10, 12],
        QrSegmentMode::Eci => [0, 0, 0],
    })[i]
}

fn total_bits(segs: &[QrSegment], ver: u8) -> usize {
    segs.iter()
        .map(|s| 4 + char_count_bits(s.mode(), ver) + s.data().len())
        .sum()
}

fn mode_name(m: QrSegmentMode) -> &'static str {
    match m {
        QrSegmentMode::Numeric => "Numérico",
        QrSegmentMode::Alphanumeric => "Alfanumérico",
        QrSegmentMode::Byte => "Bytes",
        QrSegmentMode::Kanji => "Kanji",
        QrSegmentMode::Eci => "ECI",
    }
}

/// Por qué no se pudo crear el QR.
#[derive(Clone, Debug, PartialEq)]
pub enum QrError {
    /// No hay contenido.
    Empty,
    /// Se eligió el modo numérico y el texto tiene algo que no es un dígito.
    NotNumeric,
    /// Se eligió el modo alfanumérico y el texto tiene caracteres no admitidos.
    NotAlphanumeric,
    /// El contenido supera lo que admite cualquier código QR.
    TooLong,
    /// No cabe en la versión máxima elegida: bits necesarios, bits disponibles, versión máxima.
    OverCapacity(usize, usize, u8, Ecc),
}

impl QrError {
    /// Explicación para la persona, en el idioma elegido.
    pub fn message(&self) -> String {
        use crate::i18n::{tr, trf};
        match self {
            QrError::Empty => tr("Escribe algo en el contenido para crear el QR.").into(),
            QrError::NotNumeric => tr("El modo numérico solo admite los dígitos 0-9. Cambia el modo a «Automático» o «Bytes».").into(),
            QrError::NotAlphanumeric => tr("El modo alfanumérico solo admite 0-9, A-Z en MAYÚSCULAS, espacio y $ % * + - . / :. Cambia el modo o el texto.").into(),
            QrError::TooLong => tr("El contenido es demasiado largo para un código QR.").into(),
            QrError::OverCapacity(used, cap, hi, ecc) => trf(
                "No cabe: el contenido necesita {0} bits y la versión máxima permitida ({1}) con corrección {2} admite {3}. Sube la versión máxima, baja la corrección o acorta el texto.",
                &[used, hi, &ecc.short(), cap],
            ),
        }
    }
}

/// Convierte el texto en un código QR siguiendo las reglas elegidas.
pub fn encode(text: &str, rules: &Rules) -> Result<QrResult, QrError> {
    if text.is_empty() {
        return Err(QrError::Empty);
    }
    let mut segs: Vec<QrSegment> = Vec::new();
    if rules.eci_utf8 {
        segs.push(QrSegment::make_eci(26));
    }
    match rules.mode {
        EncMode::Auto => segs.extend(QrSegment::make_segments(text)),
        EncMode::Numeric => {
            if !QrSegment::is_numeric(text) {
                return Err(QrError::NotNumeric);
            }
            segs.push(QrSegment::make_numeric(text));
        }
        EncMode::Alphanumeric => {
            if !QrSegment::is_alphanumeric(text) {
                return Err(QrError::NotAlphanumeric);
            }
            segs.push(QrSegment::make_alphanumeric(text));
        }
        EncMode::Byte => segs.push(QrSegment::make_bytes(text.as_bytes())),
    }
    let (lo, hi) = {
        let a = rules.min_version.clamp(1, 40);
        let b = rules.max_version.clamp(1, 40);
        if a <= b { (a, b) } else { (b, a) }
    };
    let mask = rules.mask.map(|m| Mask::new(m.min(7)));
    let qr = QrCode::encode_segments_advanced(
        &segs,
        to_ecc(rules.ecc),
        Version::new(lo),
        Version::new(hi),
        mask,
        rules.boost_ecc,
    )
    .map_err(|e| match e {
        qrcodegen::DataTooLong::SegmentTooLong => QrError::TooLong,
        qrcodegen::DataTooLong::DataOverCapacity(used, cap) => QrError::OverCapacity(used, cap, hi, rules.ecc),
    })?;
    let size = qr.size();
    let mut modules = Vec::with_capacity((size * size) as usize);
    for y in 0..size {
        for x in 0..size {
            modules.push(qr.get_module(x, y));
        }
    }
    let version = qr.version().value();
    let ecc = from_ecc(qr.error_correction_level());
    let mut names: Vec<&str> = Vec::new();
    for s in &segs {
        let n = mode_name(s.mode());
        if !names.contains(&n) {
            names.push(n);
        }
    }
    Ok(QrResult {
        size,
        modules,
        version,
        ecc,
        mask: qr.mask().value(),
        used_bits: total_bits(&segs, version),
        capacity_bits: data_codewords(version, ecc) * 8,
        mode_desc: names.join(" + "),
        bytes: text.len(),
    })
}
