//! Plantillas de contenido: convierten unos campos sencillos en el texto que lleva el QR,
//! con el formato que entienden las cámaras de los móviles:
//!
//! | Tipo       | Formato                                   |
//! |------------|-------------------------------------------|
//! | Enlace     | `https://...`                             |
//! | Wi-Fi      | `WIFI:T:WPA;S:red;P:clave;;`              |
//! | Contacto   | vCard 3.0 (`BEGIN:VCARD ... END:VCARD`)   |
//! | Correo     | `mailto:dir?subject=...&body=...`         |
//! | Teléfono   | `tel:+34...`                              |
//! | SMS        | `SMSTO:numero:mensaje`                    |
//! | WhatsApp   | `https://wa.me/numero?text=...`           |
//! | Ubicación  | `geo:latitud,longitud`                    |
//! | Evento     | iCalendar (`BEGIN:VEVENT ... END:VEVENT`) |

/// Tipo de contenido del QR.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Text,
    Url,
    Wifi,
    Contact,
    Email,
    Phone,
    Sms,
    WhatsApp,
    Location,
    Event,
}

impl Kind {
    pub const ALL: [Kind; 10] = [
        Kind::Url,
        Kind::Text,
        Kind::Wifi,
        Kind::Contact,
        Kind::Email,
        Kind::Phone,
        Kind::Sms,
        Kind::WhatsApp,
        Kind::Location,
        Kind::Event,
    ];
    /// Icono que acompaña al nombre.
    pub fn icon(self) -> &'static str {
        match self {
            Kind::Text => "📝",
            Kind::Url => "🔗",
            Kind::Wifi => "📶",
            Kind::Contact => "👤",
            Kind::Email => "✉",
            Kind::Phone => "📞",
            Kind::Sms => "💬",
            Kind::WhatsApp => "📱",
            Kind::Location => "📍",
            Kind::Event => "📅",
        }
    }
    /// Nombre en español (la interfaz lo traduce con `tr`).
    pub fn label(self) -> &'static str {
        match self {
            Kind::Text => "Texto",
            Kind::Url => "Enlace web",
            Kind::Wifi => "Wi-Fi",
            Kind::Contact => "Contacto",
            Kind::Email => "Correo",
            Kind::Phone => "Teléfono",
            Kind::Sms => "SMS",
            Kind::WhatsApp => "WhatsApp",
            Kind::Location => "Ubicación",
            Kind::Event => "Evento",
        }
    }
}

/// Seguridad de una red Wi-Fi.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WifiSec {
    Wpa,
    Wep,
    Open,
}

/// Los campos de todos los tipos de contenido (se conservan al cambiar de tipo).
#[derive(Clone)]
pub struct Content {
    pub kind: Kind,
    pub text: String,
    pub url: String,
    pub wifi_ssid: String,
    pub wifi_pass: String,
    pub wifi_sec: WifiSec,
    pub wifi_hidden: bool,
    pub c_name: String,
    pub c_surname: String,
    pub c_org: String,
    pub c_title: String,
    pub c_phone: String,
    pub c_email: String,
    pub c_web: String,
    pub c_address: String,
    pub mail_to: String,
    pub mail_subject: String,
    pub mail_body: String,
    pub phone: String,
    pub sms_msg: String,
    pub wa_msg: String,
    pub lat: String,
    pub lon: String,
    pub ev_title: String,
    pub ev_place: String,
    pub ev_start: String,
    pub ev_end: String,
    pub ev_desc: String,
}

impl Default for Content {
    fn default() -> Self {
        Self {
            kind: Kind::Url,
            text: "Hola, este QR se creó en mi computadora.".into(),
            url: "https://www.ejemplo.com".into(),
            wifi_ssid: "MiRedWiFi".into(),
            wifi_pass: "".into(),
            wifi_sec: WifiSec::Wpa,
            wifi_hidden: false,
            c_name: "Ana".into(),
            c_surname: "García".into(),
            c_org: "".into(),
            c_title: "".into(),
            c_phone: "+34 600 000 000".into(),
            c_email: "ana@ejemplo.com".into(),
            c_web: "".into(),
            c_address: "".into(),
            mail_to: "hola@ejemplo.com".into(),
            mail_subject: "".into(),
            mail_body: "".into(),
            phone: "+34 600 000 000".into(),
            sms_msg: "".into(),
            wa_msg: "".into(),
            lat: "40.4168".into(),
            lon: "-3.7038".into(),
            ev_title: "Reunión".into(),
            ev_place: "".into(),
            ev_start: "2026-12-01 10:00".into(),
            ev_end: "2026-12-01 11:00".into(),
            ev_desc: "".into(),
        }
    }
}

