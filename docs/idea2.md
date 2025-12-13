This is a great pivot. **Zed** is the gold standard for modern, high-performance UI right now. Its aesthetic is defined by **chromeless windows**, **muted distinctive grays**, **speed**, and **keyboard-centricity**.

To replicate the "Zed Feel" in a Tauri app (which uses HTML/CSS instead of Zed's GPUI), we need to be very specific about CSS layout, window configuration, and color theory.

Here is the **UI/UX Spec: The "Zed-Lite" Aesthetic**.

---

## 1. Window Architecture (The "Chromeless" Look)

Zed’s signature look is that the sidebar flows up behind the macOS "Traffic Lights" (Close/Min/Max buttons), and there is no distinct title bar.

### Tauri Configuration (`tauri.conf.json`)
You need to disable the system title bar but keep the controls overlaying the webview.

```json
"window": {
  "titleBarStyle": "Overlay", 
  "hiddenTitle": true,
  "trafficLightPosition": { "x": 12, "y": 18 },
  "width": 900,
  "height": 600,
  "decorations": true,
  "transparent": false
}
```

*   **Result:** Your HTML `<body>` fills 100% of the window. The red/yellow/green buttons float on top of your CSS.

---

## 2. The Color Palette (Zed One Dark)

Zed uses a very specific set of warm/neutral grays. You should hardcode these into your Tailwind config or CSS variables.

| UI Element | CSS Variable | Hex (Approx Zed Dark) |
| :--- | :--- | :--- |
| **App Background** | `--bg-app` | `#151515` |
| **Editor Background** | `--bg-editor` | `#191919` |
| **Sidebar Background** | `--bg-sidebar` | `#151515` (or slightly lighter `#1e1e1e`) |
| **Active Tab/Item** | `--bg-active` | `#252525` |
| **Border/Divider** | `--border-subtle` | `#333333` |
| **Text Primary** | `--text-main` | `#EBEBEC` |
| **Text Muted** | `--text-muted` | `#868686` |
| **Accent/Cursor** | `--accent` | `#5898F8` (Blue) or `#D98E48` (Gold) |

---

## 3. Layout Structure

Zed is strictly divided. For this scratchpad, we use a **Two-Pane Layout**.

```html
<div class="h-screen w-screen flex font-mono text-sm antialiased bg-[--bg-app] text-[--text-main]">
  
  <!-- LEFT SIDEBAR -->
  <!-- Padding-top ensures list doesn't sit under Traffic Lights -->
  <aside class="w-64 flex flex-col border-r border-[--border-subtle] pt-10">
    
    <!-- Buffer List -->
    <div class="flex-1 overflow-y-auto no-scrollbar">
      <!-- Item -->
      <div class="px-4 py-2 flex items-center justify-between cursor-pointer hover:bg-[--bg-active]">
        <div class="truncate text-[13px]">Project Notes</div>
        <span class="text-[10px] text-[--text-muted]">2m</span>
      </div>
      <!-- Active Item -->
      <div class="px-4 py-2 bg-[--bg-active] border-l-2 border-[--accent]">
        <div class="font-medium">SQL Snippets</div>
      </div>
    </div>
    
    <!-- Sidebar Footer (Status/Settings) -->
    <div class="h-10 border-t border-[--border-subtle] flex items-center px-4">
      <span class="text-xs text-[--text-muted]">24 buffers</span>
    </div>
  </aside>

  <!-- RIGHT EDITOR -->
  <main class="flex-1 flex flex-col bg-[--bg-editor]">
    
    <!-- Breadcrumb / Tab Bar (Draggable Region) -->
    <!-- data-tauri-drag-region is critical here -->
    <header data-tauri-drag-region class="h-10 flex items-center px-6 border-b border-[--border-subtle] select-none">
      <span class="text-xs text-[--text-muted]"> buffers / </span>
      <span class="text-xs ml-2 font-medium"> SQL Snippets </span>
    </header>

    <!-- CodeMirror Container -->
    <div id="editor" class="flex-1 overflow-hidden relative">
      <!-- CM6 mounts here -->
    </div>
  </main>

</div>
```

---

## 4. The Editor Styling (CodeMirror 6)

Zed’s editor is clean. To make CodeMirror look like Zed, you need to strip away the defaults.

**CSS overrides for CodeMirror:**
1.  **Font:** Use `Zed Mono`, `JetBrains Mono`, or `Fira Code`.
2.  **Cursor:** A solid block or a slightly wide bar, utilizing the accent color.
3.  **Gutters:** Minimalist. No background color on the line-number gutter, just muted text.

```css
.cm-editor {
  height: 100%;
  background-color: var(--bg-editor);
  color: var(--text-main);
}
.cm-gutters {
  background-color: transparent !important;
  border-right: none !important;
  color: var(--text-muted);
}
.cm-activeLine {
  background-color: rgba(255, 255, 255, 0.03) !important;
}
.cm-cursor {
  border-left-color: var(--accent) !important;
  border-left-width: 2px;
}
```

---

## 5. The "Command Palette" (The Zed Brain)

Zed doesn't really use buttons. It uses the Command Palette (`Cmd+Shift+P`) and the File Finder (`Cmd+P`).

Since you are building a "Flash" app, you should replicate this pattern:
1.  **Hide the Sidebar** by default (or make it toggleable).
2.  **The Trigger:** When user hits `Cmd+P`, open a modal overlay centered in the top-third of the window.
3.  **The Look:**
    *   Dark background with a slight blur (backdrop-filter).
    *   Input field with no border.
    *   List of buffers below it.
    *   Highlight matches in the text.

---

## 6. Micro-Interactions (The "Flash" Feel)

To make it feel like Zed and not a website:

1.  **No Scrollbars:** Use CSS to hide scrollbars (`::-webkit-scrollbar { display: none; }`) but keep scroll functionality. Zed relies on hover-state scrollbars or minimal indicators.
2.  **Focus States:** When the app window loses focus, dim the text color slightly (`opacity: 0.8`). When it gains focus, snap back to 100%.
3.  **Instant Search:** The search results in the sidebar/palette must update *synchronously* on keyup.

## 7. What we are cutting from Zed
To keep this "Flash":
*   **No Tabs:** Tabs encourage hoarding. A list is faster.
*   **No Terminal Panel:** Out of scope.
*   **No Chat/Collab:** Out of scope.

## Summary of the "Zed-Lite" Plan

1.  **Tauri:** `HiddenTitle` window style.
2.  **CSS:** Grid layout, Zed Dark palette, 1px borders.
3.  **Font:** JetBrains Mono (easiest free alternative to Zed Mono).
4.  **Components:**
    *   Sidebar (Left, inset traffic lights).
    *   Header (Top, drag region).
    *   Editor (Right, CodeMirror).
5.  **Behavior:** Everything autosaves. `Cmd+P` to switch buffers.

This will look incredible—a dedicated, high-speed writing tool that feels like a developer's weapon.