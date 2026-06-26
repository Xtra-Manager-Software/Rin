use clap::Parser;
use colored::Colorize;
use rpkg::DEFAULT_PREFIX;
use rpkg::manager::PackageManager;
use std::io::Read;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser, Debug)]
#[command(name = "rpkg", version, about = "Rin Package Manager")]
struct Cli {
    #[arg(long, default_value = DEFAULT_PREFIX)]
    prefix: PathBuf,

    #[arg(short = 'S', long)]
    sync: bool,

    #[arg(short = 'R', long)]
    remove: bool,

    #[arg(short = 'Q', long)]
    query: bool,

    #[arg(short = 'y', long)]
    refresh: bool,

    #[arg(short = 'u', long)]
    sysupgrade: bool,

    #[arg(short = 's', long)]
    search: bool,

    #[arg(short = 'f', long)]
    force: bool,

    targets: Vec<String>,
}

fn handle_multicall() {
    let mut args = std::env::args();
    let Some(arg0) = args.next() else { return };
    let exe_path = PathBuf::from(&arg0);
    let Some(exe_name) = exe_path.file_name().and_then(|s| s.to_str()) else {
        return;
    };

    match exe_name {
        "rpkg" | "rpkg-real" | "rpkg_cli" | "librpkg_cli.so" => {}
        _ => execute_proxied_binary(&exe_path, exe_name, args),
    }
}

fn elf_class(path: &Path) -> Option<u8> {
    let mut f = std::fs::File::open(path).ok()?;
    let mut buf = [0u8; 5];
    f.read_exact(&mut buf).ok()?;
    if buf[0..4] != *b"\x7FELF" {
        return None;
    }
    Some(buf[4])
}

fn execute_proxied_binary(exe_path: &Path, exe_name: &str, args: std::env::Args) -> ! {
    let original_path = if exe_path.parent().is_none_or(|p| p.as_os_str().is_empty())
        || exe_path.parent().unwrap().as_os_str() == "."
    {
        PathBuf::from(DEFAULT_PREFIX)
            .join("usr")
            .join("bin")
            .join(exe_name)
    } else {
        exe_path.to_path_buf()
    };

    let mut current = original_path;
    while let Ok(target) = std::fs::read_link(&current) {
        let next = if target.is_absolute() {
            target
        } else {
            current.parent().unwrap().join(target)
        };
        if next.file_name().and_then(|n| n.to_str()) == Some("rpkg") {
            break;
        }
        current = next;
    }

    let target_elf = PathBuf::from(format!("{}.elf", current.display()));

    // If no .elf proxy exists:
    if !target_elf.exists() {
        // Proxy symlink (still points to rpkg) but .elf is missing — broken install
        if std::fs::read_link(&current).is_ok() {
            eprintln!(
                "rpkg proxy: {} is a proxy symlink but {}.elf is missing;\n\
                 reinstall the package to fix this",
                current.display(),
                current.display(),
            );
            std::process::exit(1);
        }
        // Resolved package binary (e.g. /usr/lib/nvim-0.10/bin/nvim) — try multi-strategy exec
        let lib_path = PathBuf::from(DEFAULT_PREFIX).join("usr").join("lib");
        let args_vec: Vec<String> = args.collect();
        let class = elf_class(&current);
        let is_elf = class.is_some();

        // Strategy 1: direct exec
        if is_elf {
            let err = Command::new(&current)
                .args(&args_vec)
                .env("LD_LIBRARY_PATH", &lib_path)
                .exec();
            log::warn!("direct exec of {} failed ({}), retrying via linker", current.display(), err);
        }

        // Strategy 2: linker exec
        if is_elf {
            let linker = if class == Some(2) { "/system/bin/linker64" } else { "/system/bin/linker" };
            let err = Command::new(linker)
                .arg(&current)
                .args(&args_vec)
                .env("LD_LIBRARY_PATH", &lib_path)
                .exec();
            log::warn!("linker exec of {} failed ({}), retrying via shell", current.display(), err);
        }

        // Strategy 3: shell exec (last resort)
        let err = Command::new("/system/bin/sh")
            .arg(&current)
            .args(&args_vec)
            .env("LD_LIBRARY_PATH", &lib_path)
            .exec();
        eprintln!(
            "rpkg proxy: failed to exec {}: {}",
            current.display(),
            err
        );
        std::process::exit(1);
    }

    let resolved_name = current.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let mut multicall_args = Vec::new();
    if resolved_name != exe_name {
        match resolved_name {
            "coreutils" => multicall_args.push(format!("--coreutils-prog={}", exe_name)),
            "busybox" | "toybox" => multicall_args.push(exe_name.to_string()),
            _ => {}
        }
    }

    let args_vec: Vec<String> = args.collect();
    let lib_path = PathBuf::from(DEFAULT_PREFIX).join("usr").join("lib");
    let class = elf_class(&target_elf);
    let is_elf = class.is_some();

    // Strategy 1: direct exec (ELF with valid PT_INTERP, or scripts with shebang)
    if is_elf {
        let err = Command::new(&target_elf)
            .args(&multicall_args)
            .args(&args_vec)
            .env("LD_LIBRARY_PATH", &lib_path)
            .exec();
        log::warn!("direct exec of {} failed ({}), retrying via linker", target_elf.display(), err);
    }

    // Strategy 2: exec via system linker (handles noexec mount, broken PT_INTERP)
    if is_elf {
        let linker = if class == Some(2) { "/system/bin/linker64" } else { "/system/bin/linker" };
        let err = Command::new(linker)
            .arg(&target_elf)
            .args(&multicall_args)
            .args(&args_vec)
            .env("LD_LIBRARY_PATH", &lib_path)
            .exec();
        log::warn!("linker exec of {} failed ({}), retrying via shell", target_elf.display(), err);
    }

    // Strategy 3: exec via system shell (last resort)
    let err = Command::new("/system/bin/sh")
        .arg(&target_elf)
        .args(&multicall_args)
        .args(&args_vec)
        .env("LD_LIBRARY_PATH", &lib_path)
        .exec();

    eprintln!(
        "rpkg proxy: failed to exec {}: {}",
        target_elf.display(),
        err
    );
    std::process::exit(1);
}

