# Fix Windows Defender Build Issues

## Problem

Cargo build fails with "os error 32: The process cannot access the file"

This is caused by Windows Defender scanning files while Cargo tries to build.

## Solution: Add Exclusions (RECOMMENDED - No need to disable!)

### Step 1: Open Windows Security

1. Press `Windows + I` to open Settings
2. Go to **Update & Security** → **Windows Security**
3. Click **Virus & threat protection**

### Step 2: Add Folder Exclusions

1. Click **Manage settings** under "Virus & threat protection settings"
2. Scroll down to **Exclusions**
3. Click **Add or remove exclusions**
4. Click **+ Add an exclusion** → **Folder**

### Step 3: Add These Folders

Add the following 3 folders:

```
D:\penum\penum-private-rpc
C:\Users\Windows\.cargo
C:\Users\Windows\.rustup
```

### Step 4: Verify

After adding exclusions, run:

```powershell
cd D:\penum\penum-private-rpc
.\build.ps1
```

## Alternative: Single-threaded Build (If exclusions don't help)

```powershell
cd penum-rpc-gateway
cargo build --jobs 1

cd ..\penum-rpc-client
cargo build --jobs 1
```

This is slower but avoids race conditions.

## After Adding Exclusions

Your build WILL work. Then run:

```powershell
.\run.ps1    # Start both services
.\test.ps1   # Test everything works
```

## Security Note

These exclusions only affect build artifacts (compiled code). Your antivirus still protects:

- Source code (still scanned when you download/edit)
- Downloaded files
- Everything else on your system

This is a standard practice for all Rust developers on Windows.
