use std::path::Path;

use anyhow::{Result, bail};
use printpdf::{
    BuiltinFont, Mm, Op, PdfDocument, PdfFontHandle, PdfPage, PdfSaveOptions, Point, Pt, TextItem,
};

use crate::patterns;

/// Wo das PDF aufhören soll (Python: Eingabe 0 / Zahl / E / S).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdfRange {
    /// Ganze Datei (Python: 0)
    All,
    /// Nur die ersten n Zeilen (Python: Zahl)
    Lines(usize),
    /// Bis eine Zeile nach "EINSAETZE" (Python: E)
    UntilStakes,
    /// Bis eine Zeile nach "SALDO (2)" (Python: S)
    UntilSaldo2,
}

// A4-Layout mit Courier 10 pt. 60 Zeilen à 12 pt passen mit Rand auf eine Seite.
const PAGE_W: f32 = 210.0;
const PAGE_H: f32 = 297.0;
const MARGIN: f32 = 15.0;
const FONT_SIZE: f32 = 10.0;
const LINE_HEIGHT: f32 = 12.0;
const LINES_PER_PAGE: usize = 60;

/// Schneidet den Text gemäß `range` zu und bereinigt ihn für die PDF-Schrift.
///
/// Zuerst wird auf den *Rohzeilen* geschnitten (wie in Python), danach werden
/// Druckerbefehle (Zeilen, die mit einem Steuerzeichen beginnen, z. B. ESC K!)
/// entfernt und verbleibende Sonderzeichen ersetzt.
pub fn select_lines(content: &str, range: PdfRange) -> Result<Vec<String>> {
    let lines: Vec<&str> = content.lines().collect();

    let end = match range {
        PdfRange::All => lines.len(),
        PdfRange::Lines(0) => lines.len(), // 0 = alle, wie in Python
        PdfRange::Lines(n) => n.min(lines.len()),
        PdfRange::UntilStakes => end_after(&lines, patterns::KW_STAKES)?,
        PdfRange::UntilSaldo2 => end_after(&lines, patterns::KW_SALDO_2_LINE)?,
    };

    Ok(lines[..end].iter().filter_map(|l| clean_line(l)).collect())
}

/// Index hinter der Zeile mit `keyword` plus einer weiteren Zeile.
fn end_after(lines: &[&str], keyword: &str) -> Result<usize> {
    match lines.iter().position(|l| l.contains(keyword)) {
        Some(i) => Ok((i + 2).min(lines.len())),
        None => bail!("Stichwort \"{keyword}\" nicht in der Datei gefunden"),
    }
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

/// Schreibt die ausgewählten Zeilen als PDF (A4, Courier, mehrere Seiten bei Bedarf).
pub fn write_pdf(content: &str, range: PdfRange, path: &Path) -> Result<()> {
    let lines = select_lines(content, range)?;

    let title = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Quittung");
    let mut doc = PdfDocument::new(title);

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
            PdfPage::new(Mm(PAGE_W), Mm(PAGE_H), ops)
        })
        .collect();

    // Eine leere Auswahl soll trotzdem ein gültiges (leeres) PDF ergeben
    let pages = if pages.is_empty() { vec![PdfPage::new(Mm(PAGE_W), Mm(PAGE_H), vec![])] } else { pages };

    let mut warnings = Vec::new();
    let bytes = doc.with_pages(pages).save(&PdfSaveOptions::default(), &mut warnings);
    std::fs::write(path, bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Aufbau wie in den Testdateien: Befehlszeilen mit ESC, darunter die Werte
    const SAMPLE: &str = "\x11 \nMAS 3tech V3.90\n\x1bK!\nADP 00.10\n\x1bKQ\nEINWURF : 1,00\n\x1bKR\nAUSWURF - 2,00\nSALDO (2) : 3,00\n\x1bKT\nENDE";

    #[test]
    fn alle_zeilen_ohne_druckerbefehle() {
        let lines = select_lines(SAMPLE, PdfRange::All).unwrap();
        assert_eq!(lines, ["MAS 3tech V3.90", "ADP 00.10", "EINWURF : 1,00", "AUSWURF - 2,00", "SALDO (2) : 3,00", "ENDE"]);
    }

    #[test]
    fn erste_n_rohzeilen() {
        // 4 Rohzeilen: "\x11 ", "MAS..", "ESC K!", "ADP.." -> 2 bleiben sichtbar
        let lines = select_lines(SAMPLE, PdfRange::Lines(4)).unwrap();
        assert_eq!(lines, ["MAS 3tech V3.90", "ADP 00.10"]);
        // Lines(0) = alle
        assert_eq!(select_lines(SAMPLE, PdfRange::Lines(0)).unwrap().len(), 6);
        // n grösser als die Datei schneidet nicht ins Leere
        assert_eq!(select_lines(SAMPLE, PdfRange::Lines(999)).unwrap().len(), 6);
    }

    #[test]
    fn bis_saldo_2_und_bis_einsaetze() {
        let lines = select_lines(SAMPLE, PdfRange::UntilSaldo2).unwrap();
        assert_eq!(lines.last().map(String::as_str), Some("SALDO (2) : 3,00")); // Folgezeile ist ein Befehl
        // EINSAETZE kommt in SAMPLE nicht vor -> Fehler statt Absturz
        assert!(select_lines(SAMPLE, PdfRange::UntilStakes).is_err());
    }

    #[test]
    fn sonderzeichen_werden_ersetzt() {
        assert_eq!(clean_line("Gewinn \u{FFFD} äöü").as_deref(), Some("Gewinn ? äöü"));
    }

    #[test]
    fn schreibt_pdf_datei() {
        let path = std::env::temp_dir().join(format!("mas_pdf_test_{}.pdf", std::process::id()));
        write_pdf(SAMPLE, PdfRange::All, &path).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        assert!(bytes.starts_with(b"%PDF"));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn mehrere_seiten() {
        let long: String = (0..150).map(|i| format!("Zeile {i}\n")).collect();
        let path = std::env::temp_dir().join(format!("mas_pdf_pages_{}.pdf", std::process::id()));
        write_pdf(&long, PdfRange::All, &path).unwrap();
        assert!(std::fs::metadata(&path).unwrap().len() > 0);
        std::fs::remove_file(path).ok();
    }
}

#[cfg(test)]
mod fixture_tests {
    use super::*;

    #[test]
    fn echte_datei_bis_saldo_2() {
        let path = format!("{}/../TestStaette/0130_0252.ACK", env!("CARGO_MANIFEST_DIR"));
        let Ok(bytes) = std::fs::read(path) else { return }; // *.ACK ist in .gitignore
        let content = String::from_utf8_lossy(&bytes);

        let all = select_lines(&content, PdfRange::All).unwrap();
        assert!(all.iter().all(|l| !l.chars().any(|c| c.is_control())));
        assert!(all.iter().any(|l| l.contains("ZULASSUNGS NR.:426710407")));

        // Bei dieser Datei existiert "SALDO (2)" evtl. nicht -> dann Fehler, kein Absturz
        match select_lines(&content, PdfRange::UntilSaldo2) {
            Ok(cut) => assert!(cut.len() <= all.len()),
            Err(e) => assert!(e.to_string().contains("SALDO (2)")),
        }
    }
}
