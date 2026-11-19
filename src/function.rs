use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use serde::Deserialize;
use std::fs::{self, File};
use dirs::cache_dir;
use sha256::try_digest;
use colored::Colorize;

pub struct PkgInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author : String,
}

#[derive(Debug, Deserialize)]
pub struct Index {
    pub schema: u32,
    pub repository: String,
    pub packages: Vec<Package>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub archive: String,
    pub sha256: String,
}

pub const MAIN_INDEX_URL: &str = "https://codeberg.org/n1tronium/nitrite-repo/raw/branch/main/index.json";

pub fn read(path: &Path) -> Result<PkgInfo, String> {
    let pkgfile = fs::read_to_string(path)
        .map_err(|err| format!("cannot read {} -> {err}", path.display()))?;

    let mut info = PkgInfo {
        name: String::new(),
        version: String::new(),
        description: String::new(),
        author: String::new(),
    };

    for line in pkgfile.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();

            match key {
                "name" => info.name = value.to_string(),
                "version" => info.version = value.to_string(),
                "description" => info.description = value.to_string(),
                "author" => info.author = value.to_string(),
                _ => {}
            }
        }
    }

    Ok(info)
}

pub fn read_index() -> Result<Index, String> {
    let dir = cache_dir()
        .ok_or_else(|| "cannot find cache dir".to_string())?
        .join("nitrite/index/main.json");

    let index = fs::read_to_string(dir)
        .map_err(|err| format!("main repository not synced -> {err}"))?;

    let index = index.trim();

    let json : Index = serde_json::from_str(&index)
        .map_err(|err| format!("invalid index -> {err}"))?;

    if json.schema != 1 {
        return Err(format!("unsupported schema -> {}", json.schema));
    }
    Ok(json)
}

pub fn find_package(pkgname: &str, index: &Index) -> Option<Package> {
    for package in &index.packages {
        if package.name == pkgname {
            return Some(package.clone());
        }
    }
    None
}


pub fn sync_main() -> Result<(), String> {
    let dir = cache_dir()
        .expect("cannot find cache dir")
        .join("nitrite/index");

    fs::create_dir_all(&dir)
        .map_err(|err| format!("cannot create {} -> {err}", dir.display()))?;

    let part = dir.join("main.json.parted");
    let target = dir.join("main.json");

    let bytes = ureq::get(MAIN_INDEX_URL)
        .call()
        .map_err(|err| format!("cannot download {MAIN_INDEX_URL} -> {err}"))?
        .body_mut()
        .read_to_vec()
        .map_err(|err| format!("cannot read response body -> {err}"))?;

    let index : Index = serde_json::from_slice(&bytes)
        .map_err(|err| format!("invalid index -> {err}"))?;

    if index.schema != 1 {
        return Err(format!("unsupported schema -> {}", index.schema));
    }

    fs::write(&part, &bytes)
        .map_err(|err| format!("cannot write {} -> {err}", part.display()))?;
    fs::rename(&part, &target)
        .map_err(|err| format!("cannot rename {} -> {err}", part.display()))?;

    Ok(())
}


pub fn download_archive(pkg : &Package) -> Result<PathBuf, String> {
    let dir = cache_dir()
        .expect("cannot find cache dir")
        .join("nitrite/archives");

    let file_name = format!("{}-{}.tar.zst", pkg.name, pkg.version);
    let part = dir.join(format!("{file_name}.parted"));
    let target = dir.join(&file_name);

    fs::create_dir_all(&dir)
        .map_err(|err| format!("cannot create {} -> {err}", dir.display()))?;

    let mut response = ureq::get(&pkg.archive)
        .call()
        .map_err(|err| format!("cannot download {} -> {err}", pkg.archive))?;

    let mut file = fs::File::create(&part)
        .map_err(|err| format!("cannot create {} -> {err}", part.display()))?;

    let mut reader = response.body_mut().as_reader();

    std::io::copy(&mut reader, &mut file)
        .map_err(|err| format!("cannot write {} -> {err}", part.display()))?;

    drop(file);

    print!("{}", "verifying sha256... ");
    let _ = std::io::stdout().flush();

    let actual = sha256_file(&part)?;
    if actual != pkg.sha256 {
        println!("{}", "error".red().bold());
        let _ = fs::remove_file(&part);
        return Err(format!(
            "\nsha256 mismatch for '{}'\nexpected -> {}\ngot -> {}",
            pkg.name.bright_blue(), pkg.sha256.red(), actual.red()));
    }

    println!("{}\n", "ok".green().bold());

    fs::rename(&part, &target)
        .map_err(|err| format!("cannot rename {} -> {err}", part.display()))?;
    Ok(target)
}

fn sha256_file(path: &Path) -> Result<String, String> {
    try_digest(path)
        .map_err(|err| format!("cannot hash {} -> {err}", path.display()))
}

pub fn extract_archive(archive: &Path, dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest)
    .map_err(|err| format!("cannot create {} -> {err}", dest.display()))?;

    let file = File::open(archive)
    .map_err(|err| format!("cannot open {} -> {err}", archive.display()))?;

    let decoder = zstd::Decoder::new(file)
    .map_err(|err| format!("cannot decode {} -> {err}", archive.display()))?;

    let mut tar = tar::Archive::new(decoder);

    tar.unpack(dest)
    .map_err(|err| format!("cannot extract {} -> {err}", archive.display()))?;

    Ok(())
}

