//! Befehle, die das Frontend aufruft. Die Logik steckt in der Lib `mas_reader`,
//! hier wird nur übersetzt (Zustand halten, Eingaben prüfen, Ergebnisse als JSON).
//!
//! Alle Befehle sind `async` und rechnen in `spawn_blocking`: synchrone Befehle
//! liefen im Hauptthread und würden das Fenster beim Lesen vieler Dateien einfrieren.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use mas_reader::export;
use mas_reader::organize::{self, Mode};
use mas_reader::pdf::{self, PdfRange};
use mas_reader::receipt::Receipt;
use mas_reader::scan::{self, ScannedFile};
use serde::{Deserialize, Serialize};
use tauri::State;

/// Die App-Konfiguration (`tauri.conf.json`, Berechtigungen). `generate_context!` darf nur
/// einmal im Programm vorkommen, deshalb nutzen `main` und die IPC-Tests diese Funktion.
pub fn context<R: tauri::Runtime>() -> tauri::Context<R> {
    tauri::generate_context!()
}

/// Alle Befehle, die das Frontend aufrufen darf (von `main.rs` und den IPC-Tests genutzt).
pub fn handler<R: tauri::Runtime>() -> impl Fn(tauri::ipc::Invoke<R>) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        scan_folder,
        initial_folder,
        receipt_text,
        template_info,
        preview_filename,
        export_excel,
        export_pdfs,
        organize_files,
    ]
}

// ---------------------------------------------------------------------------
// Zustand: die eingelesenen Dateien bleiben im Rust-Speicher, das Frontend
// arbeitet nur mit Zeilen-Indizes.
// ---------------------------------------------------------------------------

#[derive(Default)]
struct Loaded {
    dir: PathBuf,
    location: String,
    files: Vec<ScannedFile>,
}

#[derive(Default)]
pub struct AppState(Arc<Mutex<Loaded>>);

