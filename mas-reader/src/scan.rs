use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::receipt::Receipt;

/// Eine eingelesene Quittungsdatei.
#[derive(Debug)]
pub struct ScannedFile {
    pub path: PathBuf,
    /// Rohtext inklusive Steuerzeichen (ESC), die Patterns brauchen sie.
    pub content: String,
    pub receipt: Receipt,
}

/// Ergebnis eines Ordner-Scans. Dateien, die nicht lesbar sind, landen als
/// Text in `errors`, der Rest wird trotzdem verarbeitet.
#[derive(Debug)]
pub struct ScanResult {
    /// Aufstellort = Name des Ordners (Python: `Pfad.split("\\")[-1]`)
    pub location: String,
    pub files: Vec<ScannedFile>,
    pub errors: Vec<String>,
}

/// Liest eine Datei als Text. Die Dateien sind kein gültiges UTF-8, daher
/// werden ungültige Bytes zu '�'. Steuerzeichen (ESC) bleiben erhalten.
pub fn read_content(path: &Path) -> io::Result<String> {
    let bytes = fs::read(path)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// Quittungen sind `.ACK`-Dateien; manche Quellen liefern dieselbe Struktur als `.txt`.
fn is_receipt(path: &Path) -> bool {
    path.is_file()
        && path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("ACK") || ext.eq_ignore_ascii_case("txt"))
}

/// Liest alle `.ACK`- und `.txt`-Dateien direkt in `dir` (keine Unterordner), sortiert nach Dateiname.
/// Fehler gibt es nur, wenn der Ordner selbst nicht gelesen werden kann.
pub fn scan_folder(dir: &Path) -> io::Result<ScanResult> {
    let mut paths = Vec::new();
    let mut errors = Vec::new();

    for entry in fs::read_dir(dir)? {
        match entry {
            Ok(e) if is_receipt(&e.path()) => paths.push(e.path()),
            Ok(_) => {} // Unterordner und fremde Dateien überspringen
            Err(e) => errors.push(format!("Eintrag nicht lesbar: {e}")),
        }
    }
    paths.sort();

    let mut files = Vec::new();
    for path in paths {
        match read_content(&path) {
            Ok(content) => {
                let receipt = Receipt::parse(&content);
                files.push(ScannedFile { path, content, receipt });
            }
            Err(e) => errors.push(format!("{}: {e}", path.display())),
        }
    }

    let location = dir.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    Ok(ScanResult { location, files, errors })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nur_ack_und_txt_dateien_sortiert_ohne_unterordner() {
        let dir = std::env::temp_dir().join(format!("mas_scan_test_{}", std::process::id()));
        let ort = dir.join("Bar");
        fs::create_dir_all(ort.join("unterordner.ACK")).unwrap(); // Ordner mit .ACK im Namen
        fs::write(ort.join("b.ACK"), "ZULASSUNGS NR.:111222333").unwrap();
        fs::write(ort.join("a.ack"), "ZULASSUNGS NR.:444555666").unwrap(); // Endung klein
        fs::write(ort.join("c.TXT"), "ZULASSUNGS NR.:777888999").unwrap();
        fs::write(ort.join("notiz.md"), "egal").unwrap();

        let result = scan_folder(&ort).unwrap();
        assert_eq!(result.location, "Bar");
        assert!(result.errors.is_empty());
        let names: Vec<_> = result.files.iter().map(|f| f.path.file_name().unwrap().to_str().unwrap()).collect();
        assert_eq!(names, ["a.ack", "b.ACK", "c.TXT"]);
        assert_eq!(result.files[0].receipt.license_no.as_deref(), Some("444555666"));

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn fehlender_ordner_ist_ein_fehler() {
        assert!(scan_folder(Path::new("/gibt/es/nicht")).is_err());
    }

    #[test]
    fn testdateien() {
        let dir = PathBuf::from(format!("{}/../TestStaette", env!("CARGO_MANIFEST_DIR")));
        let Ok(result) = scan_folder(&dir) else { return }; // *.ACK ist in .gitignore
        if result.files.is_empty() {
            return;
        }
        assert_eq!(result.location, "TestStaette");
        assert!(result.files.iter().all(|f| f.receipt.license_no.is_some()));
    }
}
