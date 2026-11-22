use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::receipt::Receipt;

/// Kopieren (Original bleibt) oder Verschieben (Original verschwindet).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Copy,
    Move,
}

/// Standard-Vorlage für Dateinamen, ergibt `Ort [Anfang-Ende](Ausdruck)(Zulassung)`
/// wie im Python-Skript.
pub const DEFAULT_TEMPLATE: &str = "{ort} [{anfang}-{ende}]({ausdruck})({zulassung})";

/// Platzhalter für Dateinamen-Vorlagen: (Name ohne Klammern, Beschreibung).
/// In der Vorlage als `{name}` schreiben, Groß-/Kleinschreibung egal.
pub const PLACEHOLDERS: &[(&str, &str)] = &[
    ("ort", "Aufstellort (Ordnername)"),
    ("anfang", "Anfangsdatum (TT.MM.JJJJ)"),
    ("ende", "Enddatum (TT.MM.JJJJ)"),
    ("anfang_iso", "Anfangsdatum (JJJJ-MM-TT, sortierbar)"),
    ("ende_iso", "Enddatum (JJJJ-MM-TT, sortierbar)"),
    ("ausdruck", "Ausdruck-Nr. (z. B. A 018)"),
    ("zulassung", "Zulassungsnummer"),
    ("geraetetyp", "Gerätetyp"),
    ("ablauf", "Ablaufdatum (JJJJ/MM)"),
    ("saldo1", "Saldo 1"),
    ("saldo2", "Saldo 2"),
];

/// Wert für einen Platzhalter. Fehlende Werte erscheinen als `--` (Python schrieb `None`).
fn placeholder_value(name: &str, location: &str, r: &Receipt) -> Option<String> {
    let text = |v: &Option<String>| v.clone().unwrap_or_else(|| "--".to_string());
    let date = |d: Option<chrono::NaiveDate>, fmt: &str| {
        d.map_or("--".to_string(), |d| d.format(fmt).to_string())
    };
    let money = |m: Option<crate::money::Money>| m.map_or("--".to_string(), |m| m.to_string());

    Some(match name {
        "ort" => location.to_string(),
        "anfang" => date(r.date_start, "%d.%m.%Y"),
        "ende" => date(r.date_end, "%d.%m.%Y"),
        "anfang_iso" => date(r.date_start, "%Y-%m-%d"),
        "ende_iso" => date(r.date_end, "%Y-%m-%d"),
        "ausdruck" => text(&r.printout_no),
        "zulassung" => text(&r.license_no),
        "geraetetyp" => text(&r.device_type),
        "ablauf" => text(&r.expiry),
        "saldo1" => money(r.saldo_1),
        "saldo2" => money(r.saldo_2),
        _ => return None,
    })
}

/// Setzt eine Vorlage wie `{ort} [{anfang}-{ende}]` für eine Quittung zusammen
/// (ohne Dateiendung). Das Ergebnis ist für Dateisysteme bereinigt.
///
/// Fehler (für die Live-Vorschau in den Einstellungen): unbekannter Platzhalter,
/// nicht geschlossene Klammer oder leerer Name.
pub fn render_template(template: &str, location: &str, receipt: &Receipt) -> Result<String, String> {
    let mut out = String::new();
    let mut rest = template;

    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let end = after.find('}').ok_or("Klammer { ist nicht mit } geschlossen")?;
        let name = after[..end].trim().to_lowercase();
        let value = placeholder_value(&name, location, receipt)
            .ok_or_else(|| format!("Unbekannter Platzhalter {{{}}}", after[..end].trim()))?;
        out.push_str(&value);
        rest = &after[end + 1..];
    }
    out.push_str(rest);

    let name = sanitize(out.trim());
    if name.is_empty() {
        return Err("Der Dateiname wäre leer".to_string());
    }
    Ok(name)
}

/// Wie `render_template`, hängt aber die Endung an (`extension` ohne Punkt, z. B. `"ACK"`).
pub fn new_filename_with(
    template: &str,
    location: &str,
    receipt: &Receipt,
    extension: &str,
) -> Result<String, String> {
    let name = render_template(template, location, receipt)?;
    Ok(if extension.is_empty() { name } else { format!("{name}.{extension}") })
}

/// Baut den neuen Dateinamen mit der Standard-Vorlage:
/// `Ort [Anfang-Ende](Ausdruck)(Zulassung).ACK`
pub fn new_filename(location: &str, receipt: &Receipt, extension: &str) -> String {
    new_filename_with(DEFAULT_TEMPLATE, location, receipt, extension)
        .expect("die Standard-Vorlage ist gültig")
}

/// Ersetzt Zeichen, die in Dateinamen nicht erlaubt sind (Windows und macOS).
fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| if "/\\:*?\"<>|".contains(c) || c.is_control() { '_' } else { c })
        .collect()
}

/// Kopiert oder verschiebt `source` als `new_name` in `out_dir` (wird bei Bedarf angelegt).
/// Eine vorhandene Zieldatei wird **nie überschrieben**, stattdessen gibt es einen
/// `AlreadyExists`-Fehler (Python: "doppelt").
pub fn organize(source: &Path, out_dir: &Path, new_name: &str, mode: Mode) -> io::Result<PathBuf> {
    fs::create_dir_all(out_dir)?;
    let target = out_dir.join(new_name);

    if target.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("{} existiert bereits", target.display()),
        ));
    }

    match mode {
        Mode::Copy => {
            fs::copy(source, &target)?;
        }
        Mode::Move => {
            // rename klappt nur auf demselben Laufwerk, sonst kopieren und löschen
            if fs::rename(source, &target).is_err() {
                fs::copy(source, &target)?;
                fs::remove_file(source)?;
            }
        }
    }
    Ok(target)
}

