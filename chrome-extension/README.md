# My Launcher Chrome Tab Connector

This Chrome extension enables My Launcher to access and switch to Chrome tabs.

## Features

- Lists all open Chrome tabs
- Allows My Launcher to switch to specific tabs
- Secure communication via Native Messaging API

## Installation

### 1. Build the Native Host

```bash
# From the project root directory
cargo build --release --target x86_64-pc-windows-gnu
```

### 2. Load the Chrome Extension

1. Open Chrome and navigate to `chrome://extensions/`
2. Enable "Developer mode" (toggle in the top right)
3. Click "Load unpacked"
4. Select the `chrome-extension` directory
5. Note the Extension ID that appears (you'll need this for the next step)

### 3. Install the Native Host

Run PowerShell as Administrator:

```powershell
# From the project root directory
.\install-native-host.ps1 -ExtensionId YOUR_EXTENSION_ID_HERE
```

Replace `YOUR_EXTENSION_ID_HERE` with the actual extension ID from step 2.

### 4. Test the Connection

1. The extension should now be able to communicate with My Launcher
2. Start My Launcher and switch to Browser mode
3. Type to search through your open Chrome tabs
4. Select a tab to switch to it

## Troubleshooting

### Extension not connecting

1. Check that the extension ID in the registry matches the actual extension ID
2. Ensure the native host executable exists at the specified path
3. Check Chrome's console for error messages (right-click extension icon → Inspect)

### Permission errors

- Make sure you ran the PowerShell script as Administrator
- Check that the manifest file paths are correct in the registry

## Uninstallation

To remove the Native Messaging Host registration:

```powershell
# Run as Administrator
.\uninstall-native-host.ps1
```

Then remove the extension from Chrome's extension management page.

## Security

- The extension only has access to tab information (title, URL)
- Communication is restricted to the registered native host
- No external network requests are made