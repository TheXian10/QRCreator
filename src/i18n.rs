//! Idiomas de la interfaz: español, inglés, portugués y francés.
//!
//! Cómo funciona: el texto se escribe en español dentro del código (`tr("Guardar PNG")`) y
//! esta tabla da su traducción a los demás idiomas. Si falta una traducción se muestra el
//! español, así que nunca aparece un hueco vacío.
//!
//! Para añadir un idioma nuevo: añade una variante a `Lang`, una columna más a cada fila
//! de `TABLE` y su nombre en `Lang::name`.
//!
//! Los textos con datos usan marcadores `{0}`, `{1}`... que rellena `trf`.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::OnceLock;

/// Un idioma de la interfaz.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Lang {
    Es,
    En,
    Pt,
    Fr,
}

impl Lang {
    pub const ALL: [Lang; 4] = [Lang::Es, Lang::En, Lang::Pt, Lang::Fr];

    /// Código corto ("es", "en"...), el que se guarda en los ajustes.
    pub fn code(self) -> &'static str {
        match self {
            Lang::Es => "es",
            Lang::En => "en",
            Lang::Pt => "pt",
            Lang::Fr => "fr",
        }
    }

    /// Nombre del idioma escrito en ese mismo idioma.
    pub fn name(self) -> &'static str {
        match self {
            Lang::Es => "Español",
            Lang::En => "English",
            Lang::Pt => "Português",
            Lang::Fr => "Français",
        }
    }

    /// Elige el idioma a partir de un código como "en-US", "pt_BR" o "fr". Si no se
    /// reconoce, inglés para el resto del mundo… salvo que esté vacío (español).
    pub fn from_locale(s: &str) -> Lang {
        let s = s.trim().to_lowercase();
        if s.is_empty() || s.starts_with("es") {
            Lang::Es
        } else if s.starts_with("pt") {
            Lang::Pt
        } else if s.starts_with("fr") {
            Lang::Fr
        } else {
            Lang::En
        }
    }

    fn column(self) -> usize {
        match self {
            Lang::Es => 0,
            Lang::En => 1,
            Lang::Pt => 2,
            Lang::Fr => 3,
        }
    }
}

static LANG: AtomicU8 = AtomicU8::new(0);

/// Cambia el idioma de toda la interfaz.
pub fn set_lang(l: Lang) {
    LANG.store(l.column() as u8, Ordering::Relaxed);
}

/// El idioma activo.
pub fn lang() -> Lang {
    Lang::ALL[LANG.load(Ordering::Relaxed) as usize % Lang::ALL.len()]
}

