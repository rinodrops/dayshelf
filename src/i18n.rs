use time::{Date, Month, Weekday};

#[derive(Clone, Copy)]
pub struct Text {
    pub new_folder: &'static str,
    pub existing: &'static str,
    pub new_card: &'static str,
    pub settings: &'static str,
    pub change_folder: &'static str,
    pub no_cards: &'static str,
    pub choose_folder: &'static str,
    pub title: &'static str,
    pub anchor: &'static str,
    pub direction: &'static str,
    pub save: &'static str,
    pub cancel: &'static str,
    pub delete: &'static str,
    pub edit: &'static str,
    pub auto: &'static str,
    pub countdown: &'static str,
    pub countup: &'static str,
    pub today: &'static str,
    pub untitled: &'static str,
    pub bad_date: &'static str,
    pub no_cards_dir: &'static str,
    pub settings_missing: &'static str,
    pub months: [&'static str; 12],
    pub weekdays: [&'static str; 7],
}

pub fn ui_language(setting: &str) -> &'static str {
    if setting != "os" && text(setting).is_some() {
        return match setting {
            "ara" => "ara",
            "cmn" => "cmn",
            "deu" => "deu",
            "fra" => "fra",
            "hin" => "hin",
            "ita" => "ita",
            "jpn" => "jpn",
            "kor" => "kor",
            "nld" => "nld",
            "por" => "por",
            "rus" => "rus",
            "spa" => "spa",
            "swe" => "swe",
            "tur" => "tur",
            "vie" => "vie",
            _ => "eng",
        };
    }
    os_whichlang()
}

pub fn text_for(setting: &str) -> &'static Text {
    text(ui_language(setting)).unwrap_or_else(|| text("eng").unwrap())
}

pub fn os_tag() -> String {
    sys_locale::get_locale().unwrap_or_else(|| "en".to_owned())
}

fn os_whichlang() -> &'static str {
    match primary(&os_tag()) {
        "ar" => "ara",
        "zh" => "cmn",
        "de" => "deu",
        "fr" => "fra",
        "hi" => "hin",
        "it" => "ita",
        "ja" => "jpn",
        "ko" => "kor",
        "nl" => "nld",
        "pt" => "por",
        "ru" => "rus",
        "es" => "spa",
        "sv" => "swe",
        "tr" => "tur",
        "vi" => "vie",
        _ => "eng",
    }
}

pub fn primary(tag: &str) -> &str {
    tag.split(['-', '_']).next().unwrap_or(tag)
}

pub fn region(tag: &str) -> Option<&str> {
    tag.split(['-', '_']).nth(1).filter(|part| part.len() == 2)
}

pub fn first_weekday(setting: &str, os: &str) -> Weekday {
    match setting {
        "sunday" => Weekday::Sunday,
        "monday" => Weekday::Monday,
        "saturday" => Weekday::Saturday,
        _ => locale_week_start(os),
    }
}

fn locale_week_start(os: &str) -> Weekday {
    if let Some(region) = region(os).map(str::to_ascii_uppercase) {
        if SATURDAY.contains(&region.as_str()) {
            return Weekday::Saturday;
        }
        if SUNDAY.contains(&region.as_str()) {
            return Weekday::Sunday;
        }
        if MONDAY.contains(&region.as_str()) {
            return Weekday::Monday;
        }
    }
    match os_whichlang_from_tag(os) {
        "ara" => Weekday::Saturday,
        "eng" | "hin" | "jpn" | "kor" | "por" => Weekday::Sunday,
        _ => Weekday::Monday,
    }
}

fn os_whichlang_from_tag(os: &str) -> &'static str {
    match primary(os) {
        "ar" => "ara",
        "zh" => "cmn",
        "de" => "deu",
        "fr" => "fra",
        "hi" => "hin",
        "it" => "ita",
        "ja" => "jpn",
        "ko" => "kor",
        "nl" => "nld",
        "pt" => "por",
        "ru" => "rus",
        "es" => "spa",
        "sv" => "swe",
        "tr" => "tur",
        "vi" => "vie",
        "en" => "eng",
        _ => "eng",
    }
}