fn run_operation(cli: Cli) -> anyhow::Result<()> {
    let mut pm = PackageManager::new(&cli.prefix)?;

    if cli.sync {
        if cli.refresh {
            pm.sync()?;
        }

        if cli.search {
            for query in &cli.targets {
                let results = pm.search(query)?;
                if results.is_empty() {
                    continue;
                }
                for pkg in &results {
                    let name_styled = pkg.name.bold();
                    let ver_styled = pkg.version.green().bold();
                    let installed_tag =
                        if pm.list_installed().iter().any(|i| i.info.name == pkg.name) {
                            " [installed]".cyan().bold()
                        } else {
                            "".normal()
                        };
                    println!("rin/{} {} {}", name_styled, ver_styled, installed_tag);
                    println!("    {}", pkg.description);
                }
            }
            return Ok(());
        }

        if cli.sysupgrade {
            pm.upgrade()?;
        }

        if !cli.targets.is_empty() {
            pm.install(&cli.targets, cli.force)?;
        }
    } else if cli.remove {
        if !cli.targets.is_empty() {
            pm.remove(&cli.targets)?;
        }
    } else if cli.query {
        for pkg in pm.list_installed() {
            println!(
                "{} {}",
                pkg.info.name.bold(),
                pkg.info.version.green().bold()
            );
        }
    } else {
        println!(
            "{}",
            "error: no operation specified (use -h for help)"
                .red()
                .bold()
        );
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    #[cfg(target_os = "android")]
    {
        let perm_file = std::path::Path::new(DEFAULT_PREFIX).join(".storage_permission");
        if !perm_file.exists() {
            eprintln!("\n\x1b[31m\x1b[1mError: Storage permission required!\x1b[0m");
            eprintln!("\x1b[33mRun 'rin-perm-storage' to grant access before using rpkg\x1b[0m\n");
            std::process::exit(1);
        }
    }

    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp(None)
        .init();

    handle_multicall();
    run_operation(Cli::parse())
}
