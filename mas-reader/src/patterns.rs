//! Alle Regex-Patterns und Stichwörter aus dem Python-Skript (MAS_Reader.py).
//!
//! Die Muster sind absichtlich so locker wie im Original, weil es mehrere
//! ähnliche Quittungsformate gibt. Erst verschärfen, wenn Tests es absichern.
//!
//! Nutzung: `patterns::DATE.find(zeile)`

// Noch nicht alles wird verwendet.
#![allow(dead_code)]

use regex::Regex;
use std::sync::LazyLock;

// ---------------------------------------------------------------------------
// Regex für Werte (Python: `Regex_Patterns`)
// ---------------------------------------------------------------------------

/// Langes Datum, z. B. `30.01.2023` (Python: "Datum")
pub static DATE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\d{2}\.(0[1-9]|1[0-2])\.\d{4}").unwrap());

/// Kurzes Datum, z. B. `30.01.23` (Python: "Date", dort unbenutzt)
pub static DATE_SHORT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\d{2}\.(0[1-9]|1[0-2])\.\d{2}").unwrap());

/// Uhrzeit, z. B. `17:16:08` (Python: "Uhrzeit", dort unbenutzt)
pub static TIME: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\d{2}:\d{2}:\d{2}").unwrap());

/// Zulassungsnummer, 9 Ziffern (Python: "Zulassungsnummer")
pub static LICENSE_NO: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\d{9}").unwrap());

/// Ausdruck-Nr., z. B. `A 007` oder `B 001` (Python: "Ausdruck")
pub static PRINTOUT_NO: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(A|B) \d{3}").unwrap());

/// Ablaufdatum, z. B. `2025/01` (Python: "Ablaufdatum")
pub static EXPIRY: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\d{4}/\d{2}").unwrap());

/// Geldbetrag mit 1–2 Nachkommastellen, z. B. `1133,00` oder `1215,4`.
///
/// Python: `\d+(.\d{1,2})` – dort matcht der unescapte Punkt *jedes* Zeichen.
/// Hier: nur `,` oder `.` als Trenner. Falls ein Format dadurch nicht mehr
/// erkannt wird, hier lockern (z. B. `[.,\s]`).
pub static MONEY: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\d+[.,]\d{1,2}").unwrap());

// ---------------------------------------------------------------------------
// Stichwörter (Python: erster Parameter `Wort`, dort als Regex übergeben,
// aber alles feste Texte -> `line.contains(KEYWORD)`)
// ---------------------------------------------------------------------------

pub const KW_LICENSE: &str = "ZULASSUNG";
pub const KW_PRINTOUT: &str = "AUSDRUCK";
pub const KW_PRINTOUT_COPY: &str = "KOPIE"; // Fallback, wenn "AUSDRUCK" fehlt
pub const KW_EXPIRY: &str = "ABLAUF";

pub const KW_SALDO_1: &str = " (1"; // Python: r" \(1"
pub const KW_SALDO_2: &str = " (2"; // Python: r" \(2"
pub const KW_STAKES: &str = "EINSAETZE";
pub const KW_WINNINGS: &str = "GEWINNE";
pub const KW_INSERTED: &str = "EINWURF";
pub const KW_PAYOUT: &str = "AUSWURF";

/// Enddatum: steht 2 Zeilen unter diesem Stichwort
pub const KW_DATE_END: &str = "KASSIERUNG VOM";
/// Anfangsdatum: steht 2 Zeilen unter diesem Stichwort ...
pub const KW_DATE_START: &str = "LETZTE KASSIERUNG";
/// ... oder, wenn es fehlt, unter diesem (Erstausdruck)
pub const KW_DATE_START_FALLBACK: &str = "INBETRIEBNAHME";
/// Zeilenversatz für beide Datumsangaben
pub const DATE_LINE_OFFSET: usize = 2;

/// Marker-Zeile (ESC K "), der Gerätetyp steht in der Zeile darunter.
/// Python: `Regex_Patterns["Geraetetyp"]` = `\x1bK"`
pub const MARKER_DEVICE_TYPE: &str = "\x1bK\"";
pub const DEVICE_TYPE_LINE_OFFSET: usize = 1;

// ---------------------------------------------------------------------------
// PDF-Ende (Python: Optionen E und S). Kandidaten in Prioritätsreihenfolge, der
// erste, der in der Datei vorkommt, gewinnt. Die Suche ignoriert Groß-/Klein-
// schreibung und Leerzeichen, "SALDO (2)" findet also auch "Saldo(2)".
// Für neue Quittungsformate hier einfach weitere Schreibweisen ergänzen.
// ---------------------------------------------------------------------------

/// PDF bis "Einsätze"
pub const PDF_END_STAKES: &[&str] = &["EINSAETZE", "EINSATZ"];
/// PDF bis "Saldo 2"
pub const PDF_END_SALDO_2: &[&str] = &["SALDO (2)", "SALDO 2"];

/// Python: `re.sub(' +', ' ', zeile)` – mehrere Leerzeichen zu einem.
pub static MULTI_SPACE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r" +").unwrap());

#[cfg(test)]
mod tests {
    use super::*;

    fn found(re: &Regex, s: &str) -> Option<String> {
        re.find(s).map(|m| m.as_str().to_string())
    }

    #[test]
    fn alle_patterns_kompilieren() {
        // Erzwingt die Initialisierung, damit ein kaputtes Pattern hier auffällt
        let _ = (
            &*DATE, &*DATE_SHORT, &*TIME, &*LICENSE_NO, &*PRINTOUT_NO, &*EXPIRY, &*MONEY,
            &*MULTI_SPACE,
        );
    }

    // Beispielzeilen stammen aus TestStaette/*.ACK
    #[test]
    fn werte_aus_testdateien() {
        assert_eq!(found(&DATE, "30.01.2023    17:16:19.2").as_deref(), Some("30.01.2023"));
        assert_eq!(found(&DATE_SHORT, "30.01.23 17:16:08").as_deref(), Some("30.01.23"));
        assert_eq!(found(&TIME, "30.01.23 17:16:08").as_deref(), Some("17:16:08"));
        assert_eq!(found(&LICENSE_NO, "ZULASSUNGS NR.:426710407").as_deref(), Some("426710407"));
        assert_eq!(found(&PRINTOUT_NO, "AUSDRUCK NR.:0018  A 018").as_deref(), Some("A 018"));
        assert_eq!(found(&PRINTOUT_NO, "AUSDRUCK NR.:0008  B 001").as_deref(), Some("B 001"));
        assert_eq!(found(&EXPIRY, "ABLAUFDATUM   :  2023/06").as_deref(), Some("2023/06"));
    }

    #[test]
    fn geld() {
        assert_eq!(found(&MONEY, "SALDO (1)      : 1133,00").as_deref(), Some("1133,00"));
        assert_eq!(found(&MONEY, "AUSWURF        - 3282,80").as_deref(), Some("3282,80"));
        assert_eq!(found(&MONEY, "EINWURF        :    0,00").as_deref(), Some("0,00"));
        assert_eq!(found(&MONEY, "Saldo 1215,4").as_deref(), Some("1215,4")); // Readme-Beispiel
    }

    #[test]
    fn stichwoerter_und_marker() {
        assert!("SALDO (1)      : 1133,00".contains(KW_SALDO_1));
        assert!("\x1bK\"".contains(MARKER_DEVICE_TYPE));
    }
}
