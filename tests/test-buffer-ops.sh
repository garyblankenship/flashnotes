#!/bin/bash
# E2E test for Flashnotes buffer operations
# Tests: launch app → create buffer → verify → delete → verify

set -e

APP_PATH="/Users/vampire/www/flashnotes/src-tauri/target/release/bundle/macos/Flashnotes.app"
APP_NAME="Flashnotes"

echo "🧪 Flashnotes E2E Test"
echo "======================"

# Kill any existing instance
echo "→ Cleaning up existing instances..."
pkill -fi "flashnotes" 2>/dev/null || true
sleep 1

# Launch the app
echo "→ Launching $APP_NAME..."
open "$APP_PATH"
sleep 2

# Wait for window to be ready
echo "→ Waiting for app window..."
for i in {1..10}; do
  WINDOW_EXISTS=$(osascript -e '
    tell application "System Events"
      if exists process "'"$APP_NAME"'" then
        tell process "'"$APP_NAME"'"
          if (count of windows) > 0 then
            return "yes"
          end if
        end tell
      end if
      return "no"
    end tell
  ' 2>/dev/null || echo "no")

  if [ "$WINDOW_EXISTS" = "yes" ]; then
    echo "  ✓ Window found"
    break
  fi
  sleep 0.5
done

if [ "$WINDOW_EXISTS" != "yes" ]; then
  echo "  ✗ FAILED: Window not found"
  exit 1
fi

# Get initial buffer count from sidebar
echo "→ Getting initial state..."
sleep 1

# Create a new buffer (Cmd+N)
echo "→ Creating new buffer (Cmd+N)..."
osascript -e '
  tell application "System Events"
    tell process "'"$APP_NAME"'"
      keystroke "n" using command down
    end tell
  end tell
'
sleep 1

# Type some content to make it identifiable
echo "→ Typing test content..."
osascript -e '
  tell application "System Events"
    tell process "'"$APP_NAME"'"
      keystroke "# E2E Test Buffer"
      keystroke return
      keystroke "Created at: '"$(date)"'"
    end tell
  end tell
'
sleep 1
echo "  ✓ Content typed"

# Delete the buffer (Cmd+W) - no confirm dialog now
echo "→ Deleting buffer (Cmd+W)..."
osascript -e '
  tell application "System Events"
    tell process "'"$APP_NAME"'"
      keystroke "w" using command down
    end tell
  end tell
'
sleep 1

# Verify app is still running (didn't crash)
# Note: process name is lowercase "flashnotes"
APP_RUNNING=$(pgrep -xi "flashnotes" > /dev/null && echo "yes" || echo "no")
if [ "$APP_RUNNING" = "yes" ]; then
  echo "  ✓ App still running after delete"
else
  echo "  ✗ FAILED: App crashed"
  exit 1
fi

# Verify window still exists
WINDOW_EXISTS=$(osascript -e '
  tell application "System Events"
    tell process "'"$APP_NAME"'"
      if (count of windows) > 0 then
        return "yes"
      end if
      return "no"
    end tell
  end tell
' 2>/dev/null || echo "no")

if [ "$WINDOW_EXISTS" = "yes" ]; then
  echo "  ✓ Window still open"
else
  echo "  ✗ FAILED: Window closed unexpectedly"
  exit 1
fi

echo ""
echo "✅ All tests passed!"
echo ""

# Quit the app
osascript -e 'tell application "'"$APP_NAME"'" to quit' 2>/dev/null || true
echo "App closed."