pub fn pkgfile_difference(pkg: &Package, pkginfo_path: &Path) -> Result<(), String> {
    let info = read(pkginfo_path)?;

    if info.name != pkg.name {
        return Err(format!(
            "\nPKGINFO name mismatch\nexpected -> {}\ngot -> {}",
            pkg.name.red(), info.name.red()
        ));
    }

    if info.version != pkg.version {
        return Err(format!(
            "\nPKGINFO version mismatch\nexpected -> {}\ngot -> {}",
            pkg.version.red(), info.version.red()
        ));
    }

    Ok(())
}
pub fn build_package(pkg: &Package, srcdir: &Path) -> Result<PathBuf, String> {
    let pkgdir = cache_dir()
        .ok_or_else(|| "cannot find cache dir".to_string())?
        .join("nitrite/build")
        .join(format!("{}-{}", pkg.name, pkg.version));

    if pkgdir.exists() {
        fs::remove_dir_all(&pkgdir)
            .map_err(|err| format!("cannot clean {} -> {err}", pkgdir.display()))?;
    }

    fs::create_dir_all(&pkgdir)
        .map_err(|err| format!("cannot create {} -> {err}", pkgdir.display()))?;
    println!("building...");
    let status = Command::new("sh")
        .arg("build.sh")
        .current_dir(srcdir)
        .env("SRCDIR", srcdir)
        .env("PKGDIR", &pkgdir)
        .status()
        .map_err(|err| format!("cannot run build.sh -> {err}"))?;

    if !status.success() {
        return Err(format!("build.sh failed with status {status}"));
    }

    Ok(pkgdir)
}

pub fn check_staging(staged: &Path) -> Result<usize, String> {
    fn count_files(dir: &Path) -> Result<usize, String> {
        let mut count = 0;

        for entry in fs::read_dir(dir)
            .map_err(|err| format!("cannot read {} -> {err}", dir.display()))?
        {
            let entry = entry
                .map_err(|err| format!("cannot read {} -> {err}", dir.display()))?;
            let path = entry.path();
            let kind = entry
                .file_type()
                .map_err(|err| format!("cannot stat {} -> {err}", path.display()))?;

            if kind.is_dir() {
                count += count_files(&path)?;
            } else {
                count += 1;
            }
        }

        Ok(count)
    }

    let count = count_files(staged)?;
    let word = if count == 1 { "file" } else { "files" };

    println!("\n{} {} {}", "checking staging...", count, word);

    Ok(count)
}

pub fn install_staging(staged: &Path) -> Result<(), String> {
    fn copy_files(src: &Path, dst: &Path) -> Result<(), String> {
        for entry in fs::read_dir(src)
            .map_err(|err| format!("cannot read {} -> {err}", src.display()))?
        {
            let entry = entry
                .map_err(|err| format!("cannot read {} -> {err}", src.display()))?;
            let from = entry.path();
            let to = dst.join(entry.file_name());
            let kind = entry
                .file_type()
                .map_err(|err| format!("cannot stat {} -> {err}", from.display()))?;

            if kind.is_dir() {
                fs::create_dir_all(&to)
                    .map_err(|err| format!("cannot create {} -> {err}", to.display()))?;
                copy_files(&from, &to)?;
            } else {
                fs::copy(&from, &to)
                    .map_err(|err| format!("cannot copy {} -> {err}", to.display()))?;
            }
        }

        Ok(())
    }

    let prefix = dirs::home_dir()
        .ok_or_else(|| "cannot find home dir".to_string())?
        .join(".local");

    println!("{}", "installing to ~/.local...".cyan());

    fs::create_dir_all(&prefix)
        .map_err(|err| format!("cannot create {} -> {err}", prefix.display()))?;

    copy_files(staged, &prefix)
}

pub fn clean_temp(pkg: &Package) -> Result<(), String> {
    let cache = cache_dir()
        .ok_or_else(|| "cannot find cache dir".to_string())?
        .join("nitrite");

    for dir in ["tmp", "build"] {
        let path = cache
            .join(dir)
            .join(format!("{}-{}", pkg.name, pkg.version));

        if path.exists() {
            fs::remove_dir_all(&path)
                .map_err(|err| format!("cannot clean {} -> {err}", path.display()))?;
        }
    }

    Ok(())
}

pub fn unpack_package(pkg: &Package) -> Result<PathBuf, String> {
    let archive = download_archive(pkg)?;

    let tmp = cache_dir()
        .ok_or_else(|| "cannot find cache dir".to_string())?
        .join("nitrite")
        .join("tmp")
        .join(format!("{}-{}", pkg.name, pkg.version));

    if tmp.exists() {
        fs::remove_dir_all(&tmp)
            .map_err(|err| format!("cannot clean {} -> {err}", tmp.display()))?;
    }

    extract_archive(&archive, &tmp)?;

    let pkginfo_path = tmp.join("PKGINFO");
    pkgfile_difference(pkg, &pkginfo_path)?;

    build_package(pkg, &tmp)
}