/// Standard-Zielordner zum Umbenennen: `<Ordner>/Output/<Ort>`. Liegt der geladene Ordner
/// schon in einem Ordner `Output` (oder ist selbst einer), wird dieser weiterverwendet,
/// sonst entstünde `Output/Ort/Output/Ort`.
pub fn default_output_dir(dir: &Path, location: &str) -> PathBuf {
    let is_output = |p: &Path| p.file_name().is_some_and(|n| n.eq_ignore_ascii_case("output"));
    match dir.ancestors().find(|p| is_output(p)) {
        Some(base) if is_output(dir) && location.eq_ignore_ascii_case("output") => base.to_path_buf(),
        Some(base) => base.join(location),
        None => dir.join("Output").join(location),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn receipt() -> Receipt {
        Receipt {
            license_no: Some("426710407".into()),
            printout_no: Some("A 018".into()),
            date_start: NaiveDate::from_ymd_opt(2022, 12, 30),
            date_end: NaiveDate::from_ymd_opt(2023, 1, 30),
            ..Default::default()
        }
    }

    #[test]
    fn dateiname_wie_in_python() {
        assert_eq!(
            new_filename("Bar", &receipt(), "ACK"),
            "Bar [30.12.2022-30.01.2023](A 018)(426710407).ACK"
        );
    }

    #[test]
    fn fehlende_werte_und_verbotene_zeichen() {
        assert_eq!(
            new_filename("Bar/Keller", &Receipt::default(), "ACK"),
            "Bar_Keller [-----](--)(--).ACK"
        );
    }

    #[test]
    fn standardvorlage_ergibt_den_alten_namen() {
        let r = receipt();
        assert_eq!(
            new_filename_with(DEFAULT_TEMPLATE, "Bar", &r, "ACK").unwrap(),
            new_filename("Bar", &r, "ACK")
        );
        // auch ohne Daten und mit Sonderzeichen darf die Standardvorlage nie fehlschlagen
        let _ = new_filename("///", &Receipt::default(), "");
    }

    #[test]
    fn eigene_vorlage_und_iso_datum() {
        let name = new_filename_with("{ende_iso}_{Ort}_{ausdruck}", "Bar", &receipt(), "pdf").unwrap();
        assert_eq!(name, "2023-01-30_Bar_A 018.pdf"); // {Ort}: Groß-/Kleinschreibung egal
    }

    #[test]
    fn saldo_und_ohne_endung() {
        let r = Receipt { saldo_1: Some(crate::money::Money(113300)), ..Default::default() };
        assert_eq!(render_template("S1 {saldo1}, S2 {saldo2}", "Bar", &r).unwrap(), "S1 1133,00, S2 --");
        assert_eq!(new_filename_with("{ort}", "Bar", &r, "").unwrap(), "Bar");
    }

    #[test]
    fn output_ordner_wird_nicht_verschachtelt() {
        let p = Path::new;
        assert_eq!(default_output_dir(p("/a/Bar"), "Bar"), p("/a/Bar/Output/Bar"));
        assert_eq!(default_output_dir(p("/a/Output/Bar"), "Bar"), p("/a/Output/Bar"));
        assert_eq!(default_output_dir(p("/a/Output/Bar"), "Neu"), p("/a/Output/Neu"));
        assert_eq!(default_output_dir(p("/a/Output/Bar/Keller"), "Bar"), p("/a/Output/Bar"));
        assert_eq!(default_output_dir(p("/a/Output"), "Output"), p("/a/Output"));
    }

    #[test]
    fn vorlagenfehler_werden_gemeldet() {
        let r = receipt();
        let err = render_template("{ort} {quatsch}", "Bar", &r).unwrap_err();
        assert!(err.contains("quatsch"), "{err}");
        assert!(render_template("{ort", "Bar", &r).is_err());
        assert!(render_template("   ", "Bar", &r).is_err()); // leerer Name
    }

    #[test]
    fn alle_platzhalter_sind_auswertbar() {
        for (name, _) in PLACEHOLDERS {
            assert!(placeholder_value(name, "Bar", &Receipt::default()).is_some(), "{name}");
        }
    }

    #[test]
    fn kopieren_ueberschreibt_nicht() {
        let dir = std::env::temp_dir().join(format!("mas_organize_test_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let src = dir.join("original.ACK");
        fs::write(&src, b"inhalt").unwrap();
        let out = dir.join("Output");

        // erstes Kopieren klappt, Original bleibt
        let target = organize(&src, &out, "neu.ACK", Mode::Copy).unwrap();
        assert!(target.exists() && src.exists());

        // zweites Mal: Fehler statt Überschreiben
        let err = organize(&src, &out, "neu.ACK", Mode::Copy).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::AlreadyExists);

        // Verschieben: Original ist danach weg
        organize(&src, &out, "verschoben.ACK", Mode::Move).unwrap();
        assert!(!src.exists());

        fs::remove_dir_all(&dir).ok();
    }
}
