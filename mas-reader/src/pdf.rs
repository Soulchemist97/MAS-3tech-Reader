use std::path::Path;

use anyhow::Result;
use printpdf::{
    BuiltinFont, Mm, Op, PdfDocument, PdfFontHandle, PdfPage, PdfSaveOptions, Point, Pt, TextItem,
};

use crate::patterns;

/// Wo das PDF aufhören soll (Python: Eingabe 0 / Zahl / E / S).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PdfRange {
    /// Ganze Datei (Python: 0)
    All,
    /// Nur die ersten n Zeilen (Python: Zahl)
    Lines(usize),
    /// Bis eine Zeile nach "EINSAETZE" (Python: E), Kandidaten: `patterns::PDF_END_STAKES`
    UntilStakes,
    /// Bis eine Zeile nach "SALDO (2)" (Python: S), Kandidaten: `patterns::PDF_END_SALDO_2`
    UntilSaldo2,
    /// Bis zu einem frei gewählten Stichwort. `occurrence` ist die wievielte Zeile mit dem
    /// Stichwort gemeint ist (1 = erste, 0 zählt auch als erste); `offset` verschiebt das
    /// Ende um so viele sichtbare Zeilen (+ danach, - davor, 0 = Stichwortzeile ist die letzte).
    UntilKeyword { word: String, occurrence: usize, offset: i32 },
}

/// Ergebnis der Zeilenauswahl. `warning` ist gesetzt, wenn das Ende nicht
/// gefunden wurde und deshalb die ganze Datei genommen wurde.
#[derive(Debug)]
pub struct Selection {
    pub lines: Vec<String>,
    pub warning: Option<String>,
}

// Courier 10 pt, Seitenhöhe A4. 60 Zeilen à 12 pt passen mit Rand auf eine Seite.
// Die Breite richtet sich nach der längsten Zeile (siehe `page_width_mm`).
const PAGE_H: f32 = 297.0;
const MARGIN: f32 = 10.0;
const FONT_SIZE: f32 = 10.0;
const LINE_HEIGHT: f32 = 12.0;
const LINES_PER_PAGE: usize = 60;
/// Courier ist Festbreite: jedes Zeichen ist 0,6 × Schriftgröße breit (pt → mm).
const CHAR_W_MM: f32 = FONT_SIZE * 0.6 * 25.4 / 72.0;
const MIN_PAGE_W: f32 = 60.0;

/// Seitenbreite so, dass die längste Zeile genau hineinpasst (plus Rand und
/// ein Zeichen Luft). `ShowText` bricht nicht um, zu schmal würde abschneiden.
fn page_width_mm(lines: &[String]) -> f32 {
    let longest = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    ((longest + 1) as f32 * CHAR_W_MM + 2.0 * MARGIN).max(MIN_PAGE_W)
}

/// Schneidet den Text gemäß `range` zu und bereinigt ihn für die PDF-Schrift.
///
/// Zuerst wird auf den *Rohzeilen* geschnitten (wie in Python), danach werden
/// Druckerbefehle (Zeilen, die mit einem Steuerzeichen beginnen, z. B. ESC K!)
/// entfernt und verbleibende Sonderzeichen ersetzt.
///
/// Wird das Ende-Stichwort nicht gefunden, kommt die ganze Datei mit einer
/// Warnung zurück, statt dass die Datei ohne PDF bleibt.
pub fn select_lines(content: &str, range: &PdfRange) -> Selection {
    let lines: Vec<&str> = content.lines().collect();

    if let PdfRange::UntilKeyword { word, occurrence, offset } = range {
        return keyword_selection(&lines, word, *occurrence, *offset);
    }

    let (end, warning) = match range {
        PdfRange::All | PdfRange::Lines(0) => (lines.len(), None), // 0 = alle, wie in Python
        PdfRange::Lines(n) => ((*n).min(lines.len()), None),
        PdfRange::UntilStakes => end_after(&lines, patterns::PDF_END_STAKES),
        PdfRange::UntilSaldo2 => end_after(&lines, patterns::PDF_END_SALDO_2),
        PdfRange::UntilKeyword { .. } => unreachable!("oben behandelt"),
    };

    Selection { lines: lines[..end].iter().filter_map(|l| clean_line(l)).collect(), warning }
}