/// Führt `f` mit dem Zustand auf einem Worker-Thread aus.
async fn with_loaded<T, F>(state: &State<'_, AppState>, f: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce(&mut Loaded) -> Result<T, String> + Send + 'static,
{
    let shared = Arc::clone(&state.0);
    tauri::async_runtime::spawn_blocking(move || {
        let mut guard = shared.lock().map_err(|_| "Interner Fehler (Sperre)".to_string())?;
        f(&mut guard)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Wählt Dateien nach Indizes aus (`None` = alle) und prüft die Grenzen.
fn select<'a>(files: &'a [ScannedFile], indices: &Option<Vec<usize>>) -> Result<Vec<&'a ScannedFile>, String> {
    match indices {
        None => Ok(files.iter().collect()),
        Some(list) => list
            .iter()
            .map(|&i| files.get(i).ok_or_else(|| format!("Quittung {i} gibt es nicht")))
            .collect(),
    }
}

fn file_ext(path: &Path) -> &str {
    path.extension().and_then(|e| e.to_str()).unwrap_or("")
}

// ---------------------------------------------------------------------------
// Tabelle: dieselben Spalten wie die Excel-Datei (`export::HEADERS`)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct Column {
    title: &'static str,
    /// "text", "date" oder "money" (Sortierung und Ausrichtung im Frontend)
    kind: &'static str,
}

/// Art jeder Spalte, in der Reihenfolge von `export::HEADERS`.
const KINDS: [&str; 12] = [
    "text", "text", "text", "text", "date", "date", "money", "money", "money", "money", "money", "money",
];

/// Anzeigewerte einer Quittung in der Reihenfolge von `export::HEADERS`. Fehlendes bleibt leer.
fn cells(r: &Receipt) -> Vec<String> {
    let text = |v: &Option<String>| v.clone().unwrap_or_default();
    let date = |d: Option<chrono::NaiveDate>| d.map(|d| d.format("%d.%m.%Y").to_string()).unwrap_or_default();
    let money = |m: Option<mas_reader::money::Money>| m.map(|m| m.to_string()).unwrap_or_default();
    vec![
        text(&r.license_no),
        text(&r.printout_no),
        text(&r.expiry),
        text(&r.device_type),
        date(r.date_start),
        date(r.date_end),
        money(r.saldo_1),
        money(r.saldo_2),
        money(r.stakes),
        money(r.winnings),
        money(r.inserted),
        money(r.payout),
    ]
}

#[derive(Serialize)]
struct Row {
    index: usize,
    file_name: String,
    cells: Vec<String>,
}

#[derive(Serialize)]
pub struct ScanDto {
    folder: String,
    location: String,
    columns: Vec<Column>,
    rows: Vec<Row>,
    errors: Vec<String>,
}

#[tauri::command]
pub async fn scan_folder(path: String, state: State<'_, AppState>) -> Result<ScanDto, String> {
    with_loaded(&state, move |loaded| {
        let dir = PathBuf::from(&path);
        let result = scan::scan_folder(&dir).map_err(|e| format!("Ordner konnte nicht gelesen werden: {e}"))?;

        let rows = result
            .files
            .iter()
            .enumerate()
            .map(|(index, f)| Row {
                index,
                file_name: f.path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
                cells: cells(&f.receipt),
            })
            .collect();
        let columns = export::HEADERS.iter().zip(KINDS).map(|(&title, kind)| Column { title, kind }).collect();

        let dto = ScanDto { folder: path, location: result.location.clone(), columns, rows, errors: result.errors };
        loaded.dir = dir;
        loaded.location = result.location;
        loaded.files = result.files;
        Ok(dto)
    })
    .await
}

/// Nur für Entwicklung: `MAS_OPEN_FOLDER=<Pfad> cargo tauri dev` lädt den Ordner beim Start.
#[tauri::command]
pub fn initial_folder() -> Option<String> {
    std::env::var("MAS_OPEN_FOLDER").ok().filter(|p| !p.is_empty())
}

// ---------------------------------------------------------------------------
// Schnellansicht
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct ReceiptText {
    file_name: String,
    lines: Vec<String>,
}

/// Der Bon als bereinigte Zeilen (ohne Druckerbefehle), wie er im PDF stünde.
#[tauri::command]
pub async fn receipt_text(index: usize, state: State<'_, AppState>) -> Result<ReceiptText, String> {
    with_loaded(&state, move |loaded| {
        let file = loaded.files.get(index).ok_or_else(|| format!("Quittung {index} gibt es nicht"))?;
        Ok(ReceiptText {
            file_name: file.path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
            lines: pdf::select_lines(&file.content, &PdfRange::All).lines,
        })
    })
    .await
}

// ---------------------------------------------------------------------------
// Dateinamen-Vorlage
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct PlaceholderDto {
    name: &'static str,
    description: &'static str,
}

#[derive(Serialize)]
pub struct TemplateInfo {
    default_template: &'static str,
    placeholders: Vec<PlaceholderDto>,
}

#[tauri::command]
pub fn template_info() -> TemplateInfo {
    TemplateInfo {
        default_template: organize::DEFAULT_TEMPLATE,
        placeholders: organize::PLACEHOLDERS
            .iter()
            .map(|&(name, description)| PlaceholderDto { name, description })
            .collect(),
    }
}

/// Live-Vorschau mit der gewählten Quittung (oder einer leeren, wenn nichts geladen ist).
/// Fehlerhafte Vorlagen kommen als `Err(Text)` zurück, die Oberfläche zeigt sie rot an.
#[tauri::command]
pub async fn preview_filename(
    template: String,
    index: Option<usize>,
    extension: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    with_loaded(&state, move |loaded| {
        let ext = extension.unwrap_or_default();
        match index.and_then(|i| loaded.files.get(i)) {
            Some(f) => organize::new_filename_with(&template, &loaded.location, &f.receipt, &ext),
            None => organize::new_filename_with(&template, &loaded.location, &Receipt::default(), &ext),
        }
    })
    .await
}

// ---------------------------------------------------------------------------
// Export
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn export_excel(
    path: String,
    indices: Option<Vec<usize>>,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    with_loaded(&state, move |loaded| {
        let chosen = select(&loaded.files, &indices)?;
        export::write_excel(chosen.iter().map(|f| &f.receipt), Path::new(&path))
            .map_err(|e| format!("Excel konnte nicht gespeichert werden: {e}"))?;
        Ok(chosen.len())
    })
    .await
}

/// Bis wohin das PDF reicht. Das Frontend schickt z. B. `{kind: "lines", value: 20}`.
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RangeDto {
    All,
    Lines { value: usize },
    Stakes,
    Saldo2,
    Keyword { value: String },
}

impl RangeDto {
    fn into_range(self) -> Result<PdfRange, String> {
        Ok(match self {
            RangeDto::All => PdfRange::All,
            RangeDto::Lines { value: 0 } => return Err("Die Zeilenzahl muss größer als 0 sein".into()),
            RangeDto::Lines { value } => PdfRange::Lines(value),
            RangeDto::Stakes => PdfRange::UntilStakes,
            RangeDto::Saldo2 => PdfRange::UntilSaldo2,
            RangeDto::Keyword { value } if value.trim().is_empty() => {
                return Err("Bitte ein Stichwort eingeben".into());
            }
            RangeDto::Keyword { value } => PdfRange::UntilKeyword(value),
        })
    }
}

/// Ergebnis eines Exports oder Umbenennens: was geschrieben wurde und was nicht.
#[derive(Serialize, Default)]
pub struct Report {
    out_dir: String,
    written: Vec<String>,
    /// Hinweise, die nichts verhindern (z. B. "Stichwort nicht gefunden, ganze Datei")
    warnings: Vec<String>,
    errors: Vec<String>,
}

/// Macht aus `name` einen im Stapel eindeutigen Namen: `x.pdf`, `x (2).pdf`, ...
/// Zwei Quittungen mit gleichem Namen würden sonst einander überschreiben.
fn unique_name(name: &str, used: &mut HashSet<String>) -> String {
    let mut candidate = name.to_string();
    let path = Path::new(name);
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or(name);
    let ext = path.extension().and_then(|s| s.to_str());
    let mut n = 2;
    while !used.insert(candidate.to_lowercase()) {
        candidate = match ext {
            Some(ext) => format!("{stem} ({n}).{ext}"),
            None => format!("{stem} ({n})"),
        };
        n += 1;
    }
    candidate
}

#[tauri::command]
pub async fn export_pdfs(
    range: RangeDto,
    indices: Option<Vec<usize>>,
    out_dir: Option<String>,
    template: String,
    state: State<'_, AppState>,
) -> Result<Report, String> {
    let range = range.into_range()?;
    with_loaded(&state, move |loaded| {
        let chosen = select(&loaded.files, &indices)?;
        let dir = out_dir.map(PathBuf::from).unwrap_or_else(|| loaded.dir.join("pdf").join(&loaded.location));
        std::fs::create_dir_all(&dir).map_err(|e| format!("Ordner konnte nicht angelegt werden: {e}"))?;

        let mut report = Report { out_dir: dir.display().to_string(), ..Default::default() };
        let mut used = HashSet::new();

        for file in chosen {
            let source = file.path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            let name = match organize::new_filename_with(&template, &loaded.location, &file.receipt, "pdf") {
                Ok(n) => unique_name(&n, &mut used),
                Err(e) => {
                    report.errors.push(format!("{source}: {e}"));
                    continue;
                }
            };
            match pdf::write_pdf(&file.content, &range, &dir.join(&name)) {
                Ok(warning) => {
                    report.written.push(name);
                    if let Some(w) = warning {
                        report.warnings.push(format!("{source}: {w}"));
                    }
                }
                Err(e) => report.errors.push(format!("{source}: PDF fehlgeschlagen: {e}")),
            }
        }
        Ok(report)
    })
    .await
}

/// Benennt die Dateien nach der Vorlage um und kopiert oder verschiebt sie. Bestehende
/// Dateien werden **nie überschrieben**, sie landen als Fehler im Bericht.
#[tauri::command]
pub async fn organize_files(
    mode: String,
    template: String,
    indices: Option<Vec<usize>>,
    out_dir: Option<String>,
    state: State<'_, AppState>,
) -> Result<Report, String> {
    let mode = match mode.as_str() {
        "copy" => Mode::Copy,
        "move" => Mode::Move,
        other => return Err(format!("Unbekannter Modus: {other}")),
    };

    with_loaded(&state, move |loaded| {
        // Auswahl prüfen, bevor etwas angefasst wird
        let wanted: Vec<usize> = match &indices {
            None => (0..loaded.files.len()).collect(),
            Some(list) => list.clone(),
        };
        select(&loaded.files, &indices)?;

        let dir = out_dir.map(PathBuf::from).unwrap_or_else(|| loaded.dir.join("Output").join(&loaded.location));
        let mut report = Report { out_dir: dir.display().to_string(), ..Default::default() };

        for i in wanted {
            let location = loaded.location.clone();
            let file = &mut loaded.files[i];
            let source = file.path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();

            let name = match organize::new_filename_with(&template, &location, &file.receipt, file_ext(&file.path)) {
                Ok(n) => n,
                Err(e) => {
                    report.errors.push(format!("{source}: {e}"));
                    continue;
                }
            };
            match organize::organize(&file.path, &dir, &name, mode) {
                Ok(target) => {
                    if mode == Mode::Move {
                        file.path = target; // der Eintrag zeigt jetzt auf die neue Datei
                    }
                    report.written.push(name);
                }
                Err(e) => report.errors.push(format!("{source}: {e}")),
            }
        }
        Ok(report)
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tabellenspalten_passen_zu_excel() {
        assert_eq!(KINDS.len(), export::HEADERS.len());
        assert_eq!(cells(&Receipt::default()).len(), export::HEADERS.len());
    }

    #[test]
    fn zellen_sind_lesbar_formatiert() {
        let r = Receipt {
            license_no: Some("426710407".into()),
            date_end: chrono::NaiveDate::from_ymd_opt(2023, 1, 30),
            saldo_1: Some(mas_reader::money::Money(113300)),
            ..Default::default()
        };
        let c = cells(&r);
        assert_eq!(c[0], "426710407");
        assert_eq!(c[5], "30.01.2023");
        assert_eq!(c[6], "1133,00");
        assert_eq!(c[7], ""); // fehlend = leer, die Oberfläche zeigt "–"
    }

    #[test]
    fn zeilenbereich_wird_geprueft() {
        assert!(RangeDto::Lines { value: 0 }.into_range().is_err());
        assert!(RangeDto::Keyword { value: "  ".into() }.into_range().is_err());
        assert_eq!(RangeDto::Lines { value: 5 }.into_range().unwrap(), PdfRange::Lines(5));
        assert_eq!(RangeDto::Saldo2.into_range().unwrap(), PdfRange::UntilSaldo2);
    }

    #[test]
    fn range_kommt_als_json_an() {
        let r: RangeDto = serde_json::from_str(r#"{"kind":"lines","value":20}"#).unwrap();
        assert_eq!(r.into_range().unwrap(), PdfRange::Lines(20));
        let r: RangeDto = serde_json::from_str(r#"{"kind":"saldo2"}"#).unwrap();
        assert_eq!(r.into_range().unwrap(), PdfRange::UntilSaldo2);
    }

    #[test]
    fn gleiche_namen_im_stapel_werden_nummeriert() {
        let mut used = HashSet::new();
        assert_eq!(unique_name("Bar.pdf", &mut used), "Bar.pdf");
        assert_eq!(unique_name("Bar.pdf", &mut used), "Bar (2).pdf");
        assert_eq!(unique_name("BAR.pdf", &mut used), "BAR (3).pdf"); // Groß-/Kleinschreibung egal
        assert_eq!(unique_name("Anders.pdf", &mut used), "Anders.pdf");
    }
}

/// Ruft die Befehle über die echte IPC-Schicht auf, genau wie das Frontend
/// (JSON-Argumente in camelCase, JSON-Antworten in snake_case).
#[cfg(test)]
mod ipc_tests {
    use super::*;
    use serde_json::{Value, json};
    use tauri::ipc::{CallbackFn, InvokeBody};
    use tauri::test::{INVOKE_KEY, MockRuntime, get_ipc_response, mock_builder};
    use tauri::webview::InvokeRequest;
    use tauri::{WebviewWindow, WebviewWindowBuilder};

    const BON: &str = "ZULASSUNGS NR.:426710407\nAUSDRUCK NR.:0018  A 018\nKASSIERUNG VOM:\n\x1bK&\n30.01.2023 17:16:19.2\nLETZTE KASSIERUNG:\n\x1bK'\n30.12.2022 16:50:21.6\nEINWURF : 4415,80\nSALDO (1) : 1133,00\n";

    fn call(window: &WebviewWindow<MockRuntime>, cmd: &str, args: Value) -> Result<Value, Value> {
        let request = InvokeRequest {
            cmd: cmd.into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            // lokaler Ursprung der App: unter Windows http://tauri.localhost, sonst tauri://localhost
            url: (if cfg!(windows) { "http://tauri.localhost" } else { "tauri://localhost" }).parse().unwrap(),
            body: InvokeBody::Json(args),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_string(),
        };
        get_ipc_response(window, request).map(|body| body.deserialize::<Value>().unwrap())
    }

    #[test]
    fn frontend_ablauf_ueber_ipc() {
        let base = std::env::temp_dir().join(format!("mas_gui_ipc_{}", std::process::id()));
        let dir = base.join("Bar");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.ACK"), BON).unwrap();
        std::fs::write(dir.join("b.ACK"), BON).unwrap();

        let app = mock_builder()
            .manage(AppState::default())
            .invoke_handler(handler())
            .build(context())
            .unwrap();
        let w = WebviewWindowBuilder::new(&app, "main", Default::default()).build().unwrap();

        // Vorlage + Platzhalter
        let info = call(&w, "template_info", json!({})).unwrap();
        assert!(info["default_template"].as_str().unwrap().contains("{ort}"));
        assert_eq!(info["placeholders"].as_array().unwrap().len(), organize::PLACEHOLDERS.len());

        // Ordner laden: Tabelle wie in Excel
        let scan = call(&w, "scan_folder", json!({ "path": dir })).unwrap();
        assert_eq!(scan["location"], "Bar");
        assert_eq!(scan["columns"].as_array().unwrap().len(), export::HEADERS.len());
        assert_eq!(scan["rows"].as_array().unwrap().len(), 2);
        assert_eq!(scan["rows"][0]["cells"][0], "426710407");
        assert_eq!(scan["rows"][0]["cells"][1], "A 018");
        assert_eq!(scan["rows"][0]["cells"][6], "1133,00");

        // Schnellansicht: ohne Druckerbefehle
        let text = call(&w, "receipt_text", json!({ "index": 0 })).unwrap();
        assert!(text["lines"].as_array().unwrap().iter().all(|l| !l.as_str().unwrap().contains('\x1b')));

        // Live-Vorschau, auch mit Fehlern
        let name = call(&w, "preview_filename", json!({ "template": "{ort} {ausdruck}", "index": 0, "extension": "ACK" })).unwrap();
        assert_eq!(name, "Bar A 018.ACK");
        let err = call(&w, "preview_filename", json!({ "template": "{quatsch}", "index": 0, "extension": "ACK" })).unwrap_err();
        assert!(err.as_str().unwrap().contains("quatsch"));

        // Excel
        let xlsx = base.join("t.xlsx");
        assert_eq!(call(&w, "export_excel", json!({ "path": xlsx, "indices": null })).unwrap(), 2);
        assert!(xlsx.exists());

        // PDF: Stichwort fehlt -> ganze Datei + Hinweis; gleiche Namen werden nummeriert
        let pdfs = call(&w, "export_pdfs", json!({
            "range": { "kind": "saldo2" }, "indices": null, "outDir": null, "template": "{ort}-{zulassung}"
        })).unwrap();
        assert_eq!(pdfs["written"], json!(["Bar-426710407.pdf", "Bar-426710407 (2).pdf"]));
        assert_eq!(pdfs["warnings"].as_array().unwrap().len(), 2);
        assert!(dir.join("pdf").join("Bar").join("Bar-426710407 (2).pdf").exists());

        // PDF-Optionen werden geprüft
        let bad = call(&w, "export_pdfs", json!({ "range": { "kind": "lines", "value": 0 }, "indices": null, "outDir": null, "template": "x" }));
        assert!(bad.is_err());

        // Umbenennen: erst Kopie, dann Wiederholung ohne Überschreiben
        let args = json!({ "mode": "copy", "template": organize::DEFAULT_TEMPLATE, "indices": [0], "outDir": null });
        let first = call(&w, "organize_files", args.clone()).unwrap();
        assert_eq!(first["written"].as_array().unwrap().len(), 1);
        assert!(dir.join("a.ACK").exists()); // Original bleibt
        let again = call(&w, "organize_files", args).unwrap();
        assert_eq!(again["written"].as_array().unwrap().len(), 0);
        assert_eq!(again["errors"].as_array().unwrap().len(), 1); // existiert bereits

        // Verschieben: Original weg, Bon bleibt in der Schnellansicht lesbar
        let moved = call(&w, "organize_files", json!({ "mode": "move", "template": "{ort}-{zulassung}-neu", "indices": [1], "outDir": null })).unwrap();
        assert_eq!(moved["written"].as_array().unwrap().len(), 1);
        assert!(!dir.join("b.ACK").exists());
        assert!(call(&w, "receipt_text", json!({ "index": 1 })).is_ok());

        assert!(call(&w, "organize_files", json!({ "mode": "quatsch", "template": "x", "indices": null, "outDir": null })).is_err());

        std::fs::remove_dir_all(&base).ok();
    }
}
