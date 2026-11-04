//! Kern des MAS-Readers: Quittungen einlesen, umbenennen, als Excel und PDF ausgeben.
//! Wird vom CLI (`main.rs`) und von der Tauri-GUI (`src-tauri`) genutzt.

pub mod export;
pub mod money;
pub mod organize;
pub mod patterns;
pub mod pdf;
pub mod receipt;
pub mod scan;
