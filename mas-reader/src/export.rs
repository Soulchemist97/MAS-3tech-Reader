use std::path::Path;

use chrono::{Datelike, NaiveDate};
use rust_xlsxwriter::{ExcelDateTime, Format, Workbook, Worksheet, XlsxError};

use crate::money::Money;
use crate::receipt::Receipt;

const HEADERS: [&str; 12] = [
    "Zulassung",
    "Ausdruck",
    "Ablauf",
    "Gerätetyp",
    "Anfangsdatum",
    "Enddatum",
    "Saldo 1",
    "Saldo 2",
    "Einsätze",
    "Gewinne",
    "Einwurf",
    "Auswurf",
];

/// Schreibt alle Quittungen in eine Excel-Datei: eine Zeile pro Quittung.
pub fn write_excel(receipts: &[Receipt], path: &Path) -> Result<(), XlsxError> {
    let mut workbook = Workbook::new();
    let sheet = workbook.add_worksheet();

    let bold = Format::new().set_bold();
    let date_format = Format::new().set_num_format("dd.mm.yyyy");
    let money_format = Format::new().set_num_format("#,##0.00");

    for (col, title) in HEADERS.iter().enumerate() {
        sheet.write_string_with_format(0, col as u16, *title, &bold)?;
    }

    for (i, r) in receipts.iter().enumerate() {
        let row = i as u32 + 1; // Zeile 0 sind die Überschriften

        write_text(sheet, row, 0, &r.license_no)?;
        write_text(sheet, row, 1, &r.printout_no)?;
        write_text(sheet, row, 2, &r.expiry)?;
        write_text(sheet, row, 3, &r.device_type)?;
        write_date(sheet, row, 4, r.date_start, &date_format)?;
        write_date(sheet, row, 5, r.date_end, &date_format)?;
        write_money(sheet, row, 6, r.saldo_1, &money_format)?;
        write_money(sheet, row, 7, r.saldo_2, &money_format)?;
        write_money(sheet, row, 8, r.stakes, &money_format)?;
        write_money(sheet, row, 9, r.winnings, &money_format)?;
        write_money(sheet, row, 10, r.inserted, &money_format)?;
        write_money(sheet, row, 11, r.payout, &money_format)?;
    }

    sheet.autofit();
    workbook.save(path)
}

// `None` schreibt nichts, die Zelle bleibt leer.

fn write_text(sheet: &mut Worksheet, row: u32, col: u16, value: &Option<String>) -> Result<(), XlsxError> {
    if let Some(text) = value {
        sheet.write_string(row, col, text)?;
    }
    Ok(())
}

fn write_money(sheet: &mut Worksheet, row: u32, col: u16, value: Option<Money>, format: &Format) -> Result<(), XlsxError> {
    if let Some(m) = value {
        // Excel kennt nur Fließkommazahlen; intern bleibt alles in Cent.
        sheet.write_number_with_format(row, col, m.0 as f64 / 100.0, format)?;
    }
    Ok(())
}

fn write_date(sheet: &mut Worksheet, row: u32, col: u16, value: Option<NaiveDate>, format: &Format) -> Result<(), XlsxError> {
    if let Some(d) = value {
        let excel_date = ExcelDateTime::from_ymd(d.year() as u16, d.month() as u8, d.day() as u8)?;
        sheet.write_datetime_with_format(row, col, &excel_date, format)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schreibt_excel_datei_auch_mit_leeren_feldern() {
        let voll = Receipt {
            license_no: Some("426710407".into()),
            date_end: NaiveDate::from_ymd_opt(2023, 1, 30),
            saldo_1: Some(Money(113300)),
            ..Default::default()
        };
        let leer = Receipt::default(); // alles None -> leere Zellen

        let path = std::env::temp_dir().join("mas_reader_test.xlsx");
        write_excel(&[voll, leer], &path).unwrap();

        assert!(std::fs::metadata(&path).unwrap().len() > 0);
        std::fs::remove_file(path).ok();
    }
}