const SATURDAY: &[&str] = &[
    "AE", "BH", "DZ", "EG", "IQ", "JO", "KW", "LY", "OM", "QA", "SA", "SD", "SY", "YE",
];
const SUNDAY: &[&str] = &[
    "US", "JP", "KR", "IN", "BR", "MX", "CA", "AU", "IL", "TW", "PH",
];
const MONDAY: &[&str] = &[
    "CN", "DE", "FR", "IT", "NL", "SE", "RU", "ES", "PT", "TR", "VN", "GB", "AT", "CH", "BE", "PL",
];

pub fn show_week_numbers(os: &str, start: Weekday) -> bool {
    start == Weekday::Monday && min_days(os) >= 4
}

fn min_days(os: &str) -> u8 {
    if let Some(region) = region(os).map(str::to_ascii_uppercase) {
        if MONDAY.contains(&region.as_str()) && region != "CN" {
            return 4;
        }
        return 1;
    }
    match os_whichlang_from_tag(os) {
        "deu" | "fra" | "ita" | "nld" | "swe" | "rus" | "spa" | "tur" | "vie" => 4,
        _ => 1,
    }
}

pub fn format_number(lang: &str, os: &str, n: i64) -> String {
    let system = digit_system(lang, os);
    let raw = n.to_string();
    raw.chars().map(|ch| digit_char(ch, system)).collect()
}

#[derive(Clone, Copy)]
enum Digits {
    Latn,
    Arab,
    Deva,
}

fn digit_system(lang: &str, os: &str) -> Digits {
    let region = region(os).map(str::to_ascii_uppercase);
    if lang == "ara" {
        if matches!(region.as_deref(), Some("MA" | "DZ" | "TN")) {
            return Digits::Latn;
        }
        return Digits::Arab;
    }
    if lang == "hin" {
        return Digits::Deva;
    }
    Digits::Latn
}

fn digit_char(ch: char, system: Digits) -> char {
    let table: &[char] = match system {
        Digits::Latn => return ch,
        Digits::Arab => &['٠', '١', '٢', '٣', '٤', '٥', '٦', '٧', '٨', '٩'],
        Digits::Deva => &['०', '१', '२', '३', '४', '५', '६', '७', '८', '९'],
    };
    match ch.to_digit(10) {
        Some(index) => table[index as usize],
        None => ch,
    }
}

pub fn month_title(lang: &str, year: i32, month: u8) -> String {
    let copy = text_for_code(lang);
    let name = Month::try_from(month)
        .ok()
        .and_then(|month| copy.months.get(u8::from(month) as usize - 1))
        .copied()
        .unwrap_or("");
    if lang == "jpn" {
        let era = era_label(year, month)
            .map(|era| format!("（{era}）"))
            .unwrap_or_default();
        return format!("{year}年{month}月{era}");
    }
    format!("{name} {year}")
}

fn era_label(year: i32, month: u8) -> Option<String> {
    let start = Date::from_calendar_date(year, Month::try_from(month).ok()?, 1).ok()?;
    let reiwa = Date::from_calendar_date(2019, Month::May, 1).ok()?;
    let heisei = Date::from_calendar_date(1989, Month::January, 8).ok()?;
    let showa = Date::from_calendar_date(1926, Month::December, 25).ok()?;
    if start >= reiwa {
        Some(format!("令和{}年", year - 2018))
    } else if start >= heisei {
        Some(format!("平成{}年", year - 1988))
    } else if start >= showa {
        Some(format!("昭和{}年", year - 1925))
    } else {
        None
    }
}

pub fn day_phrase(lang: &str, n: i64, kind: crate::days::CountKind) -> String {
    let copy = text_for_code(lang);
    if kind == crate::days::CountKind::Today {
        return copy.today.to_owned();
    }
    let word = plural_unit(lang, n.abs());
    match kind {
        crate::days::CountKind::Countdown => match lang {
            "jpn" => format!("あと{n}日"),
            "cmn" => format!("还有{n}天"),
            "kor" => format!("{n}일 남음"),
            "ara" => format!("{n} {word}"),
            "rus" => format!("ещё {n} {word}"),
            _ => format!("{n} {word}"),
        },
        _ => match lang {
            "jpn" => format!("{n}日"),
            "cmn" => format!("{n}天"),
            "kor" => format!("{n}일"),
            _ => format!("{n} {word}"),
        },
    }
}

