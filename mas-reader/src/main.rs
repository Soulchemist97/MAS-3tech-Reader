use dialoguer::{Input, Select};
use mas_reader::organize::{self, Mode};
use mas_reader::pdf::{self, PdfRange};
use mas_reader::{export, scan};
use rfd::FileDialog;
use std::fs;

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
            let occurrence: usize = Input::new()
                .with_prompt("Wievielte Nennung des Stichworts (1 = erste)")
                .default(1)
                .interact_text()?;
            let offset: i32 = Input::new()
                .with_prompt("Zeilen nach (+) oder vor (-) der Stichwortzeile")
                .default(0)
                .interact_text()?;
            Some(PdfRange::UntilKeyword { word, occurrence, offset })
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
    // Ordnerdialog
    let Some(path) = FileDialog::new().set_title("Select a folder").pick_folder() else {
        println!("No folder selected");
        return;
    };
    println!("Selected folder: {}", path.display());

    let scanned = match scan::scan_folder(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Ordner konnte nicht gelesen werden: {e}");
            return;
        }
    };
    for err in &scanned.errors {
        eprintln!("{err}");
    }

    // Aufstellort = Name des gewählten Ordners
    let location = &scanned.location;
    let out_dir = organize::default_output_dir(&path, location);

    // Optionen einmal vor der Schleife abfragen
    let mode = ask_mode().unwrap_or_else(|e| {
        eprintln!("Abfrage fehlgeschlagen, es wird kopiert: {e}");
        Mode::Copy
    });
    let pdf_range = ask_pdf_range().unwrap_or_else(|e| {
        eprintln!("Abfrage fehlgeschlagen, es werden keine PDFs erzeugt: {e}");
        None
    });
    let pdf_dir = path.join("pdf").join(location);
    if pdf_range.is_some()
        && let Err(e) = fs::create_dir_all(&pdf_dir)
    {
        eprintln!("PDF-Ordner konnte nicht angelegt werden: {e}");
    }

    for file in &scanned.files {
        let receipt = &file.receipt;

        println!("===== {} =====", file.path.display());
        println!("{receipt:#?}");
        println!("Gerätetyp: {}", receipt.device_type.as_deref().unwrap_or("unbekannt"));

        // Umbenennen: kopieren oder verschieben, je nach Auswahl oben
        let extension = file.path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let new_name = organize::new_filename(location, receipt, extension);
        match organize::organize(&file.path, &out_dir, &new_name, mode) {
            Ok(target) => println!("-> {}", target.display()),
            Err(e) => eprintln!("{}: {e}", file.path.display()),
        }

        // PDF mit der gewählten Auswahl (None = keine PDFs)
        if let Some(range) = &pdf_range {
            let pdf_name = std::path::Path::new(&new_name).with_extension("pdf");
            let pdf_path = pdf_dir.join(pdf_name);
            match pdf::write_pdf(&file.content, range, &pdf_path) {
                Ok(warning) => {
                    println!("-> {}", pdf_path.display());
                    if let Some(w) = warning {
                        eprintln!("   Hinweis: {w}");
                    }
                }
                Err(e) => eprintln!("{}: PDF fehlgeschlagen: {e}", file.path.display()),
            }
        }
    }

    // Einmal nach der Schleife: alle Quittungen in eine Excel-Datei
    let excel_path = path.join("Zusammenfassung.xlsx");
    match export::write_excel(scanned.files.iter().map(|f| &f.receipt), &excel_path) {
        Ok(()) => println!("{} Quittungen gespeichert: {}", scanned.files.len(), excel_path.display()),
        Err(e) => eprintln!("Excel konnte nicht gespeichert werden: {e}"),
    }
}
