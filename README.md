# Robocopy+

A gorgeous, minimal, Native Windows GUI wrapper for `robocopy.exe`, written purely in Rust + Slint.

<p align="center">
  <img src="assets/icon.png" width="128" />
</p>

## Overview
Robocopy is arguably the most powerful native file-management utility built into the Windows Command Line, but its steep terminal learning curve forces users to memorize dozens of flags.

**Robocopy+** solves this by providing a lightweight, blisteringly fast GUI on top of the native Windows binary, abstracting away the CLI without losing any speed or safety.

### Core Features
- **Concurrent Execution Pipeline**: Slint UI runs off tracking channels, allowing `MainWindow` to initiate multiple heavy transfers into independent, spawnable `ProgressWindows` concurrently.
- **Save/Load OS Configurations**: Exposes built-in OS native dialogs via `rfd` perfectly mapping to Rust serialization config formats (`serde`) allowing you to securely "Save" and "Load" custom default routines exactly where you need them across your drives.
- **Job Cancellation Architecture**: Added an underlying polling mechanism on an `AtomicBool` mapping to send forceful Kill signals to stop a Robocopy operation safely mid-flight.
- **Advanced CLI Granularity**: Visually specify Multithreading (`/MT`), Retry limits (`/R`), Wait Times (`/W`), empty directory mapping (`/E` vs `/S`), Restartable Mode (`/Z`), and precise File Filtering (e.g., `*.txt`).
- **Real-Time Job Analytics**: Exposes real-time dual-progress mapping. Visually track both the current individual file, plus an accumulated count of total job completion (`Total Files: X / Y`).

---

## Installation 

### Option 1: Direct Download (Recommended)
Navigate to the [Releases page](https://github.com/dcryptoniun/robocpyplusgui/releases) and choose your preferred distribution:
1. `RobocopyPlus_Portable.zip` (standalone, requires zero installation setup)
2. `RobocopyPlus_Setup.exe` (full Windows Installer that automatically manages shortcuts)

### Option 2: Build From Source

#### Requirements
- [Rust & Cargo](https://rustup.rs/) (Stable)
- A Modern Desktop (Windows 10/11)

#### Compiling
Clone the repository and compile via Cargo:

```console
git clone https://github.com/dcryptoniun/robocpyplusgui.git
cd robocpyplusgui
cargo build --release
```

The resulting executable will map to `./target/release/robocpyplusgui.exe`.
