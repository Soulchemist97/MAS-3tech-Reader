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

/// Baut den neuen Dateinamen:
/// `Ort [Anfang-Ende](Ausdruck)(Zulassung).ACK`
///
/// Fehlende Werte erscheinen als `--` (Python schrieb hier `None`).
/// `extension` ohne Punkt, z. B. `"ACK"`.
pub fn new_filename(location: &str, receipt: &Receipt, extension: &str) -> String {
    let date = |d: Option<chrono::NaiveDate>| {
        d.map_or("--".to_string(), |d| d.format("%d.%m.%Y").to_string())
    };
    let printout = receipt.printout_no.as_deref().unwrap_or("--");
    let license = receipt.license_no.as_deref().unwrap_or("--");

    let name = format!(
        "{location} [{}-{}]({printout})({license})",
        date(receipt.date_start),
        date(receipt.date_end),
    );
    let name = sanitize(&name);

    if extension.is_empty() { name } else { format!("{name}.{extension}") }
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
