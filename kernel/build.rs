use std::fs::{read_dir, File};
use std::io::{Result, Write};
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=../user/c/src");
    println!("cargo:rerun-if-changed=../user/rust/src");
    insert_app_data().unwrap();
}

fn insert_app_data() -> Result<()> {
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let app_path = PathBuf::from("../user/build/").join(&arch);
    let link_app_path = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("link_app.S");

    if let Ok(dir) = read_dir(&app_path) {
        let mut apps = dir
            .into_iter()
            .map(|dir_entry| dir_entry.unwrap().file_name().into_string().unwrap())
            .collect::<Vec<_>>();
        apps.sort();

        let mut f = File::create(link_app_path)?;
        writeln!(
            f,
            "
.section .data
.balign 8
.global _app_count
_app_count:
    .quad {}",
            apps.len()
        )?;
        for i in 0..apps.len() {
            writeln!(f, "    .quad app_{}_name", i)?;
            writeln!(f, "    .quad app_{}_start", i)?;
        }
        writeln!(f, "    .quad app_{}_end", apps.len() - 1)?;

        for (idx, app) in apps.iter().enumerate() {
            println!("app_{}: {}", idx, app_path.join(app).display());
            writeln!(
                f,
                "
app_{0}_name:
    .string \"{1}\"
.balign 8
app_{0}_start:
    .incbin \"{2}\"
app_{0}_end:",
                idx,
                app,
                app_path.join(app).display()
            )?;
        }
    } else {
        let mut f = File::create(link_app_path)?;
        writeln!(
            f,
            "
.section .data
.balign 8
.global _app_count
_app_count:
    .quad 0"
        )?;
    }
    Ok(())
}
