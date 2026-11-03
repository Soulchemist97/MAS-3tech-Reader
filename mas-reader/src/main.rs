use rfd::FileDialog;
use std::fs;


fn main() {
    let folder = FileDialog::new()
    .set_title("Select a folder")
    .pick_folder();

    match folder {
        Some(path) => {
            println!("Selected folder: {}", path.display());
            for entry in fs::read_dir(&path).unwrap() {
                let entry = entry.unwrap();
                println!(" - {}", entry.path().display());
            }
        }
        None => println!("No folder selected"),
    }
}