/// Índice de la tabla por el texto en español (se construye la primera vez).
fn index() -> &'static HashMap<&'static str, &'static [&'static str; 4]> {
    static IDX: OnceLock<HashMap<&'static str, &'static [&'static str; 4]>> = OnceLock::new();
    IDX.get_or_init(|| TABLE.iter().map(|row| (row[0], row)).collect())
}

/// Traduce un texto escrito en español al idioma activo.
pub fn tr(es: &str) -> &str {
    let col = lang().column();
    match index().get(es) {
        Some(row) => row[col],
        None => es,
    }
}

/// Traduce y rellena los marcadores `{0}`, `{1}`... con los datos dados.
pub fn trf(es: &str, args: &[&dyn std::fmt::Display]) -> String {
    let mut s = tr(es).to_string();
    for (i, a) in args.iter().enumerate() {
        s = s.replace(&format!("{{{i}}}"), &a.to_string());
    }
    s
}

/// Si `text` es un texto de la tabla en el idioma `from`, lo devuelve en `to`
/// (sirve para traducir la etiqueta del marco al cambiar de idioma).
pub fn retranslate(text: &str, from: Lang, to: Lang) -> String {
    let t = text.trim();
    TABLE
        .iter()
        .find(|row| row[from.column()] == t)
        .map(|row| row[to.column()].to_string())
        .unwrap_or_else(|| text.to_string())
}

/// Traducciones: [español, inglés, portugués, francés].
static TABLE: &[[&str; 4]] = &[
    // ── Secciones
    ["Diseños", "Designs", "Designs", "Modèles"],
    ["Contenido", "Content", "Conteúdo", "Contenu"],
    ["Formas", "Shapes", "Formas", "Formes"],
    ["Colores", "Colors", "Cores", "Couleurs"],
    ["Logo", "Logo", "Logo", "Logo"],
    ["Marco", "Frame", "Moldura", "Cadre"],
    ["Reglas", "Rules", "Regras", "Règles"],
    ["Elige un diseño listo y luego personalízalo", "Pick a ready-made design, then make it yours", "Escolha um design pronto e depois personalize", "Choisissez un modèle prêt puis personnalisez-le"],
    ["Lo que se abrirá al escanear el código", "What opens when the code is scanned", "O que abre ao escanear o código", "Ce qui s'ouvre quand on scanne le code"],
    ["La forma de los puntos y de las esquinas", "The shape of the dots and the corners", "A forma dos pontos e dos cantos", "La forme des points et des coins"],
    ["Colores sólidos o degradados", "Solid colors or gradients", "Cores sólidas ou degradês", "Couleurs unies ou dégradés"],
    ["Tu imagen en el centro del código", "Your image in the middle of the code", "Sua imagem no centro do código", "Votre image au centre du code"],
    ["Una etiqueta que invite a escanear", "A label that invites people to scan", "Uma etiqueta que convida a escanear", "Une étiquette qui invite à scanner"],
    ["Ajustes técnicos de la norma QR", "Technical settings of the QR standard", "Ajustes técnicos da norma QR", "Réglages techniques de la norme QR"],
    // ── Guardar, abrir, copiar
    ["Guardado en {0}", "Saved to {0}", "Salvo em {0}", "Enregistré dans {0}"],
    ["No se pudo guardar: {0}", "Couldn't save: {0}", "Não foi possível salvar: {0}", "Impossible d'enregistrer : {0}"],
    ["Guardar QR como PNG", "Save QR as PNG", "Salvar QR como PNG", "Enregistrer le QR en PNG"],
    ["Imagen PNG", "PNG image", "Imagem PNG", "Image PNG"],
    ["No hay un QR válido.", "There's no valid QR.", "Não há um QR válido.", "Aucun QR valide."],
    ["Guardar QR como SVG", "Save QR as SVG", "Salvar QR como SVG", "Enregistrer le QR en SVG"],
    ["Imagen vectorial SVG", "SVG vector image", "Imagem vetorial SVG", "Image vectorielle SVG"],
    ["Imagen copiada. Pégala donde quieras.", "Image copied. Paste it wherever you like.", "Imagem copiada. Cole onde quiser.", "Image copiée. Collez-la où vous voulez."],
    ["No se pudo copiar: {0}", "Couldn't copy: {0}", "Não foi possível copiar: {0}", "Impossible de copier : {0}"],
    ["Guardar diseño", "Save design", "Salvar design", "Enregistrer le modèle"],
    ["Diseño de QR", "QR design", "Design de QR", "Modèle de QR"],
    ["Cargar diseño", "Open design", "Abrir design", "Ouvrir un modèle"],
    ["Diseño cargado.", "Design opened.", "Design aberto.", "Modèle ouvert."],
    ["Ese archivo no es un diseño válido: {0}", "That file isn't a valid design: {0}", "Esse arquivo não é um design válido: {0}", "Ce fichier n'est pas un modèle valide : {0}"],
    ["Subí la corrección de errores a H para que el logo no impida leer el código.", "Error correction raised to H so the logo doesn't stop the code from scanning.", "A correção de erros subiu para H para o logo não impedir a leitura.", "Correction d'erreurs passée à H pour que le logo n'empêche pas la lecture."],
    ["Elegir logo", "Choose logo", "Escolher logo", "Choisir un logo"],
    ["Imágenes", "Images", "Imagens", "Images"],
    ["No se pudo abrir la imagen: {0}", "Couldn't open the image: {0}", "Não foi possível abrir a imagem: {0}", "Impossible d'ouvrir l'image : {0}"],
    // ── Barra superior
    ["Creador de QR", "QR Creator", "Criador de QR", "Créateur de QR"],
    ["Todo se hace en tu dispositivo, sin internet", "Everything happens on your device, no internet needed", "Tudo acontece no seu dispositivo, sem internet", "Tout se fait sur votre appareil, sans internet"],
    ["Modo claro", "Light mode", "Modo claro", "Mode clair"],
    ["Modo oscuro", "Dark mode", "Modo escuro", "Mode sombre"],
    ["Escribe un contenido válido para ver las miniaturas.", "Enter valid content to see the previews.", "Escreva um conteúdo válido para ver as miniaturas.", "Saisissez un contenu valide pour voir les aperçus."],
    // ── Contenido
    ["¿Qué quieres que abra el QR?", "What should the QR open?", "O que o QR deve abrir?", "Que doit ouvrir le QR ?"],
    ["Datos", "Details", "Dados", "Données"],
    ["Texto", "Text", "Texto", "Texte"],
    ["Dirección web", "Web address", "Endereço web", "Adresse web"],
    ["Nombre de la red", "Network name", "Nome da rede", "Nom du réseau"],
    ["Seguridad", "Security", "Segurança", "Sécurité"],
    ["Abierta", "Open", "Aberta", "Ouvert"],
    ["Contraseña", "Password", "Senha", "Mot de passe"],
    ["Red oculta", "Hidden network", "Rede oculta", "Réseau masqué"],
    ["Nombre", "First name", "Nome", "Prénom"],
    ["Apellidos", "Last name", "Sobrenome", "Nom"],
    ["Teléfono", "Phone", "Telefone", "Téléphone"],
    ["Correo", "Email", "E-mail", "E-mail"],
    ["Empresa", "Company", "Empresa", "Entreprise"],
    ["opcional", "optional", "opcional", "facultatif"],
    ["Cargo", "Job title", "Cargo", "Poste"],
    ["Web", "Website", "Site", "Site web"],
    ["Dirección", "Address", "Endereço", "Adresse"],
    ["Para", "To", "Para", "À"],
    ["Asunto", "Subject", "Assunto", "Objet"],
    ["Mensaje", "Message", "Mensagem", "Message"],
    ["Número", "Number", "Número", "Numéro"],
    ["Número (con prefijo)", "Number (with country code)", "Número (com código do país)", "Numéro (avec indicatif)"],
    ["Latitud", "Latitude", "Latitude", "Latitude"],
    ["Longitud", "Longitude", "Longitude", "Longitude"],
    ["Título", "Title", "Título", "Titre"],
    ["Lugar", "Place", "Local", "Lieu"],
    ["Empieza", "Starts", "Começa", "Début"],
    ["AAAA-MM-DD HH:MM", "YYYY-MM-DD HH:MM", "AAAA-MM-DD HH:MM", "AAAA-MM-JJ HH:MM"],
    ["Termina", "Ends", "Termina", "Fin"],
    ["Descripción", "Description", "Descrição", "Description"],
    ["Ver el texto exacto que llevará el QR", "See the exact text inside the QR", "Ver o texto exato que o QR terá", "Voir le texte exact contenu dans le QR"],
    ["{0} caracteres · {1} bytes", "{0} characters · {1} bytes", "{0} caracteres · {1} bytes", "{0} caractères · {1} octets"],
    ["Enlace web", "Web link", "Link", "Lien web"],
    ["Wi-Fi", "Wi-Fi", "Wi-Fi", "Wi-Fi"],
    ["Contacto", "Contact", "Contato", "Contact"],
    ["SMS", "SMS", "SMS", "SMS"],
    ["WhatsApp", "WhatsApp", "WhatsApp", "WhatsApp"],
    ["Ubicación", "Location", "Localização", "Position"],
    ["Evento", "Event", "Evento", "Événement"],
    // ── Formas
    ["Forma de los módulos", "Module shape", "Forma dos módulos", "Forme des modules"],
    ["Tamaño del módulo", "Module size", "Tamanho do módulo", "Taille des modules"],
    ["Más pequeño deja huecos entre módulos (más ligero), pero cuesta un poco más de leer.", "Smaller leaves gaps between modules (lighter look), but is a little harder to scan.", "Menor deixa espaços entre módulos (mais leve), mas fica um pouco mais difícil de ler.", "Plus petit laisse des espaces entre les modules (plus léger), mais se lit un peu moins bien."],
    ["Marco de los ojos (esquinas)", "Eye frames (corners)", "Moldura dos olhos (cantos)", "Contour des yeux (coins)"],
    ["Centro de los ojos", "Eye centers", "Centro dos olhos", "Centre des yeux"],
    ["Esquinas del fondo", "Background corners", "Cantos do fundo", "Coins du fond"],
    ["Redondeo", "Rounding", "Arredondamento", "Arrondi"],
    ["Cuadrado", "Square", "Quadrado", "Carré"],
    ["Redondeado", "Rounded", "Arredondado", "Arrondi"],
    ["Círculos", "Circles", "Círculos", "Cercles"],
    ["Puntitos", "Small dots", "Pontinhos", "Petits points"],
    ["Líquido", "Liquid", "Líquido", "Liquide"],
    ["Rombos", "Diamonds", "Losangos", "Losanges"],
    ["Barras vert.", "Vert. bars", "Barras vert.", "Barres vert."],
    ["Barras horiz.", "Horiz. bars", "Barras horiz.", "Barres horiz."],
    ["Estrellas", "Stars", "Estrelas", "Étoiles"],
    ["Cruces", "Pluses", "Cruzes", "Croix"],
    ["Corazones", "Hearts", "Corações", "Cœurs"],
    ["Hexágonos", "Hexagons", "Hexágonos", "Hexagones"],
    ["Pétalos", "Petals", "Pétalas", "Pétales"],
    ["Equis", "X marks", "Xis", "Croix en X"],
    ["Mosaico", "Mosaic", "Mosaico", "Mosaïque"],
    ["Muy redondeado", "Extra rounded", "Muito arredondado", "Très arrondi"],
    ["Círculo", "Circle", "Círculo", "Cercle"],
    ["Hoja", "Leaf", "Folha", "Feuille"],
    ["Gota", "Drop", "Gota", "Goutte"],
    ["Octágono", "Octagon", "Octógono", "Octogone"],
    ["Punteado", "Dotted", "Pontilhado", "Pointillé"],
    ["Rombo", "Diamond", "Losango", "Losange"],
    ["Estrella", "Star", "Estrela", "Étoile"],
    ["Corazón", "Heart", "Coração", "Cœur"],
    ["Hexágono", "Hexagon", "Hexágono", "Hexagone"],
    // ── Colores
    ["Color del código", "Code color", "Cor do código", "Couleur du code"],
    ["Color de los ojos", "Eye color", "Cor dos olhos", "Couleur des yeux"],
    ["Usar colores propios para los ojos", "Use separate colors for the eyes", "Usar cores próprias para os olhos", "Couleurs séparées pour les yeux"],
    ["Centro", "Center", "Centro", "Centre"],
    ["Fondo", "Background", "Fundo", "Fond"],
    ["Legibilidad", "Readability", "Legibilidade", "Lisibilité"],
    ["Intercambiar código y fondo", "Swap code and background", "Trocar código e fundo", "Inverser code et fond"],
    ["Color", "Color", "Cor", "Couleur"],
    ["Inicio", "Start", "Início", "Début"],
    ["Borde", "Edge", "Borda", "Bord"],
    ["Final", "End", "Fim", "Fin"],
    ["Intercambiar", "Swap", "Trocar", "Inverser"],
    ["Ángulo", "Angle", "Ângulo", "Angle"],
    ["El PNG y el SVG quedarán sin fondo. Ponlo siempre sobre algo claro.", "The PNG and SVG will have no background. Always place it on something light.", "O PNG e o SVG ficarão sem fundo. Coloque sempre sobre algo claro.", "Le PNG et le SVG n'auront pas de fond. Placez-le toujours sur un fond clair."],
    ["Contraste entre código y fondo: {0} : 1", "Contrast between code and background: {0} : 1", "Contraste entre código e fundo: {0} : 1", "Contraste entre code et fond : {0} : 1"],
    ["Contraste bajo: muchos móviles no podrán leerlo. Oscurece el código o aclara el fondo.", "Low contrast: many phones won't scan it. Darken the code or lighten the background.", "Contraste baixo: muitos celulares não vão ler. Escureça o código ou clareie o fundo.", "Contraste faible : beaucoup de téléphones ne le liront pas. Foncez le code ou éclaircissez le fond."],
    ["Contraste aceptable. Por encima de 4.5 : 1 es más seguro.", "Acceptable contrast. Above 4.5 : 1 is safer.", "Contraste aceitável. Acima de 4.5 : 1 é mais seguro.", "Contraste acceptable. Au-dessus de 4.5 : 1 c'est plus sûr."],
    ["Colores invertidos (código claro sobre fondo oscuro): algunos lectores antiguos no lo leen.", "Inverted colors (light code on dark background): some older readers can't scan it.", "Cores invertidas (código claro em fundo escuro): alguns leitores antigos não leem.", "Couleurs inversées (code clair sur fond foncé) : certains anciens lecteurs ne le lisent pas."],
    ["Color sólido", "Solid", "Sólida", "Unie"],
    ["Degradado lineal", "Linear gradient", "Degradê linear", "Dégradé linéaire"],
    ["Degradado radial", "Radial gradient", "Degradê radial", "Dégradé radial"],
    ["Transparente", "Transparent", "Transparente", "Transparent"],
    // ── Logo
    ["Logo en el centro", "Logo in the middle", "Logo no centro", "Logo au centre"],
    ["Elegir imagen…", "Choose image…", "Escolher imagem…", "Choisir une image…"],
    ["Logo de ejemplo", "Sample logo", "Logo de exemplo", "Logo d'exemple"],
    ["Quitar", "Remove", "Remover", "Retirer"],
    ["Imagen: {0}", "Image: {0}", "Imagem: {0}", "Image : {0}"],
    ["Puedes poner tu logo o una imagen (PNG con transparencia queda mejor).", "Add your logo or any image (a PNG with transparency looks best).", "Coloque seu logo ou uma imagem (PNG com transparência fica melhor).", "Ajoutez votre logo ou une image (un PNG transparent rend mieux)."],
    ["En el móvil, copia tus imágenes a esta carpeta: {0}", "On a phone, copy your images to this folder: {0}", "No celular, copie suas imagens para esta pasta: {0}", "Sur téléphone, copiez vos images dans ce dossier : {0}"],
    ["ejemplo", "sample", "exemplo", "exemple"],
    ["Tamaño y espacio", "Size and spacing", "Tamanho e espaço", "Taille et espace"],
    ["Tamaño", "Size", "Tamanho", "Taille"],
    ["Quitar los módulos que quedan debajo del logo", "Remove the modules under the logo", "Remover os módulos embaixo do logo", "Retirer les modules sous le logo"],
    ["Espacio alrededor", "Space around", "Espaço ao redor", "Espace autour"],
    ["Fondo del logo", "Logo background", "Fundo do logo", "Fond du logo"],
    ["Sin fondo", "None", "Sem fundo", "Aucun"],
    ["El logo tapa ≈{0} % del código. Con corrección {1} se aguanta hasta ≈{2} %.", "The logo covers ≈{0} % of the code. Correction {1} tolerates up to ≈{2} %.", "O logo cobre ≈{0} % do código. Com correção {1} aguenta até ≈{2} %.", "Le logo couvre ≈{0} % du code. La correction {1} tolère jusqu'à ≈{2} %."],
    ["El logo es grande para este nivel de corrección: hazlo más pequeño o sube la corrección a H.", "The logo is large for this correction level: make it smaller or raise correction to H.", "O logo é grande para este nível de correção: diminua-o ou suba a correção para H.", "Le logo est grand pour ce niveau de correction : réduisez-le ou passez la correction à H."],
    // ── Marco
    ["Estilo del marco", "Frame style", "Estilo da moldura", "Style du cadre"],
    ["Añade una etiqueta como «ESCANÉAME» para invitar a la gente a usarlo.", "Add a label like “SCAN ME” to invite people to use it.", "Adicione uma etiqueta como «ESCANEIE» para convidar as pessoas a usar.", "Ajoutez une étiquette comme « SCANNEZ-MOI » pour inviter à l'utiliser."],
    ["Escribe un texto para que aparezca la etiqueta.", "Type some text to show the label.", "Escreva um texto para a etiqueta aparecer.", "Saisissez un texte pour afficher l'étiquette."],
    ["Tamaño del texto", "Text size", "Tamanho do texto", "Taille du texte"],
    ["Grosor del borde", "Border width", "Espessura da borda", "Épaisseur du bord"],
    ["Sin marco", "No frame", "Sem moldura", "Sans cadre"],
    ["Etiqueta abajo", "Label below", "Etiqueta embaixo", "Étiquette en bas"],
    ["Etiqueta arriba", "Label above", "Etiqueta em cima", "Étiquette en haut"],
    ["Borde con texto", "Border with text", "Borda com texto", "Bordure avec texte"],
    // ── Reglas
    ["Corrección de errores", "Error correction", "Correção de erros", "Correction d'erreurs"],
    ["Cuanto más alta, más daño o decoración aguanta el código, pero tendrá más cuadritos.", "The higher it is, the more damage or decoration the code tolerates, but it gets more modules.", "Quanto mais alta, mais dano ou decoração o código aguenta, mas terá mais quadradinhos.", "Plus elle est élevée, plus le code tolère dégâts et décoration, mais il a plus de modules."],
    ["Subirla automáticamente si cabe sin agrandar el código", "Raise it automatically if it fits without enlarging the code", "Subir automaticamente se couber sem aumentar o código", "L'augmenter automatiquement si cela tient sans agrandir le code"],
    ["Versión (tamaño)", "Version (size)", "Versão (tamanho)", "Version (taille)"],
    ["La versión 1 tiene 21×21 módulos y cada versión suma 4 (la 40 tiene 177×177). Se usa la más pequeña donde quepa el contenido.", "Version 1 has 21×21 modules and each version adds 4 (version 40 has 177×177). The smallest one that fits the content is used.", "A versão 1 tem 21×21 módulos e cada versão soma 4 (a 40 tem 177×177). Usa-se a menor onde o conteúdo couber.", "La version 1 a 21×21 modules et chaque version en ajoute 4 (la 40 a 177×177). La plus petite qui contient le contenu est utilisée."],
    ["Mínima", "Minimum", "Mínima", "Minimum"],
    ["Máxima", "Maximum", "Máxima", "Maximum"],
    ["Patrón de máscara", "Mask pattern", "Padrão de máscara", "Motif de masque"],
    ["La norma prueba 8 máscaras y elige la que evita zonas confusas. Puedes fijar una a mano.", "The standard tries 8 masks and picks the one that avoids confusing areas. You can fix one by hand.", "A norma testa 8 máscaras e escolhe a que evita áreas confusas. Você pode fixar uma manualmente.", "La norme essaie 8 masques et choisit celui qui évite les zones confuses. Vous pouvez en fixer un."],
    ["Automática", "Automatic", "Automática", "Automatique"],
    ["Modo de codificación", "Encoding mode", "Modo de codificação", "Mode d'encodage"],
    ["Marcar el texto como UTF-8 (ECI 26) para tildes y emojis", "Mark the text as UTF-8 (ECI 26) for accents and emoji", "Marcar o texto como UTF-8 (ECI 26) para acentos e emojis", "Marquer le texte en UTF-8 (ECI 26) pour les accents et emojis"],
    ["Ayuda a algunos lectores con caracteres especiales. Casi todos los móviles ya entienden UTF-8 sin esto.", "Helps some readers with special characters. Almost every phone already understands UTF-8 without it.", "Ajuda alguns leitores com caracteres especiais. Quase todos os celulares já entendem UTF-8 sem isso.", "Aide certains lecteurs avec les caractères spéciaux. Presque tous les téléphones comprennent déjà l'UTF-8."],
    ["Zona silenciosa (margen)", "Quiet zone (margin)", "Zona de silêncio (margem)", "Zone de silence (marge)"],
    ["La norma pide 4 módulos de margen libre alrededor. Menos puede fallar en algunos lectores.", "The standard asks for a 4-module clear margin around it. Less may fail on some readers.", "A norma pede 4 módulos de margem livre ao redor. Menos pode falhar em alguns leitores.", "La norme demande une marge libre de 4 modules. Moins peut échouer sur certains lecteurs."],
    ["Margen", "Margin", "Margem", "Marge"],
    ["Con menos de 2 módulos de margen algunos lectores no lo detectan.", "With less than 2 modules of margin some readers won't detect it.", "Com menos de 2 módulos de margem alguns leitores não o detectam.", "Avec moins de 2 modules de marge certains lecteurs ne le détectent pas."],
    ["Mis diseños", "My designs", "Meus designs", "Mes modèles"],
    ["Empezar de nuevo", "Start over", "Recomeçar", "Recommencer"],
    ["Volver a las reglas de la norma", "Reset to the standard rules", "Voltar às regras da norma", "Revenir aux règles de la norme"],
    ["L · Baja (≈7 %)", "L · Low (≈7 %)", "L · Baixa (≈7 %)", "L · Faible (≈7 %)"],
    ["M · Media (≈15 %)", "M · Medium (≈15 %)", "M · Média (≈15 %)", "M · Moyenne (≈15 %)"],
    ["Q · Alta (≈25 %)", "Q · High (≈25 %)", "Q · Alta (≈25 %)", "Q · Haute (≈25 %)"],
    ["H · Máxima (≈30 %)", "H · Highest (≈30 %)", "H · Máxima (≈30 %)", "H · Maximale (≈30 %)"],
    ["Automático (el más compacto)", "Automatic (most compact)", "Automático (o mais compacto)", "Automatique (le plus compact)"],
    ["Numérico (solo 0-9)", "Numeric (0-9 only)", "Numérico (só 0-9)", "Numérique (0-9 uniquement)"],
    ["Alfanumérico (0-9 A-Z espacio $%*+-./:)", "Alphanumeric (0-9 A-Z space $%*+-./:)", "Alfanumérico (0-9 A-Z espaço $%*+-./:)", "Alphanumérique (0-9 A-Z espace $%*+-./:)"],
    ["Bytes (UTF-8, cualquier texto)", "Bytes (UTF-8, any text)", "Bytes (UTF-8, qualquer texto)", "Octets (UTF-8, tout texte)"],
    // ── Información y guardar
    ["Versión {0}", "Version {0}", "Versão {0}", "Version {0}"],
    ["{0}×{0} módulos", "{0}×{0} modules", "{0}×{0} módulos", "{0}×{0} modules"],
    ["Corrección {0}", "Correction {0}", "Correção {0}", "Correction {0}"],
    ["aguanta ≈{0} % de daño", "tolerates ≈{0} % damage", "aguenta ≈{0} % de dano", "tolère ≈{0} % de dégâts"],
    ["Máscara {0}", "Mask {0}", "Máscara {0}", "Masque {0}"],
    ["elegida sola", "chosen automatically", "escolhida sozinha", "choisi automatiquement"],
    ["fijada a mano", "set by hand", "fixada manualmente", "fixé à la main"],
    ["Capacidad usada {0} %", "Capacity used {0} %", "Capacidade usada {0} %", "Capacité utilisée {0} %"],
    ["Se puede leer", "Scans fine", "Pode ser lido", "Lisible"],
    ["Lo encontramos y leímos en la imagen completa, como haría la cámara de un móvil.", "We found and read it in the full image, as a phone camera would.", "Encontramos e lemos na imagem completa, como faria a câmera de um celular.", "Nous l'avons trouvé et lu dans l'image entière, comme le ferait l'appareil photo d'un téléphone."],
    ["Todos los módulos se distinguen bien y el contenido se recupera. Los ojos decorados los reconocen las cámaras de los móviles actuales.", "Every module is clearly distinguishable and the content is recovered. Today's phone cameras recognise decorated eyes.", "Todos os módulos se distinguem bem e o conteúdo é recuperado. As câmeras dos celulares atuais reconhecem os olhos decorados.", "Tous les modules se distinguent bien et le contenu est récupéré. Les appareils photo actuels reconnaissent les yeux décorés."],
    ["Lectura dudosa", "May not scan", "Leitura duvidosa", "Lecture douteuse"],
    ["Comprobando lectura…", "Checking readability…", "Verificando leitura…", "Vérification de la lecture…"],
    ["Guardar PNG", "Save PNG", "Salvar PNG", "Enregistrer PNG"],
    ["Guardar SVG", "Save SVG", "Salvar SVG", "Enregistrer SVG"],
    ["Ideal para imprenta: se puede agrandar sin perder calidad", "Best for print: it can be enlarged without losing quality", "Ideal para gráfica: pode ser ampliado sem perder qualidade", "Idéal pour l'impression : agrandissable sans perte de qualité"],
    ["Copiar", "Copy", "Copiar", "Copier"],
    ["Se guarda en: {0}", "Saved in: {0}", "Salvo em: {0}", "Enregistré dans : {0}"],
    ["No hay archivos todavía. Cópialos a: {0}", "No files yet. Copy them to: {0}", "Ainda não há arquivos. Copie-os para: {0}", "Pas encore de fichiers. Copiez-les dans : {0}"],
    // ── Errores del QR y de la lectura
    ["Escribe algo en el contenido para crear el QR.", "Type some content to create the QR.", "Escreva algo no conteúdo para criar o QR.", "Saisissez un contenu pour créer le QR."],
    ["El modo numérico solo admite los dígitos 0-9. Cambia el modo a «Automático» o «Bytes».", "Numeric mode only accepts the digits 0-9. Switch the mode to “Automatic” or “Bytes”.", "O modo numérico só aceita os dígitos 0-9. Mude o modo para «Automático» ou «Bytes».", "Le mode numérique n'accepte que les chiffres 0-9. Passez en « Automatique » ou « Octets »."],
    ["El modo alfanumérico solo admite 0-9, A-Z en MAYÚSCULAS, espacio y $ % * + - . / :. Cambia el modo o el texto.", "Alphanumeric mode only accepts 0-9, UPPERCASE A-Z, space and $ % * + - . / :. Change the mode or the text.", "O modo alfanumérico só aceita 0-9, A-Z em MAIÚSCULAS, espaço e $ % * + - . / :. Mude o modo ou o texto.", "Le mode alphanumérique n'accepte que 0-9, A-Z en MAJUSCULES, espace et $ % * + - . / :. Changez le mode ou le texte."],
    ["El contenido es demasiado largo para un código QR.", "The content is too long for a QR code.", "O conteúdo é longo demais para um código QR.", "Le contenu est trop long pour un code QR."],
    ["No cabe: el contenido necesita {0} bits y la versión máxima permitida ({1}) con corrección {2} admite {3}. Sube la versión máxima, baja la corrección o acorta el texto.", "It doesn't fit: the content needs {0} bits and the maximum allowed version ({1}) with correction {2} holds {3}. Raise the maximum version, lower the correction or shorten the text.", "Não cabe: o conteúdo precisa de {0} bits e a versão máxima permitida ({1}) com correção {2} comporta {3}. Suba a versão máxima, baixe a correção ou encurte o texto.", "Ça ne tient pas : le contenu demande {0} bits et la version maximale ({1}) avec la correction {2} en contient {3}. Augmentez la version, baissez la correction ou raccourcissez le texte."],
    ["Contraste insuficiente entre el código y el fondo.", "Not enough contrast between the code and the background.", "Contraste insuficiente entre o código e o fundo.", "Contraste insuffisant entre le code et le fond."],
    ["Se lee, pero el contenido no coincide.", "It scans, but the content doesn't match.", "É lido, mas o conteúdo não coincide.", "Il se lit, mais le contenu ne correspond pas."],
    ["No se pudo leer. Prueba más contraste, un logo más pequeño o más corrección.", "Couldn't scan it. Try more contrast, a smaller logo or more correction.", "Não foi possível ler. Tente mais contraste, um logo menor ou mais correção.", "Lecture impossible. Essayez plus de contraste, un logo plus petit ou plus de correction."],
    // ── Diseños de la galería
    ["Clásico", "Classic", "Clássico", "Classique"],
    ["Océano", "Ocean", "Oceano", "Océan"],
    ["Atardecer", "Sunset", "Pôr do sol", "Coucher de soleil"],
    ["Bosque", "Forest", "Floresta", "Forêt"],
    ["Uva", "Grape", "Uva", "Raisin"],
    ["Elegante", "Elegant", "Elegante", "Élégant"],
    ["Noche", "Night", "Noite", "Nuit"],
    ["Caramelo", "Candy", "Caramelo", "Bonbon"],
    ["Menta", "Mint", "Menta", "Menthe"],
    ["Coral", "Coral", "Coral", "Corail"],
    ["Cielo", "Sky", "Céu", "Ciel"],
    ["Dorado", "Gold", "Dourado", "Doré"],
    ["Lava", "Lava", "Lava", "Lave"],
    ["Café", "Coffee", "Café", "Café"],
    ["Fiesta", "Party", "Festa", "Fête"],
    ["Pizarra", "Slate", "Ardósia", "Ardoise"],
    ["Lavanda", "Lavender", "Lavanda", "Lavande"],
    // ── Textos del marco
    ["ESCANÉAME", "SCAN ME", "ESCANEIE", "SCANNEZ-MOI"],
    ["¡ESCANÉAME!", "SCAN ME!", "ESCANEIE!", "SCANNEZ-MOI !"],
    ["MI MENÚ", "MY MENU", "MEU MENU", "MON MENU"],
    ["BIENVENIDOS", "WELCOME", "BEM-VINDOS", "BIENVENUE"],
    ["CON CARIÑO", "WITH LOVE", "COM CARINHO", "AVEC AMOUR"],
    ["MÁS INFO", "MORE INFO", "MAIS INFO", "PLUS D'INFOS"],
    ["PREMIUM", "PREMIUM", "PREMIUM", "PREMIUM"],
    ["NUESTRA CARTA", "OUR MENU", "NOSSO CARDÁPIO", "NOTRE CARTE"],
    ["¡ESTÁS INVITADO!", "YOU'RE INVITED!", "VOCÊ ESTÁ CONVIDADO!", "VOUS ÊTES INVITÉ !"],
    ["VISÍTANOS", "VISIT US", "VISITE-NOS", "RENDEZ-NOUS VISITE"],
    ["MENÚ", "MENU", "CARDÁPIO", "MENU"],
    ["WIFI GRATIS", "FREE WIFI", "WIFI GRÁTIS", "WIFI GRATUIT"],
    ["SÍGUENOS", "FOLLOW US", "SIGA-NOS", "SUIVEZ-NOUS"],
    ["PAGA AQUÍ", "PAY HERE", "PAGUE AQUI", "PAYEZ ICI"],
    // ── Versión 2.2: interfaz y atajos
    ["Menú", "Menu", "Menu", "Menu"],
    ["Acerca de", "About", "Sobre", "À propos"],
    ["Vista previa", "Preview", "Pré-visualização", "Aperçu"],
    ["Se exportará a {0} × {1} px", "Exports at {0} × {1} px", "Será exportado em {0} × {1} px", "Exporté en {0} × {1} px"],
    ["Cuadros", "Checker", "Xadrez", "Damier"],
    ["Oscuro", "Dark", "Escuro", "Sombre"],
    ["Claro", "Light", "Claro", "Clair"],
    ["Fondo de la vista previa", "Preview background", "Fundo da pré-visualização", "Fond de l'aperçu"],
    ["Crea códigos QR bonitos sin internet, con todas las reglas de la norma.", "Create beautiful QR codes offline, following every rule of the standard.", "Crie códigos QR bonitos sem internet, com todas as regras da norma.", "Créez de beaux codes QR sans internet, selon toutes les règles de la norme."],
    ["Atajos de teclado", "Keyboard shortcuts", "Atalhos de teclado", "Raccourcis clavier"],
    ["Guardar la imagen", "Save the image", "Salvar a imagem", "Enregistrer l'image"],
    ["Copiar la imagen", "Copy the image", "Copiar a imagem", "Copier l'image"],
    ["Código abierto en GitHub", "Open source on GitHub", "Código aberto no GitHub", "Code source sur GitHub"],
    // ── Versión 2.1: guardar, historial, galería, iconos, formas y marcos nuevos
    ["Imagen con transparencia, para web y documentos", "Image with transparency, for web and documents", "Imagem com transparência, para web e documentos", "Image avec transparence, pour le web et les documents"],
    ["Imagen ligera con fondo blanco, para compartir", "Lightweight image on white, for sharing", "Imagem leve com fundo branco, para compartilhar", "Image légère sur fond blanc, pour partager"],
    ["Guardar QR como JPG", "Save QR as JPG", "Salvar QR como JPG", "Enregistrer le QR en JPG"],
    ["Imagen JPG", "JPG image", "Imagem JPG", "Image JPG"],
    ["Rehacer", "Redo", "Refazer", "Rétablir"],
    ["Deshacer", "Undo", "Desfazer", "Annuler"],
    ["Sorpréndeme", "Surprise me", "Surpreenda-me", "Surprends-moi"],
    ["Crea un diseño al azar que se pueda leer", "Creates a random design that still scans", "Cria um design aleatório que pode ser lido", "Crée un modèle aléatoire qui reste lisible"],
    ["Todos", "All", "Todos", "Tous"],
    ["Iconos listos", "Ready-made icons", "Ícones prontos", "Icônes prêtes"],
    ["Un icono para el centro, con el color de tu código.", "An icon for the middle, in your code's color.", "Um ícone para o centro, com a cor do seu código.", "Une icône pour le centre, à la couleur de votre code."],
    ["Recortar el logo en círculo", "Crop the logo into a circle", "Recortar o logo em círculo", "Rogner le logo en cercle"],
    ["Formato", "Format", "Formato", "Format"],
    ["Guardar", "Save", "Salvar", "Enregistrer"],
    ["Vídeo", "Video", "Vídeo", "Vidéo"],
    ["Chat", "Chat", "Chat", "Discussion"],
    ["Música", "Music", "Música", "Musique"],
    ["Gotas", "Drops", "Gotas", "Gouttes"],
    ["Octágonos", "Octagons", "Octógonos", "Octogones"],
    ["Red", "Network", "Rede", "Réseau"],
    ["Orgánico", "Organic", "Orgânico", "Organique"],
    ["Pétalo", "Petal", "Pétala", "Pétale"],
    ["Cruz", "Plus", "Cruz", "Croix"],
    ["Flor", "Flower", "Flor", "Fleur"],
    ["Nueve puntos", "Nine dots", "Nove pontos", "Neuf points"],
    ["Burbuja", "Bubble", "Balão", "Bulle"],
    ["Visor", "Viewfinder", "Visor", "Viseur"],
    ["Polaroid", "Instant photo", "Foto instantânea", "Photo instantanée"],
    ["Clásicos", "Classic", "Clássicos", "Classiques"],
    ["Coloridos", "Colorful", "Coloridos", "Colorés"],
    ["Elegantes", "Elegant", "Elegantes", "Élégants"],
    ["Divertidos", "Fun", "Divertidos", "Amusants"],
    ["Con marco", "Framed", "Com moldura", "Avec cadre"],
    ["Bocadillo", "Speech bubble", "Balão de fala", "Bulle de BD"],
    ["Conexiones", "Connections", "Conexões", "Connexions"],
    ["Lluvia", "Rain", "Chuva", "Pluie"],
    ["Panal", "Honeycomb", "Favo de mel", "Nid d'abeille"],
    ["Flores", "Flowers", "Flores", "Fleurs"],
    ["Minimalista", "Minimal", "Minimalista", "Minimaliste"],
    ["Arcoíris", "Rainbow", "Arco-íris", "Arc-en-ciel"],
    ["Esmeralda", "Emerald", "Esmeralda", "Émeraude"],
    ["Tinta", "Ink", "Tinta", "Encre"],
    ["Medianoche", "Midnight", "Meia-noite", "Minuit"],
    ["En el visor, las esquinas y el texto usan el color del marco.", "In the viewfinder, the corners and the text use the frame color.", "No visor, os cantos e o texto usam a cor da moldura.", "Dans le viseur, les coins et le texte prennent la couleur du cadre."],
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Ninguna fila repetida (la segunda nunca se usaría) y ningún marcador perdido.
    #[test]
    fn table_is_consistent() {
        let mut seen = std::collections::HashSet::new();
        for row in TABLE {
            assert!(seen.insert(row[0]), "fila repetida: {}", row[0]);
            for i in 0..4 {
                let n = row[0].matches('{').count();
                assert_eq!(row[i].matches('{').count(), n, "marcadores distintos en: {}", row[0]);
            }
        }
    }
}
