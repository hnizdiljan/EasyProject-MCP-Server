# Vyřešení problémů s kompilací Rust na Windows

## 🚨 Problém
```
error: linker `link.exe` not found
note: the msvc targets depend on the msvc linker but `link.exe` was not found
note: please ensure that Visual Studio 2017 or later, or Build Tools for Visual Studio were installed with the Visual C++ option.
```

## ✅ Řešení

### Možnost 1: Dokončení instalace Visual Studio Build Tools

1. **Otevřete Start Menu** a vyhledejte "Visual Studio Installer"
2. **Spusťte Visual Studio Installer**
3. **Klikněte na "Modify"** u Visual Studio BuildTools 2022
4. **Zaškrtněte tyto komponenty**:
   - ✅ C++ build tools
   - ✅ Windows 10/11 SDK (nejnovější verze)
   - ✅ CMake tools for Visual Studio
   - ✅ MSVC v143 - VS 2022 C++ x64/x86 build tools

5. **Klikněte "Install"** a počkejte na dokončení
6. **Restartujte PowerShell/terminal**

### Možnost 2: Přepnutí na GNU toolchain (RYCHLEJŠÍ)

Pokud nechcete instalovat Visual Studio Build Tools, můžete použít GNU toolchain:

```powershell
# 1. Přidejte GNU target
rustup target add x86_64-pc-windows-gnu

# 2. Nainstalujte MinGW-w64
winget install msys2.msys2

# 3. Po instalaci MSYS2, otevřete MSYS2 terminal a spusťte:
# pacman -S mingw-w64-x86_64-gcc

# 4. Přidejte MinGW do PATH (v PowerShell):
$env:PATH += ";C:\msys64\mingw64\bin"

# 5. Nastavte Rust aby používal GNU toolchain:
rustup default stable-x86_64-pc-windows-gnu
```

### Možnost 3: Ruční instalace Build Tools

```powershell
# Stáhněte a nainstalujte Build Tools manuálně
Invoke-WebRequest -Uri "https://aka.ms/vs/17/release/vs_buildtools.exe" -OutFile "vs_buildtools.exe"

# Spusťte s automatickou instalací C++ tools
.\vs_buildtools.exe --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended
```

## 🧪 Test po instalaci

Po dokončení kterékoliv z možností, otestujte kompilaci:

```powershell
# Vyčistěte předchozí build
cargo clean

# Zkuste znovu buildnout
cargo build --release
```

## 🔍 Ověření nastavení

```powershell
# Zkontrolujte dostupné targets
rustup show

# Zkontrolujte, že link.exe je dostupný
where link.exe

# Zkontrolujte verzi kompilátoru
rustc --version --verbose
```

## 💡 Rychlé řešení: Přepnutí na GNU (DOPORUČENO PRO VÝVOJ)

Pokud jen chcete rychle pokračovat ve vývoji:

```powershell
# 1. Přidejte GNU target
rustup target add x86_64-pc-windows-gnu

# 2. Nastavte default
rustup default stable-x86_64-pc-windows-gnu

# 3. Build s GNU
cargo build --release --target x86_64-pc-windows-gnu
```

Toto je rychlejší a nevyžaduje instalaci Visual Studio. 