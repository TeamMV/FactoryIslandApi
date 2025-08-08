use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::str::FromStr;
use toml_edit::{Document, DocumentMut, Item};
use crate::MapErrorToString;

//this is all chatgpt lmao i cannot be asked
pub fn fix_relative_dependency_paths(
    src_cargo: impl AsRef<Path>,
    dst_cargo: impl AsRef<Path>,
) -> Result<(), String> {
    let src_cargo = src_cargo.as_ref();
    let dst_cargo = dst_cargo.as_ref();

    let src_dir = src_cargo.parent().unwrap();
    let dst_dir = dst_cargo.parent().unwrap();

    let src_contents = fs::read_to_string(src_cargo).mets()?;
    let dst_raw = fs::read_to_string(dst_cargo).mets()?;
    let mut dst_contents = DocumentMut::from_str(&dst_raw).mets()?;
    let src_doc = Document::parse(&src_contents).mets()?;

    // Sections that may contain path dependencies
    let sections = [
        "dependencies",
        "dev-dependencies",
        "build-dependencies",
        "target",
        "patch",
    ];

    for section in &sections {
        if section == &"target" {
            if let Some(targets) = src_doc.get("target") {
                for (target_name, target_table) in targets.as_table().unwrap().iter() {
                    if let Some(deps) = target_table.get("dependencies") {
                        fix_path_table(
                            deps.as_table().unwrap(),
                            dst_contents
                                .get_mut("target")
                                .and_then(|t| t.get_mut(target_name))
                                .and_then(|t| t.get_mut("dependencies"))
                                .and_then(Item::as_table_like_mut),
                            src_dir,
                            dst_dir,
                        );
                    }
                }
            }
        } else if let Some(src_table) = src_doc.get(section).and_then(Item::as_table_like) {
            let dst_table = dst_contents
                .get_mut(section)
                .and_then(Item::as_table_like_mut);
            fix_path_table(src_table, dst_table, src_dir, dst_dir);
        }
    }

    fs::write(dst_cargo, dst_contents.to_string()).mets()?;
    Ok(())
}

fn fix_path_table(
    src_table: &dyn toml_edit::TableLike,
    dst_table: Option<&mut dyn toml_edit::TableLike>,
    src_dir: &Path,
    dst_dir: &Path,
) {
    let Some(dst_table) = dst_table else { return };

    for (key, src_dep_item) in src_table.iter() {
        let Some(dst_dep_item) = dst_table.get_mut(key) else { continue };

        // Inline table: foo = { path = "../lib" }
        if let Some(dep_table) = dst_dep_item.as_inline_table_mut() {
            if let Some(path_value) = dep_table.get("path") {
                if let Some(relative_path) = path_value.as_str() {
                    if let Some(new_path) =
                        fix_relative_path(relative_path, src_dir, dst_dir)
                    {
                        dep_table.insert("path", toml_edit::value(new_path).into_value().unwrap());
                    }
                }
            }
        }

        // Full table: [dependencies.foo] path = "../lib"
        else if let Some(dep_table) = dst_dep_item.as_table_mut() {
            if let Some(path_value) = dep_table.get("path") {
                if let Some(relative_path) = path_value.as_str() {
                    if let Some(new_path) =
                        fix_relative_path(relative_path, src_dir, dst_dir)
                    {
                        dep_table.insert("path", toml_edit::value(new_path));
                    }
                }
            }
        }
    }
}

fn fix_relative_path(relative: &str, src_dir: &Path, dst_dir: &Path) -> Option<String> {
    let old_path = src_dir.join(relative);
    let new_relative = pathdiff::diff_paths(old_path, dst_dir)?;
    Some(new_relative.to_string_lossy().into_owned())
}

pub fn prettify_rust_code(code: &str) -> Result<String, String> {
    let mut child = Command::new("rustfmt")
        .arg("--emit").arg("stdout")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn rustfmt: {}", e))?;

    {
        let stdin = child.stdin.as_mut().ok_or("Failed to open stdin")?;
        stdin.write_all(code.as_bytes()).map_err(|e| format!("Failed to write to rustfmt stdin: {}", e))?;
    }

    let output = child.wait_with_output().map_err(|e| format!("Failed to read rustfmt output: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "rustfmt failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    String::from_utf8(output.stdout).map_err(|e| format!("Invalid UTF-8 from rustfmt: {}", e))
}