/// Stichwortmodus: gezählt wird auf den sichtbaren Zeilen (wie in der Ansicht nummeriert),
/// nicht auf den Rohzeilen mit Druckerbefehlen. Ohne die gewünschte Nennung: ganze Datei + Warnung.
fn keyword_selection(lines: &[&str], word: &str, occurrence: usize, offset: i32) -> Selection {
    let visible: Vec<String> = lines.iter().filter_map(|l| clean_line(l)).collect();
    let needle = normalize(word);
    let nth = occurrence.max(1);

    let hits: Vec<usize> = if needle.is_empty() {
        Vec::new()
    } else {
        visible.iter().enumerate().filter(|(_, l)| normalize(l).contains(&needle)).map(|(i, _)| i).collect()
    };

    match hits.get(nth - 1) {
        Some(&i) => {
            let end = (i as i64 + 1 + i64::from(offset)).clamp(0, visible.len() as i64) as usize;
            Selection { lines: visible[..end].to_vec(), warning: None }
        }
        None => {
            let warning = match hits.len() {
                0 => format!("Kein Stichwort gefunden ({word}), ganze Datei verwendet"),
                n => format!("Stichwort \"{word}\" nur {n}-mal gefunden, {nth}. Nennung fehlt, ganze Datei verwendet"),
            };
            Selection { lines: visible, warning: Some(warning) }
        }
    }
}

/// Großbuchstaben ohne Leerzeichen und Steuerzeichen: "Saldo  (2)" -> "SALDO(2)".
fn normalize(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_whitespace() && !c.is_control())
        .flat_map(char::to_uppercase)
        .collect()
}

/// Index hinter der Zeile mit dem ersten gefundenen Kandidaten plus einer
/// weiteren Zeile. Kandidaten werden in der angegebenen Reihenfolge probiert.
/// Ohne Treffer: Dateiende und eine Warnung.
fn end_after(lines: &[&str], candidates: &[&str]) -> (usize, Option<String>) {
    let normalized: Vec<String> = lines.iter().map(|l| normalize(l)).collect();

    for candidate in candidates {
        let needle = normalize(candidate);
        if needle.is_empty() {
            continue;
        }
        if let Some(i) = normalized.iter().position(|l| l.contains(&needle)) {
            return ((i + 2).min(lines.len()), None);
        }
    }

    let tried = candidates.join(", ");
    (lines.len(), Some(format!("Kein Stichwort gefunden ({tried}), ganze Datei verwendet")))
}

/// `None` für Druckerbefehle, sonst die Zeile ohne Steuerzeichen und Nicht-Latin-1-Zeichen.
fn clean_line(line: &str) -> Option<String> {
    if line.chars().next().is_some_and(|c| c.is_control() && c != '\t') {
        return None;
    }
    let cleaned = line
        .chars()
        .filter(|c| !c.is_control())
        .map(|c| if (c as u32) < 256 { c } else { '?' })
        .collect();
    Some(cleaned)
}

