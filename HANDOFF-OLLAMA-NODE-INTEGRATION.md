# Handoff: Ollama Integration & Node UI Improvements

## Summary

This session added Ollama (local AI) integration to rhizos-node and improved the IPFS/storage controls:
1. **Ollama Manager** - Full Ollama detection, installation, and model management
2. **AI Models UI** - Card for managing local AI models (pull, delete, start/stop)
3. **IPFS Storage Limits** - Configurable storage limit for IPFS datastore
4. **Setup Wizard Improvements** - 5-step wizard covering storage, IPFS, and Ollama setup

## What Was Built

### Repository: https://github.com/Huck-dev/rhizos-node

### New Files

**`src/ollama-manager.ts`** - Complete Ollama lifecycle management:
```typescript
export class OllamaManager extends EventEmitter {
  // Detection
  detectOllamaPath(): Promise<void>    // Finds Ollama binary on system
  isInstalled(): boolean               // Check if Ollama is available
  getVersion(): Promise<string | null> // Get Ollama version

  // Server Management
  checkRunning(): Promise<boolean>     // Check if Ollama server running
  start(): Promise<void>               // Start Ollama server
  stop(): Promise<void>                // Stop Ollama server

  // Model Management
  getModels(): Promise<OllamaModel[]>  // List installed models
  pullModel(name, onProgress): Promise<void>  // Download a model
  deleteModel(name): Promise<void>     // Remove a model

  // Installation
  install(onProgress): Promise<void>   // Download & install Ollama
  setOllamaPath(path): boolean         // Manually set Ollama location
}
```

### Updated Files

**`src/node-service.ts`** - Added Ollama integration:
- `getOllamaStatus()` - Returns full status including models
- `installOllama(onProgress)` - Download and install Ollama
- `startOllama()` / `stopOllama()` - Server control
- `pullOllamaModel(name, onProgress)` - Download models
- `deleteOllamaModel(name)` - Remove models
- `setOllamaPath(path)` / `getOllamaPath()` - Path management

**`src/ipfs-manager.ts`** - Added storage limit control:
```typescript
setStorageLimit(limitGb: number): Promise<void>  // Set max IPFS storage
getStorageLimit(): Promise<number | null>        // Get current limit
```

**`src/main.ts`** - Added IPC handlers:
- `ollama-status` - Get Ollama status
- `ollama-install` - Install Ollama
- `ollama-start` / `ollama-stop` - Server control
- `ollama-pull-model` / `ollama-delete-model` - Model management
- `ollama-set-path` / `ollama-get-path` - Path configuration
- `browse-for-file` - File browser dialog
- `ipfs-set-storage-limit` / `ipfs-get-storage-limit` - IPFS limits

**`src/preload.ts`** - Exposed APIs to renderer:
- All Ollama operations
- `browseForFile(options)` - Native file picker
- `setIPFSStorageLimit(gb)` / `getIPFSStorageLimit()`

**`src/index.html`** - UI updates:
- **Ollama AI Models Card** - Status, start/stop, model list, pull/delete
- **IPFS Storage Limit** - Slider control (5-500 GB)
- **Setup Wizard** - 5 steps: Welcome → Storage → IPFS → Ollama → Complete
- **Bug Fix** - IPFS button now enables after drive selection

## UI Features

### Ollama AI Models Card
- Status badge (Running/Stopped/Not installed)
- Start/Stop toggle button
- List of installed models with size info
- Delete button for each model
- Dropdown to select and pull new models:
  - Llama 3.2 (1B, 3B)
  - Llama 3.1 8B
  - Mistral 7B
  - Code Llama 7B
  - Phi-3 Mini
  - Gemma 2 2B
  - Qwen 2.5 3B
  - DeepSeek Coder
- Progress bar during model download

### IPFS Storage Limit
- Slider: 5-500 GB range
- Apply button to save setting
- Located in IPFS card below stats

