# My Launcher Chrome Tab Connector

This Chrome extension enables My Launcher to access and switch to Chrome tabs.

## Features

- Lists all open Chrome tabs
- Allows My Launcher to switch to specific tabs
- Secure communication via Native Messaging API
- Fast 500ms polling for responsive tab switching
- Comprehensive error handling and debug logging
- Visual feedback in launcher during tab switch

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
4. Look for "Connected to native host" message in console

### Tab switching not working

1. Open Chrome DevTools Console (F12) and look for:
   - `=== CHROME EXTENSION: Message received ===`
   - `=== TAB SWITCH COMMAND DETECTED ===`
   - `=== EXECUTING TAB SWITCH ===`
   - `=== TAB SWITCH SUCCESSFUL ===`
2. If you don't see these messages, reload the extension
3. Check that the native host binary is up to date
4. Run My Launcher with `RUST_LOG=debug` for detailed logs

### Permission errors

- Make sure you ran the PowerShell script as Administrator
- Check that the manifest file paths are correct in the registry

### Debugging tips

1. **Chrome Extension Console**: Shows all extension activity
2. **My Launcher Logs**: Run with `RUST_LOG=debug` environment variable
3. **Common issues**:
   - Extension needs reload after code changes
   - Native host binary needs update after rebuild
   - Chrome must not be in "Do Not Disturb" mode

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

## Technical Details

### Message Flow

1. **Tab Listing**:
   - Extension polls native host every 500ms with `getTabs` command
   - Native host communicates with launcher via IPC
   - Tab list is updated in launcher

2. **Tab Switching**:
   - User selects tab in launcher
   - Command queued in TabManager
   - Next poll retrieves command from queue
   - Extension switches to tab using Chrome APIs
   - Success acknowledgment sent back

### Debug Mode

To enable detailed logging:
1. Chrome Extension: Open DevTools Console (F12)
2. My Launcher: Set `RUST_LOG=debug` environment variable
3. Look for messages prefixed with `===` for major events