/// Codifica un texto para usarlo dentro de una dirección web (%20 para el espacio...).
fn pct(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

/// Escapa los caracteres especiales del formato WIFI: (\\ ; , : ").
fn wifi_esc(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if matches!(c, '\\' | ';' | ',' | ':' | '"') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// Escapa los caracteres especiales de vCard e iCalendar.
fn vesc(s: &str) -> String {
    s.replace('\\', "\\\\").replace(';', "\\;").replace(',', "\\,").replace('\n', "\\n")
}

/// Solo los dígitos de un texto.
fn digits(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii_digit()).collect()
}

/// Un número de teléfono sin espacios ni guiones (conserva el +).
fn tel(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii_digit() || *c == '+').collect()
}

/// "2026-12-01 10:00" → "20261201T100000"
fn ical_date(s: &str) -> String {
    let d: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    if d.len() >= 12 {
        format!("{}T{}00", &d[0..8], &d[8..12])
    } else if d.len() >= 8 {
        d[0..8].to_string()
    } else {
        d
    }
}

impl Content {
    /// El texto exacto que se guarda dentro del QR.
    pub fn payload(&self) -> String {
        match self.kind {
            Kind::Text => self.text.clone(),
            Kind::Url => {
                let u = self.url.trim();
                if u.is_empty() {
                    String::new()
                } else if u.contains("://") || u.starts_with("mailto:") {
                    u.to_string()
                } else {
                    format!("https://{u}")
                }
            }
            Kind::Wifi => {
                let t = match self.wifi_sec {
                    WifiSec::Wpa => "WPA",
                    WifiSec::Wep => "WEP",
                    WifiSec::Open => "nopass",
                };
                let mut s = format!("WIFI:T:{};S:{};", t, wifi_esc(&self.wifi_ssid));
                if self.wifi_sec != WifiSec::Open {
                    s.push_str(&format!("P:{};", wifi_esc(&self.wifi_pass)));
                }
                if self.wifi_hidden {
                    s.push_str("H:true;");
                }
                s.push(';');
                s
            }
            Kind::Contact => {
                let mut s = String::from("BEGIN:VCARD\nVERSION:3.0\n");
                s.push_str(&format!("N:{};{};;;\n", vesc(&self.c_surname), vesc(&self.c_name)));
                s.push_str(&format!(
                    "FN:{}\n",
                    vesc(format!("{} {}", self.c_name, self.c_surname).trim())
                ));
                if !self.c_org.is_empty() {
                    s.push_str(&format!("ORG:{}\n", vesc(&self.c_org)));
                }
                if !self.c_title.is_empty() {
                    s.push_str(&format!("TITLE:{}\n", vesc(&self.c_title)));
                }
                if !self.c_phone.is_empty() {
                    s.push_str(&format!("TEL;TYPE=CELL:{}\n", tel(&self.c_phone)));
                }
                if !self.c_email.is_empty() {
                    s.push_str(&format!("EMAIL:{}\n", self.c_email.trim()));
                }
                if !self.c_web.is_empty() {
                    s.push_str(&format!("URL:{}\n", self.c_web.trim()));
                }
                if !self.c_address.is_empty() {
                    s.push_str(&format!("ADR:;;{};;;;\n", vesc(&self.c_address)));
                }
                s.push_str("END:VCARD");
                s
            }
            Kind::Email => {
                let mut s = format!("mailto:{}", self.mail_to.trim());
                let mut q = Vec::new();
                if !self.mail_subject.is_empty() {
                    q.push(format!("subject={}", pct(&self.mail_subject)));
                }
                if !self.mail_body.is_empty() {
                    q.push(format!("body={}", pct(&self.mail_body)));
                }
                if !q.is_empty() {
                    s.push('?');
                    s.push_str(&q.join("&"));
                }
                s
            }
            Kind::Phone => format!("tel:{}", tel(&self.phone)),
            Kind::Sms => {
                if self.sms_msg.is_empty() {
                    format!("SMSTO:{}", tel(&self.phone))
                } else {
                    format!("SMSTO:{}:{}", tel(&self.phone), self.sms_msg)
                }
            }
            Kind::WhatsApp => {
                let mut s = format!("https://wa.me/{}", digits(&self.phone));
                if !self.wa_msg.is_empty() {
                    s.push_str(&format!("?text={}", pct(&self.wa_msg)));
                }
                s
            }
            Kind::Location => format!(
                "geo:{},{}",
                self.lat.trim().replace(',', "."),
                self.lon.trim().replace(',', ".")
            ),
            Kind::Event => {
                let mut s = String::from("BEGIN:VEVENT\n");
                s.push_str(&format!("SUMMARY:{}\n", vesc(&self.ev_title)));
                if !self.ev_place.is_empty() {
                    s.push_str(&format!("LOCATION:{}\n", vesc(&self.ev_place)));
                }
                s.push_str(&format!("DTSTART:{}\n", ical_date(&self.ev_start)));
                if !self.ev_end.is_empty() {
                    s.push_str(&format!("DTEND:{}\n", ical_date(&self.ev_end)));
                }
                if !self.ev_desc.is_empty() {
                    s.push_str(&format!("DESCRIPTION:{}\n", vesc(&self.ev_desc)));
                }
                s.push_str("END:VEVENT");
                s
            }
        }
    }
}