### Setup Wizard Flow
```
Step 1: Welcome
  - Overview of what will be configured
  ↓
Step 2: Storage Selection
  - Drive picker with free space info
  ↓
Step 3: IPFS Setup
  - Auto-downloads IPFS binary (~50MB)
  - Shows progress bar
  ↓
Step 4: Ollama Setup
  - Detects GPU
  - Downloads & runs Ollama installer if needed
  - Browse option if auto-detection fails
  ↓
Step 5: Complete
  - Summary of configured components
  - Auto-starts IPFS and Ollama
```

## Ollama Installation Logic

### Windows Installation Flow
```
1. Check if already installed → Skip if yes
2. Download OllamaSetup.exe via curl.exe (~250MB)
3. Open installer with shell.openPath() (visible UI)
4. Poll every 3 seconds for up to 3 minutes
5. Detect installation via:
   - Common paths (LOCALAPPDATA, Program Files, etc.)
   - PATH lookup via 'where ollama' (5s timeout)
6. If not detected → Show browse button for manual selection
```

### Detection Paths (Windows)
```
%LOCALAPPDATA%\Programs\Ollama\ollama.exe
%LOCALAPPDATA%\Ollama\ollama.exe
%PROGRAMFILES%\Ollama\ollama.exe
C:\Program Files\Ollama\ollama.exe
C:\Ollama\ollama.exe
```

## Bug Fixes

1. **IPFS button not enabling** - Added `updateIPFSStatus()` call after drive selection
2. **Ollama installer popup when already installed** - Added check at start of install
3. **Wizard freezing at 90%** - Added timeout to PATH detection, better error handling
4. **Done button disabled** - Fixed button state management in wizard steps

## API Reference

### Ollama Status Response
```typescript
interface OllamaStatus {
  installed: boolean;
  version?: string;
  running: boolean;
  models: OllamaModel[];
  endpoint?: string;  // http://127.0.0.1:11434 when running
}

interface OllamaModel {
  name: string;
  size: number;           // bytes
  quantization?: string;
  family?: string;
  parameterSize?: string;
  modifiedAt?: string;
}
```

### IPC Events
```typescript
// Progress events (renderer listens)
'ollama-install-progress' → (percent: number)
'ollama-pull-progress' → ({ model: string, status: string, percent?: number })
'ollama-status-change' → (OllamaStatus)
```

## Commits

**rhizos-node:**
- `fcdc77e` - feat: Add Ollama integration and IPFS storage controls

## Known Issues / TODOs

1. **Linux Ollama Install** - Uses curl pipe to sh, may need sudo
2. **macOS Ollama Install** - Currently throws, directs user to manual download
3. **Model Size Estimates** - Dropdown shows approximate sizes, actual may vary
4. **Ollama Auto-Start** - Could optionally start Ollama with the node
5. **Model Recommendations** - Could suggest models based on GPU VRAM

## Testing

### Prerequisites
1. Windows 10/11 with GPU (optional but recommended)
2. ~5GB free disk space for a small model

### Test Steps
1. Build: `npx tsc && npm run copy-html && npx electron-builder --win`
2. Run installer or launch from `release/win-unpacked/`
3. Complete setup wizard:
   - Select storage drive
   - Let IPFS download
   - Install Ollama (or skip if already installed)
4. After wizard:
   - Verify IPFS card shows "Ready to start"
   - Verify Ollama card shows status
5. Pull a model:
   - Select "Llama 3.2 1B" from dropdown
   - Click "Pull Model"
   - Wait for download (1.3 GB)
6. Start Ollama server
7. Verify model appears in list

## Next Steps

1. **Ollama API Integration** - Use local models for workspace AI features
2. **Model Presets** - Workspace-specific model recommendations
3. **VRAM Monitoring** - Show GPU memory usage during inference
4. **Model Sharing** - Share custom models between workspace nodes via IPFS
5. **Remote Model Management** - Pull/delete models from OtherThing dashboard