fn plural_unit(lang: &str, n: i64) -> &'static str {
    match lang {
        "rus" => match n % 100 {
            11..=14 => "дней",
            _ => match n % 10 {
                1 => "день",
                2..=4 => "дня",
                _ => "дней",
            },
        },
        "ara" => match n {
            0 => "يوم",
            1 => "يوم",
            2 => "يومين",
            3..=10 => "أيام",
            _ => "يومًا",
        },
        "fra" => {
            if n == 0 || n == 1 {
                "jour"
            } else {
                "jours"
            }
        }
        "jpn" | "cmn" | "kor" | "vie" | "tur" => "",
        _ => {
            if n == 1 {
                "day"
            } else {
                "days"
            }
        }
    }
}

pub fn title_is_rtl(title: &str, override_lang: Option<&str>) -> bool {
    let code = override_lang
        .filter(|code| !code.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| {
            whichlang::detect_language(title)
                .three_letter_code()
                .to_owned()
        });
    code == "ara"
}

fn text_for_code(lang: &str) -> &'static Text {
    text(lang).unwrap_or_else(|| text("eng").unwrap())
}

fn text(code: &str) -> Option<&'static Text> {
    Some(match code {
        "eng" => &ENG,
        "jpn" => &JPN,
        "cmn" => &CMN,
        "kor" => &KOR,
        "deu" => &DEU,
        "fra" => &FRA,
        "spa" => &SPA,
        "por" => &POR,
        "ita" => &ITA,
        "nld" => &NLD,
        "swe" => &SWE,
        "rus" => &RUS,
        "tur" => &TUR,
        "ara" => &ARA,
        "hin" => &HIN,
        "vie" => &VIE,
        _ => return None,
    })
}

const ENG: Text = Text {
    new_folder: "New",
    existing: "Existing",
    new_card: "New card",
    settings: "Settings",
    change_folder: "Change folder",
    no_cards: "No cards yet.",
    choose_folder: "Choose a folder for the cards. A cloud-storage folder works the same way.",
    title: "Title",
    anchor: "Anchor date",
    direction: "Direction",
    save: "Save",
    cancel: "Cancel",
    delete: "Delete",
    edit: "Edit",
    auto: "Auto",
    countdown: "Countdown",
    countup: "Countup",
    today: "Today",
    untitled: "Untitled",
    bad_date: "Use a date like 2026-11-03.",
    no_cards_dir: "That folder has no cards directory.",
    settings_missing: "The settings program was not found.",
    months: [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ],
    weekdays: ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"],
};

const JPN: Text = Text {
    new_folder: "新規",
    existing: "既存",
    new_card: "新しいカード",
    settings: "設定",
    change_folder: "フォルダを変更",
    no_cards: "カードはまだありません。",
    choose_folder:
        "カードを置くフォルダを選んでください。クラウドストレージのフォルダでも同じです。",
    title: "タイトル",
    anchor: "基準日",
    direction: "向き",
    save: "保存",
    cancel: "キャンセル",
    delete: "削除",
    edit: "編集",
    auto: "自動",
    countdown: "カウントダウン",
    countup: "カウントアップ",
    today: "今日",
    untitled: "無題",
    bad_date: "2026-11-03 のように日付を書いてください。",
    no_cards_dir: "そのフォルダには cards がありません。",
    settings_missing: "設定プログラムが見つかりません。",
    months: [
        "1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月", "12月",
    ],
    weekdays: ["月", "火", "水", "木", "金", "土", "日"],
};

