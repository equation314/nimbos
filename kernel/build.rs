use std::fs::{read_dir, File};
use std::io::{Result, Write};

fn main() {
    println!("cargo:rerun-if-changed=../user/c/src");
    println!("cargo:rerun-if-changed=../user/rust/src");
    insert_app_data().ok();
}

static TARGET_PATH: &str = "../user/build/aarch64/";

fn insert_app_data() -> Result<()> {
    let mut f = File::create("src/link_app.S")?;
    let mut apps: Vec<_> = read_dir(TARGET_PATH)?
        .into_iter()
        .map(|dir_entry| dir_entry.unwrap().file_name().into_string().unwrap())
        .collect();
    apps.sort();

    writeln!(
        f,
        r#"
    .align 3
    .section .data
    .global _app_count
_app_count:
    .quad {}"#,
        apps.len()
    )?;

    for i in 0..apps.len() {
        writeln!(f, r#"    .quad app_{}_name"#, i)?;
        writeln!(f, r#"    .quad app_{}_start"#, i)?;
    }
    writeln!(f, r#"    .quad app_{}_end"#, apps.len() - 1)?;

    for (idx, app) in apps.iter().enumerate() {
        println!("app_{}: {}", idx, app);
        writeln!(
            f,
            r#"
    .section .data
app_{0}_name:
    .string "{1}"
    .align 3
app_{0}_start:
    .incbin "{2}{1}"
app_{0}_end:"#,
            idx, app, TARGET_PATH
        )?;
    }
    Ok(())
}
