# 🔨 Build Guide - EasyProject MCP Server

## ⚠️ Ring Crate Build Issue

Váš projekt používá `ring` crate (kryptografická knihovna), která má známé problémy s buildem na Windows:

```
Error: called `Result::unwrap()` on an `Err` value: Os { code: 183, kind: AlreadyExists, message: "Nelze vytvořit soubor, který již existuje." }
```

Tento problém nastává kvůli:
- Chybějící Visual Studio Build Tools
- Konflikty v temp souborech ring crate
- Race conditions při parallel build

## 🚀 Řešení (od nejsnazšího)

### 1. **GitHub Actions (Doporučeno)**

Automatický build v cloudu - žádné lokální problémy!

**Postup:**
1. Pushněte kód na GitHub
2. GitHub Actions automaticky sestaví Windows EXE
3. Stáhněte hotový EXE z Artifacts

**Setup:**
```bash
git add .
git commit -m "Add GitHub Actions build"
git push origin main
```

Jděte na GitHub → Actions → "Build Windows EXE" → Download artifacts

### 2. **Lokální build s fix**

**A) Instalace Visual Studio Build Tools:**
```powershell
# Automaticky
.\setup-build-tools.ps1

# Nebo manuálně
winget install Microsoft.VisualStudio.2022.BuildTools
```

**B) Restart terminal a zkuste:**
```powershell
.\deploy.ps1 -Force
```

### 3. **Cross-kompilace z Linux/WSL**

Pokud máte WSL2 nebo Linux:

```bash
# Install Rust with Windows target
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add x86_64-pc-windows-gnu

# Install mingw cross-compiler
sudo apt install gcc-mingw-w64-x86-64

# Build Windows EXE
cargo build --release --target x86_64-pc-windows-gnu
```

### 4. **Používání pre-built EXE**

Pokud máte přístup k již sestaveného EXE:

```powershell
# Zkopírujte EXE do target/release/
mkdir target\release -Force
copy path\to\easyproject-mcp-server.exe target\release\

# Vytvořte deployment
.\deploy.ps1 -SkipBuild
```

## 🛠️ Troubleshooting

### Ring Crate Cache Clear

```powershell
# Vymazat cache
Remove-Item -Recurse -Force $env:CARGO_HOME\registry\src\index.crates.io-1949cf8c6b5b557f\ring-0.17.14 -ErrorAction SilentlyContinue

# Clean build
cargo clean
```

### Single-threaded Build

```powershell
$env:CARGO_BUILD_JOBS = "1"
$env:RING_PREGENERATE_ASM = "1"
cargo build --release
```

### MSVC vs GNU Toolchain

```powershell
# MSVC (doporučeno pro Windows)
rustup toolchain install stable-x86_64-pc-windows-msvc
rustup default stable-x86_64-pc-windows-msvc
cargo build --release

# GNU (fallback)
rustup default stable-x86_64-pc-windows-gnu
cargo build --release
```

## 📋 Build Status Check

Zkontrolujte váš build environment:

```powershell
# Rust info
rustup show
cargo --version

# Build tools check
where link.exe  # Měl by najít Visual Studio linker

# Test basic build
cargo check
```

## 🎯 Rychlé řešení

**Pro okamžité použití:**

1. **GitHub Actions** - Push na GitHub, počkejte na build, stáhněte EXE
2. **Použijte deployment skript** - `.\deploy.ps1 -SkipBuild` (pokud už máte EXE)
3. **Setup build tools** - `.\setup-build-tools.ps1` + restart terminal

## 📊 Proč ring crate selhává?

Ring crate:
- Kompiluje C/Assembly kód pro kryptografii
- Potřebuje C compiler (MSVC nebo MinGW)
- Vytváří temp soubory, které mohou kolidovat
- Má specifické požadavky na Windows build prostředí

**Náš deployment skript má workarounds:**
- Single-threaded build
- Cache clearing
- Multiple retry attempts
- Fallback na existující EXE

## ✅ Doporučený workflow

1. **Vývoj:** Použijte GitHub Actions pro build
2. **Testing:** `.\deploy.ps1 -SkipBuild` s pre-built EXE
3. **Production:** GitHub releases s automaticky sestavenými binárkami

Tímto způsobem obejdete všechny lokální build problémy! 🎉 