const CMN: Text = Text {
    new_folder: "新建",
    existing: "现有",
    new_card: "新建卡片",
    settings: "设置",
    change_folder: "更改文件夹",
    no_cards: "还没有卡片。",
    choose_folder: "选择存放卡片的文件夹。云存储文件夹同样可用。",
    title: "标题",
    anchor: "基准日",
    direction: "方向",
    save: "保存",
    cancel: "取消",
    delete: "删除",
    edit: "编辑",
    auto: "自动",
    countdown: "倒计时",
    countup: "正计时",
    today: "今天",
    untitled: "无标题",
    bad_date: "请使用 2026-11-03 这样的日期。",
    no_cards_dir: "该文件夹没有 cards 目录。",
    settings_missing: "未找到设置程序。",
    months: [
        "1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月", "12月",
    ],
    weekdays: ["一", "二", "三", "四", "五", "六", "日"],
};

const KOR: Text = Text {
    new_folder: "새로 만들기",
    existing: "기존",
    new_card: "새 카드",
    settings: "설정",
    change_folder: "폴더 변경",
    no_cards: "카드가 없습니다.",
    choose_folder: "카드를 둘 폴더를 고르세요. 클라우드 폴더도 같습니다.",
    title: "제목",
    anchor: "기준일",
    direction: "방향",
    save: "저장",
    cancel: "취소",
    delete: "삭제",
    edit: "편집",
    auto: "자동",
    countdown: "카운트다운",
    countup: "카운트업",
    today: "오늘",
    untitled: "제목 없음",
    bad_date: "2026-11-03 형식으로 입력하세요.",
    no_cards_dir: "그 폴더에는 cards 가 없습니다.",
    settings_missing: "설정 프로그램을 찾지 못했습니다.",
    months: [
        "1월", "2월", "3월", "4월", "5월", "6월", "7월", "8월", "9월", "10월", "11월", "12월",
    ],
    weekdays: ["월", "화", "수", "목", "금", "토", "일"],
};

macro_rules! latin {
    ($new_folder:literal, $existing:literal, $new_card:literal, $settings:literal, $change:literal, $none:literal, $choose:literal, $title:literal, $anchor:literal, $direction:literal, $save:literal, $cancel:literal, $delete:literal, $edit:literal, $auto:literal, $down:literal, $up:literal, $today:literal, $untitled:literal, $bad:literal, $nodir:literal, $missing:literal, $months:expr, $days:expr) => {
        Text {
            new_folder: $new_folder,
            existing: $existing,
            new_card: $new_card,
            settings: $settings,
            change_folder: $change,
            no_cards: $none,
            choose_folder: $choose,
            title: $title,
            anchor: $anchor,
            direction: $direction,
            save: $save,
            cancel: $cancel,
            delete: $delete,
            edit: $edit,
            auto: $auto,
            countdown: $down,
            countup: $up,
            today: $today,
            untitled: $untitled,
            bad_date: $bad,
            no_cards_dir: $nodir,
            settings_missing: $missing,
            months: $months,
            weekdays: $days,
        }
    };
}