/// Schreibt die ausgewählten Zeilen als PDF (Courier, mehrere Seiten bei Bedarf).
/// Gibt eine Warnung zurück, wenn das Ende nicht gefunden wurde (siehe `select_lines`).
pub fn write_pdf(content: &str, range: &PdfRange, path: &Path) -> Result<Option<String>> {
    let Selection { lines, warning } = select_lines(content, range);

    let title = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Quittung");
    let mut doc = PdfDocument::new(title);

    let page_w = page_width_mm(&lines);

    let pages: Vec<PdfPage> = lines
        .chunks(LINES_PER_PAGE)
        .map(|chunk| {
            let mut ops = vec![
                Op::StartTextSection,
                Op::SetTextCursor { pos: Point::new(Mm(MARGIN), Mm(PAGE_H - MARGIN)) },
                Op::SetFont { font: PdfFontHandle::Builtin(BuiltinFont::Courier), size: Pt(FONT_SIZE) },
                Op::SetLineHeight { lh: Pt(LINE_HEIGHT) },
            ];
            for line in chunk {
                ops.push(Op::ShowText { items: vec![TextItem::Text(line.clone())] });
                ops.push(Op::AddLineBreak);
            }
            ops.push(Op::EndTextSection);
            PdfPage::new(Mm(page_w), Mm(PAGE_H), ops)
        })
        .collect();

    // Eine leere Auswahl soll trotzdem ein gültiges (leeres) PDF ergeben
    let pages = if pages.is_empty() { vec![PdfPage::new(Mm(page_w), Mm(PAGE_H), vec![])] } else { pages };

    let mut warnings = Vec::new();
    let bytes = doc.with_pages(pages).save(&PdfSaveOptions::default(), &mut warnings);
    std::fs::write(path, bytes)?;
    Ok(warning)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Aufbau wie in den Testdateien: Befehlszeilen mit ESC, darunter die Werte
    const SAMPLE: &str = "\x11 \nMAS 3tech V3.90\n\x1bK!\nADP 00.10\n\x1bKQ\nEINWURF : 1,00\n\x1bKR\nAUSWURF - 2,00\nSALDO (2) : 3,00\n\x1bKT\nENDE";

    fn lines(content: &str, range: PdfRange) -> Vec<String> {
        select_lines(content, &range).lines
    }

    #[test]
    fn alle_zeilen_ohne_druckerbefehle() {
        let sel = select_lines(SAMPLE, &PdfRange::All);
        assert_eq!(sel.lines, ["MAS 3tech V3.90", "ADP 00.10", "EINWURF : 1,00", "AUSWURF - 2,00", "SALDO (2) : 3,00", "ENDE"]);
        assert!(sel.warning.is_none());
    }

    #[test]
    fn erste_n_rohzeilen() {
        // 4 Rohzeilen: "\x11 ", "MAS..", "ESC K!", "ADP.." -> 2 bleiben sichtbar
        assert_eq!(lines(SAMPLE, PdfRange::Lines(4)), ["MAS 3tech V3.90", "ADP 00.10"]);
        assert_eq!(lines(SAMPLE, PdfRange::Lines(0)).len(), 6); // 0 = alle
        assert_eq!(lines(SAMPLE, PdfRange::Lines(999)).len(), 6); // n zu gross schneidet nicht ins Leere
    }

    #[test]
    fn bis_saldo_2() {
        let sel = select_lines(SAMPLE, &PdfRange::UntilSaldo2);
        assert_eq!(sel.lines.last().map(String::as_str), Some("SALDO (2) : 3,00")); // Folgezeile ist ein Befehl
        assert!(sel.warning.is_none());
    }

    #[test]
    fn stichwort_ignoriert_gross_klein_und_leerzeichen() {
        for variante in ["Saldo(2) : 3,00", "SALDO   (2)  3,00", "saldo ( 2 ) 3,00"] {
            let text = format!("Kopf\n{variante}\nEnde");
            let sel = select_lines(&text, &PdfRange::UntilSaldo2);
            assert!(sel.warning.is_none(), "{variante}");
            assert_eq!(sel.lines.len(), 3, "{variante}"); // Kopf + Zeile + 1 Folgezeile
        }
    }

    #[test]
    fn zweiter_kandidat_wird_probiert() {
        // "SALDO 2" ist der zweite Kandidat für Saldo 2
        let sel = select_lines("Kopf\nSALDO 2 : 3,00\nEnde\nMehr", &PdfRange::UntilSaldo2);
        assert!(sel.warning.is_none());
        assert_eq!(sel.lines, ["Kopf", "SALDO 2 : 3,00", "Ende"]);
    }

    #[test]
    fn eigenes_stichwort() {
        let kw = |word: &str, occurrence, offset| PdfRange::UntilKeyword { word: word.into(), occurrence, offset };
        let sel = select_lines(SAMPLE, &kw("auswurf", 1, 1));
        assert!(sel.warning.is_none());
        // Stichwortzeile + eine Folgezeile (SALDO), danach ist Schluss (ENDE fehlt)
        assert_eq!(sel.lines.last().map(String::as_str), Some("SALDO (2) : 3,00"));
        assert!(!sel.lines.iter().any(|l| l == "ENDE"));
    }

    #[test]
    fn stichwort_nennung_und_zeilenversatz() {
        let kw = |word: &str, occurrence, offset| PdfRange::UntilKeyword { word: word.into(), occurrence, offset };
        let text = "A\nSALDO x\n\x1bK!\nB\nSALDO y\nC\nD";
        // 1. Nennung, Stichwortzeile ist die letzte
        assert_eq!(lines(text, kw("saldo", 1, 0)), ["A", "SALDO x"]);
        // 2. Nennung + 1 Zeile danach; Druckerbefehle zählen nicht mit
        assert_eq!(lines(text, kw("saldo", 2, 1)), ["A", "SALDO x", "B", "SALDO y", "C"]);
        // minus: davor aufhören
        assert_eq!(lines(text, kw("saldo", 2, -1)), ["A", "SALDO x", "B"]);
        // Versatz über die Grenzen schneidet nicht ins Leere
        assert_eq!(lines(text, kw("saldo", 1, 99)).len(), 6);
        assert!(lines(text, kw("saldo", 1, -99)).is_empty());
        // 3. Nennung gibt es nicht
        let sel = select_lines(text, &kw("saldo", 3, 0));
        assert_eq!(sel.lines.len(), 6);
        assert!(sel.warning.is_some_and(|w| w.contains("2-mal")));
    }

    #[test]
    fn ohne_treffer_ganze_datei_mit_warnung() {
        // EINSAETZE kommt in SAMPLE nicht vor
        let sel = select_lines(SAMPLE, &PdfRange::UntilStakes);
        assert_eq!(sel.lines.len(), 6);
        assert!(sel.warning.as_deref().is_some_and(|w| w.contains("EINSAETZE")));

        // leeres eigenes Stichwort darf nicht "überall" treffen
        let sel = select_lines(SAMPLE, &PdfRange::UntilKeyword { word: "  ".into(), occurrence: 1, offset: 0 });
        assert!(sel.warning.is_some());
    }

    #[test]
    fn seitenbreite_folgt_der_laengsten_zeile() {
        let zeilen = |n: usize| vec!["x".repeat(24), "x".repeat(n)];
        // 24 Zeichen: (24+1) * 2,117 mm + 2 * 10 mm Rand ≈ 72,9 mm
        let w24 = page_width_mm(&zeilen(24));
        assert!((w24 - 72.9).abs() < 0.2, "{w24}");
        // längere Zeilen -> breitere Seite, nie schmaler als nötig
        assert!(page_width_mm(&zeilen(100)) > 220.0);
        // leere Auswahl -> Mindestbreite
        assert_eq!(page_width_mm(&[]), MIN_PAGE_W);
    }

    #[test]
    fn sonderzeichen_werden_ersetzt() {
        assert_eq!(clean_line("Gewinn \u{FFFD} äöü").as_deref(), Some("Gewinn ? äöü"));
    }

    #[test]
    fn schreibt_pdf_datei() {
        let path = std::env::temp_dir().join(format!("mas_pdf_test_{}.pdf", std::process::id()));
        let warning = write_pdf(SAMPLE, &PdfRange::UntilStakes, &path).unwrap();
        assert!(warning.is_some()); // Stichwort fehlt, PDF wird trotzdem geschrieben
        let bytes = std::fs::read(&path).unwrap();
        assert!(bytes.starts_with(b"%PDF"));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn mehrere_seiten() {
        let long: String = (0..150).map(|i| format!("Zeile {i}\n")).collect();
        let path = std::env::temp_dir().join(format!("mas_pdf_pages_{}.pdf", std::process::id()));
        write_pdf(&long, &PdfRange::All, &path).unwrap();
        assert!(std::fs::metadata(&path).unwrap().len() > 0);
        std::fs::remove_file(path).ok();
    }
}

#[cfg(test)]
mod fixture_tests {
    use super::*;

    #[test]
    fn echte_datei() {
        let path = format!("{}/../TestStaette/0130_0252.ACK", env!("CARGO_MANIFEST_DIR"));
        let Ok(bytes) = std::fs::read(path) else { return }; // *.ACK ist in .gitignore
        let content = String::from_utf8_lossy(&bytes);

        let all = select_lines(&content, &PdfRange::All).lines;
        assert!(all.iter().all(|l| !l.chars().any(|c| c.is_control())));
        assert!(all.iter().any(|l| l.contains("ZULASSUNGS NR.:426710407")));

        // Egal ob "SALDO (2)" in der Datei steht: nie mehr Zeilen als die ganze Datei, nie ein Fehler
        let cut = select_lines(&content, &PdfRange::UntilSaldo2);
        assert!(cut.lines.len() <= all.len());
        if cut.warning.is_some() {
            assert_eq!(cut.lines.len(), all.len());
        }
    }
}
