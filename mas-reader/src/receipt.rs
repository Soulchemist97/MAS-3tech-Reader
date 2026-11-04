use chrono::NaiveDate;
use regex::Regex;

use crate::money::Money;
use crate::patterns;

/// Eine ausgelesene Quittung. Enthält nur eigene Werte (`String`, `Money`, ...),
/// keine Referenzen auf den Dateitext, damit sie frei weitergegeben werden kann.
#[derive(Debug, Default)]
pub struct Receipt {
    pub license_no: Option<String>,
    pub printout_no: Option<String>,
    pub expiry: Option<String>,
    pub device_type: Option<String>,
    pub date_start: Option<NaiveDate>,
    pub date_end: Option<NaiveDate>,
    pub saldo_1: Option<Money>,
    pub saldo_2: Option<Money>,
    pub stakes: Option<Money>,
    pub winnings: Option<Money>,
    pub inserted: Option<Money>,
    pub payout: Option<Money>,
}

impl Receipt {
    /// Liest alle bekannten Werte aus dem Dateitext. Was nicht gefunden wird,
    /// bleibt `None` (Python: `None`-Defaults).
    pub fn parse(content: &str) -> Receipt {
        let lines: Vec<&str> = content.lines().collect();

        let money = |keyword: &str| {
            find_in_line(&lines, keyword, &patterns::MONEY).and_then(Money::parse)
        };

        // AUSDRUCK, sonst KOPIE (Python: zweiter Versuch bei None)
        let printout_no = find_in_line(&lines, patterns::KW_PRINTOUT, &patterns::PRINTOUT_NO)
            .or_else(|| find_in_line(&lines, patterns::KW_PRINTOUT_COPY, &patterns::PRINTOUT_NO));

        // Enddatum: unter "KASSIERUNG VOM"
        let date_end = date_below(&lines, patterns::KW_DATE_END);
        // Anfangsdatum: unter "LETZTE KASSIERUNG", sonst unter "INBETRIEBNAHME"
        let date_start = date_below(&lines, patterns::KW_DATE_START)
            .or_else(|| date_below(&lines, patterns::KW_DATE_START_FALLBACK));

        // Gerätetyp: Zeile nach dem ESC-Marker, Mehrfach-Leerzeichen zusammenfassen
        let device_type =
            value_at_offset(&lines, patterns::MARKER_DEVICE_TYPE, patterns::DEVICE_TYPE_LINE_OFFSET)
                .map(|l| patterns::MULTI_SPACE.replace_all(l.trim(), " ").into_owned());

        Receipt {
            license_no: find_in_line(&lines, patterns::KW_LICENSE, &patterns::LICENSE_NO)
                .map(str::to_string),
            printout_no: printout_no.map(str::to_string),
            expiry: find_in_line(&lines, patterns::KW_EXPIRY, &patterns::EXPIRY)
                .map(str::to_string),
            device_type,
            date_start,
            date_end,
            saldo_1: money(patterns::KW_SALDO_1),
            saldo_2: money(patterns::KW_SALDO_2),
            stakes: money(patterns::KW_STAKES),
            winnings: money(patterns::KW_WINNINGS),
            inserted: money(patterns::KW_INSERTED),
            payout: money(patterns::KW_PAYOUT),
        }
    }
}

/// Python: `Extract_Value` – erste Zeile mit `keyword`, darin das erste Regex-Match.
/// Hat die Zeile kein Match, wird die nächste Zeile mit `keyword` versucht.
fn find_in_line<'a>(lines: &[&'a str], keyword: &str, re: &Regex) -> Option<&'a str> {
    lines
        .iter()
        .copied()
        .filter(|l| l.contains(keyword))
        .find_map(|l| re.find(l).map(|m| m.as_str()))
}

/// Python: `Extract_OtherValue` – Zeile `offset` Zeilen unter der ersten mit `keyword`.
fn value_at_offset<'a>(lines: &[&'a str], keyword: &str, offset: usize) -> Option<&'a str> {
    let i = lines.iter().position(|l| l.contains(keyword))?;
    lines.get(i + offset).copied()
}

/// Datum (dd.mm.yyyy), das `DATE_LINE_OFFSET` Zeilen unter `keyword` steht.
fn date_below(lines: &[&str], keyword: &str) -> Option<NaiveDate> {
    let line = value_at_offset(lines, keyword, patterns::DATE_LINE_OFFSET)?;
    let text = patterns::DATE.find(line)?.as_str();
    NaiveDate::parse_from_str(text, "%d.%m.%Y").ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Liest eine Datei aus TestStaette/. `None`, wenn sie fehlt (*.ACK steht in .gitignore).
    fn fixture(name: &str) -> Option<String> {
        let path = format!("{}/../TestStaette/{name}", env!("CARGO_MANIFEST_DIR"));
        let bytes = std::fs::read(path).ok()?;
        Some(String::from_utf8_lossy(&bytes).into_owned())
    }

    fn ymd(y: i32, m: u32, d: u32) -> Option<NaiveDate> {
        NaiveDate::from_ymd_opt(y, m, d)
    }

    #[test]
    fn letzte_kassierung() {
        let Some(content) = fixture("0130_0252.ACK") else { return };
        let r = Receipt::parse(&content);
        assert_eq!(r.license_no.as_deref(), Some("426710407"));
        assert_eq!(r.printout_no.as_deref(), Some("A 018"));
        assert_eq!(r.expiry.as_deref(), Some("2023/06"));
        assert_eq!(r.device_type.as_deref(), Some("M.JAMA M C CC1"));
        assert_eq!(r.date_start, ymd(2022, 12, 30));
        assert_eq!(r.date_end, ymd(2023, 1, 30));
        assert_eq!(r.inserted, Some(Money(441580)));
        assert_eq!(r.payout, Some(Money(328280)));
        assert_eq!(r.saldo_1, Some(Money(113300)));
    }

    #[test]
    fn inbetriebnahme_statt_letzte_kassierung() {
        let Some(content) = fixture("0102_0481.ACK") else { return };
        let r = Receipt::parse(&content);
        assert_eq!(r.printout_no.as_deref(), Some("B 001"));
        assert_eq!(r.date_start, ymd(2025, 1, 2));
        assert_eq!(r.date_end, ymd(2025, 1, 2));
    }
}