const DEU: Text = latin!(
    "Neu",
    "Vorhanden",
    "Neue Karte",
    "Einstellungen",
    "Ordner wechseln",
    "Noch keine Karten.",
    "Ordner für die Karten wählen. Ein Cloud-Ordner funktioniert genauso.",
    "Titel",
    "Stichtag",
    "Richtung",
    "Sichern",
    "Abbrechen",
    "Löschen",
    "Bearbeiten",
    "Automatisch",
    "Countdown",
    "Countup",
    "Heute",
    "Ohne Titel",
    "Datum wie 2026-11-03 verwenden.",
    "Dieser Ordner hat kein cards-Verzeichnis.",
    "Das Einstellungsprogramm wurde nicht gefunden.",
    [
        "Januar",
        "Februar",
        "März",
        "April",
        "Mai",
        "Juni",
        "Juli",
        "August",
        "September",
        "Oktober",
        "November",
        "Dezember"
    ],
    ["Mo", "Di", "Mi", "Do", "Fr", "Sa", "So"]
);
const FRA: Text = latin!(
    "Nouveau",
    "Existant",
    "Nouvelle carte",
    "Réglages",
    "Changer de dossier",
    "Aucune carte.",
    "Choisissez un dossier pour les cartes. Un dossier cloud convient aussi.",
    "Titre",
    "Date",
    "Sens",
    "Enregistrer",
    "Annuler",
    "Supprimer",
    "Modifier",
    "Auto",
    "Compte à rebours",
    "Compte croissant",
    "Aujourd’hui",
    "Sans titre",
    "Utilisez une date comme 2026-11-03.",
    "Ce dossier n’a pas de répertoire cards.",
    "Le programme de réglages est introuvable.",
    [
        "janvier",
        "février",
        "mars",
        "avril",
        "mai",
        "juin",
        "juillet",
        "août",
        "septembre",
        "octobre",
        "novembre",
        "décembre"
    ],
    ["lu", "ma", "me", "je", "ve", "sa", "di"]
);
const SPA: Text = latin!(
    "Nueva",
    "Existente",
    "Nueva tarjeta",
    "Ajustes",
    "Cambiar carpeta",
    "Aún no hay tarjetas.",
    "Elige una carpeta para las tarjetas. Una carpeta en la nube sirve igual.",
    "Título",
    "Fecha",
    "Sentido",
    "Guardar",
    "Cancelar",
    "Eliminar",
    "Editar",
    "Auto",
    "Cuenta atrás",
    "Cuenta adelante",
    "Hoy",
    "Sin título",
    "Usa una fecha como 2026-11-03.",
    "Esa carpeta no tiene un directorio cards.",
    "No se encontró el programa de ajustes.",
    [
        "enero",
        "febrero",
        "marzo",
        "abril",
        "mayo",
        "junio",
        "julio",
        "agosto",
        "septiembre",
        "octubre",
        "noviembre",
        "diciembre"
    ],
    ["lu", "ma", "mi", "ju", "vi", "sá", "do"]
);
const POR: Text = latin!(
    "Nova",
    "Existente",
    "Nova carta",
    "Definições",
    "Mudar pasta",
    "Ainda não há cartas.",
    "Escolha uma pasta para as cartas. Uma pasta na nuvem funciona da mesma forma.",
    "Título",
    "Data",
    "Sentido",
    "Guardar",
    "Cancelar",
    "Apagar",
    "Editar",
    "Auto",
    "Contagem decrescente",
    "Contagem crescente",
    "Hoje",
    "Sem título",
    "Use uma data como 2026-11-03.",
    "Essa pasta não tem um diretório cards.",
    "O programa de definições não foi encontrado.",
    [
        "janeiro",
        "fevereiro",
        "março",
        "abril",
        "maio",
        "junho",
        "julho",
        "agosto",
        "setembro",
        "outubro",
        "novembro",
        "dezembro"
    ],
    ["seg", "ter", "qua", "qui", "sex", "sáb", "dom"]
);
const ITA: Text = latin!(
    "Nuova",
    "Esistente",
    "Nuova scheda",
    "Impostazioni",
    "Cambia cartella",
    "Nessuna scheda.",
    "Scegli una cartella per le schede. Una cartella cloud va bene lo stesso.",
    "Titolo",
    "Data",
    "Direzione",
    "Salva",
    "Annulla",
    "Elimina",
    "Modifica",
    "Auto",
    "Conto alla rovescia",
    "Conto progressivo",
    "Oggi",
    "Senza titolo",
    "Usa una data come 2026-11-03.",
    "Quella cartella non ha una directory cards.",
    "Programma delle impostazioni non trovato.",
    [
        "gennaio",
        "febbraio",
        "marzo",
        "aprile",
        "maggio",
        "giugno",
        "luglio",
        "agosto",
        "settembre",
        "ottobre",
        "novembre",
        "dicembre"
    ],
    ["lun", "mar", "mer", "gio", "ven", "sab", "dom"]
);
const NLD: Text = latin!(
    "Nieuw",
    "Bestaand",
    "Nieuwe kaart",
    "Instellingen",
    "Map wijzigen",
    "Nog geen kaarten.",
    "Kies een map voor de kaarten. Een cloudmap werkt hetzelfde.",
    "Titel",
    "Datum",
    "Richting",
    "Bewaren",
    "Annuleren",
    "Verwijderen",
    "Bewerken",
    "Auto",
    "Aftellen",
    "Optellen",
    "Vandaag",
    "Naamloos",
    "Gebruik een datum zoals 2026-11-03.",
    "Die map heeft geen cards-map.",
    "Het instellingenprogramma is niet gevonden.",
    [
        "januari",
        "februari",
        "maart",
        "april",
        "mei",
        "juni",
        "juli",
        "augustus",
        "september",
        "oktober",
        "november",
        "december"
    ],
    ["ma", "di", "wo", "do", "vr", "za", "zo"]
);
const SWE: Text = latin!(
    "Ny",
    "Befintlig",
    "Nytt kort",
    "Inställningar",
    "Byt mapp",
    "Inga kort ännu.",
    "Välj en mapp för korten. En molnmapp fungerar på samma sätt.",
    "Titel",
    "Datum",
    "Riktning",
    "Spara",
    "Avbryt",
    "Ta bort",
    "Redigera",
    "Auto",
    "Nedräkning",
    "Uppräkning",
    "I dag",
    "Namnlöst",
    "Använd ett datum som 2026-11-03.",
    "Den mappen har ingen cards-katalog.",
    "Inställningsprogrammet hittades inte.",
    [
        "januari",
        "februari",
        "mars",
        "april",
        "maj",
        "juni",
        "juli",
        "augusti",
        "september",
        "oktober",
        "november",
        "december"
    ],
    ["må", "ti", "on", "to", "fr", "lö", "sö"]
);
const RUS: Text = latin!(
    "Новая",
    "Существующая",
    "Новая карточка",
    "Настройки",
    "Сменить папку",
    "Карточек пока нет.",
    "Выберите папку для карточек. Папка в облаке работает так же.",
    "Название",
    "Дата",
    "Направление",
    "Сохранить",
    "Отмена",
    "Удалить",
    "Изменить",
    "Авто",
    "Обратный отсчёт",
    "Прямой отсчёт",
    "Сегодня",
    "Без названия",
    "Используйте дату вида 2026-11-03.",
    "В этой папке нет каталога cards.",
    "Программа настроек не найдена.",
    [
        "январь",
        "февраль",
        "март",
        "апрель",
        "май",
        "июнь",
        "июль",
        "август",
        "сентябрь",
        "октябрь",
        "ноябрь",
        "декабрь"
    ],
    ["пн", "вт", "ср", "чт", "пт", "сб", "вс"]
);
const TUR: Text = latin!(
    "Yeni",
    "Var olan",
    "Yeni kart",
    "Ayarlar",
    "Klasörü değiştir",
    "Henüz kart yok.",
    "Kartlar için bir klasör seçin. Bulut klasörü de aynı şekilde çalışır.",
    "Başlık",
    "Tarih",
    "Yön",
    "Kaydet",
    "İptal",
    "Sil",
    "Düzenle",
    "Otomatik",
    "Geri sayım",
    "İleri sayım",
    "Bugün",
    "Adsız",
    "2026-11-03 gibi bir tarih kullanın.",
    "Bu klasörde cards dizini yok.",
    "Ayar programı bulunamadı.",
    [
        "ocak", "şubat", "mart", "nisan", "mayıs", "haziran", "temmuz", "ağustos", "eylül", "ekim",
        "kasım", "aralık"
    ],
    ["pt", "sa", "ça", "pe", "cu", "ct", "pz"]
);
const ARA: Text = latin!(
    "جديد",
    "موجود",
    "بطاقة جديدة",
    "الإعدادات",
    "تغيير المجلد",
    "لا توجد بطاقات بعد.",
    "اختر مجلدًا للبطاقات. مجلد التخزين السحابي يعمل بالطريقة نفسها.",
    "العنوان",
    "التاريخ",
    "الاتجاه",
    "حفظ",
    "إلغاء",
    "حذف",
    "تحرير",
    "تلقائي",
    "عد تنازلي",
    "عد تصاعدي",
    "اليوم",
    "بدون عنوان",
    "استخدم تاريخًا مثل 2026-11-03.",
    "هذا المجلد لا يحتوي على دليل cards.",
    "لم يُعثر على برنامج الإعدادات.",
    [
        "يناير",
        "فبراير",
        "مارس",
        "أبريل",
        "مايو",
        "يونيو",
        "يوليو",
        "أغسطس",
        "سبتمبر",
        "أكتوبر",
        "نوفمبر",
        "ديسمبر"
    ],
    ["إث", "ثل", "أر", "خم", "جم", "سب", "أح"]
);
const HIN: Text = latin!(
    "नया",
    "मौजूदा",
    "नया कार्ड",
    "सेटिंग्स",
    "फ़ोल्डर बदलें",
    "अभी कोई कार्ड नहीं।",
    "कार्डों के लिए एक फ़ोल्डर चुनें। क्लाउड फ़ोल्डर भी उसी तरह काम करता है।",
    "शीर्षक",
    "तिथि",
    "दिशा",
    "सहेजें",
    "रद्द",
    "हटाएँ",
    "संपादित करें",
    "स्वतः",
    "उलटी गिनती",
    "आगे की गिनती",
    "आज",
    "शीर्षकहीन",
    "2026-11-03 जैसी तिथि लिखें।",
    "उस फ़ोल्डर में cards निर्देशिका नहीं है।",
    "सेटिंग्स प्रोग्राम नहीं मिला।",
    [
        "जनवरी",
        "फ़रवरी",
        "मार्च",
        "अप्रैल",
        "मई",
        "जून",
        "जुलाई",
        "अगस्त",
        "सितंबर",
        "अक्टूबर",
        "नवंबर",
        "दिसंबर"
    ],
    ["सोम", "मंग", "बुध", "गुरु", "शुक्र", "शनि", "रवि"]
);
const VIE: Text = latin!(
    "Mới",
    "Có sẵn",
    "Thẻ mới",
    "Cài đặt",
    "Đổi thư mục",
    "Chưa có thẻ.",
    "Chọn thư mục cho các thẻ. Thư mục đám mây cũng dùng được.",
    "Tiêu đề",
    "Ngày",
    "Hướng",
    "Lưu",
    "Hủy",
    "Xóa",
    "Sửa",
    "Tự động",
    "Đếm ngược",
    "Đếm xuôi",
    "Hôm nay",
    "Không tiêu đề",
    "Dùng ngày dạng 2026-11-03.",
    "Thư mục đó không có thư mục cards.",
    "Không tìm thấy chương trình cài đặt.",
    [
        "tháng 1",
        "tháng 2",
        "tháng 3",
        "tháng 4",
        "tháng 5",
        "tháng 6",
        "tháng 7",
        "tháng 8",
        "tháng 9",
        "tháng 10",
        "tháng 11",
        "tháng 12"
    ],
    ["T2", "T3", "T4", "T5", "T6", "T7", "CN"]
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn japanese_november_2026_includes_reiwa_8() {
        assert_eq!(month_title("jpn", 2026, 11), "2026年11月（令和8年）");
    }

    #[test]
    fn arabic_digits_and_maghreb_latin() {
        assert_eq!(format_number("ara", "ar-SA", 12), "١٢");
        assert_eq!(format_number("ara", "ar-MA", 12), "12");
        assert_eq!(format_number("hin", "hi-IN", 12), "१२");
    }

    #[test]
    fn russian_plural_and_saturday_default() {
        assert!(day_phrase("rus", 1, crate::days::CountKind::Countup).contains("день"));
        assert!(day_phrase("rus", 2, crate::days::CountKind::Countup).contains("дня"));
        assert!(day_phrase("rus", 5, crate::days::CountKind::Countup).contains("дней"));
        assert_eq!(first_weekday("locale", "ar"), Weekday::Saturday);
        assert_eq!(first_weekday("locale", "en-US"), Weekday::Sunday);
        assert_eq!(first_weekday("locale", "zh-CN"), Weekday::Monday);
        assert!(!show_week_numbers("zh-CN", Weekday::Monday));
        assert!(show_week_numbers("de-DE", Weekday::Monday));
    }

    #[test]
    fn arabic_title_is_right_to_left() {
        assert!(title_is_rtl("مرحبا بالعالم", None));
        assert!(!title_is_rtl("hello there friend", None));
        assert!(title_is_rtl("hello", Some("ara")));
    }
}
