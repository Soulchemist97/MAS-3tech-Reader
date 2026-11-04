mod export;
mod money;
mod organize;
mod patterns;
mod pdf;
mod receipt;

use dialoguer::{Input, Select};
use organize::Mode;
use pdf::PdfRange;
use receipt::Receipt;

use rfd::FileDialog;
use std::fs;

/// Liest eine Datei als Text. Die Dateien sind kein gültiges UTF-8, daher
/// werden ungültige Bytes zu '�'. Steuerzeichen (ESC) bleiben erhalten,
/// weil die Patterns sie brauchen.
fn read_content(path: &std::path::Path) -> std::io::Result<String> {
    let bytes = fs::read(path)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// Fragt im Terminal, ob und bis wohin PDFs erzeugt werden sollen.
/// `Ok(None)` = keine PDFs.
fn ask_pdf_range() -> dialoguer::Result<Option<PdfRange>> {
    let choice = Select::new()
        .with_prompt("PDFs erzeugen?")
        .items([
            "Nein, keine PDFs",
            "Ganze Datei",
            "Bis Einsätze (EINSAETZE + 1 Zeile)",
            "Bis Saldo 2 (SALDO (2) + 1 Zeile)",
            "Erste N Zeilen",
            "Bis zu einem eigenen Stichwort",
        ])
        .default(0)
        .interact()?;

    Ok(match choice {
        1 => Some(PdfRange::All),
        2 => Some(PdfRange::UntilStakes),
        3 => Some(PdfRange::UntilSaldo2),
        4 => {
            let n: usize = Input::new().with_prompt("Anzahl Zeilen").interact_text()?;
            Some(PdfRange::Lines(n))
        }
        5 => {
            let word: String = Input::new()
                .with_prompt("Stichwort (Groß-/Kleinschreibung und Leerzeichen egal)")
                .interact_text()?;
            Some(PdfRange::UntilKeyword(word))
        }
        _ => None,
    })
}

/// Fragt, ob die umbenannten Dateien kopiert oder verschoben werden.
/// Standard ist Kopieren, weil das nichts zerstört.
fn ask_mode() -> dialoguer::Result<Mode> {
    let choice = Select::new()
        .with_prompt("Dateien umbenennen")
        .items(["Kopieren (Original bleibt)", "Verschieben (Original wird entfernt)"])
        .default(0)
        .interact()?;

    Ok(if choice == 1 { Mode::Move } else { Mode::Copy })
}

fn main() {
    // Open a file dialog to select a folder
    let folder = FileDialog::new()
    .set_title("Select a folder")
    .pick_folder();

    // print folder
    match folder {
        Some(path) => {
            println!("Selected folder: {}", path.display());
            let mut receipts: Vec<Receipt> = Vec::new(); // sammelt alle Quittungen

            // Aufstellort = Name des gewählten Ordners (Python: Pfad.split("\\")[-1])
            let location = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let out_dir = path.join("Output").join(&location);

            // Optionen einmal vor der Schleife abfragen
            let mode = ask_mode().unwrap_or_else(|e| {
                eprintln!("Abfrage fehlgeschlagen, es wird kopiert: {e}");
                Mode::Copy
            });
            let pdf_range = ask_pdf_range().unwrap_or_else(|e| {
                eprintln!("Abfrage fehlgeschlagen, es werden keine PDFs erzeugt: {e}");
                None
            });
            let pdf_dir = path.join("pdf").join(&location);
            if pdf_range.is_some()
                && let Err(e) = fs::create_dir_all(&pdf_dir)
            {
                eprintln!("PDF-Ordner konnte nicht angelegt werden: {e}");
            }
            // Alle Dateien im Ordner durchgehen und Inhalt ausgeben
            for entry in fs::read_dir(&path).unwrap() {
                let file_path = entry.unwrap().path();

                // Unterordner und fremde Dateien überspringen
                let is_ack = file_path
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("ACK"));
                if !file_path.is_file() || !is_ack {
                    continue;
                }

                let content = match read_content(&file_path) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("{}: {e}", file_path.display());
                        continue;
                    }
                };

                let receipt = Receipt::parse(&content);

                println!("===== {} =====", file_path.display());
                println!("{receipt:#?}");
                println!("Gerätetyp: {}", receipt.device_type.as_deref().unwrap_or("unbekannt"));

                // Umbenennen: kopieren oder verschieben, je nach Auswahl oben
                let extension = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
                let new_name = organize::new_filename(&location, &receipt, extension);
                match organize::organize(&file_path, &out_dir, &new_name, mode) {
                    Ok(target) => println!("-> {}", target.display()),
                    Err(e) => eprintln!("{}: {e}", file_path.display()),
                }

                // PDF mit der gewählten Auswahl (None = keine PDFs)
                if let Some(range) = &pdf_range {
                    let pdf_name = std::path::Path::new(&new_name).with_extension("pdf");
                    let pdf_path = pdf_dir.join(pdf_name);
                    match pdf::write_pdf(&content, range, &pdf_path) {
                        Ok(warning) => {
                            println!("-> {}", pdf_path.display());
                            if let Some(w) = warning {
                                eprintln!("   Hinweis: {w}");
                            }
                        }
                        Err(e) => eprintln!("{}: PDF fehlgeschlagen: {e}", file_path.display()),
                    }
                }

                receipts.push(receipt); // ab hier gehört die Quittung dem Vec
            }

            // Einmal nach der Schleife: alle Quittungen in eine Excel-Datei
            let excel_path = path.join("Zusammenfassung.xlsx");
            match export::write_excel(&receipts, &excel_path) {
                Ok(()) => println!("{} Quittungen gespeichert: {}", receipts.len(), excel_path.display()),
                Err(e) => eprintln!("Excel konnte nicht gespeichert werden: {e}"),
            }
        }
        None => println!("No folder selected"),
    }
}
