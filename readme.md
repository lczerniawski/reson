# Reson System Monitor

Reson is a terminal-based system monitor written in Rust.
The idea behind the project is to provide an efficient and insightful monitoring tool for your system's resources, that is easy to use and understand.
It uses a text-based UI to display various system statistics such as CPU usage, memory usage, running processes, disk usage, and network activity.
The application leverages asynchronous tasks (via Tokio) to simultaneously handle user input, refresh the system data, and update the UI.

![Application Screenshot](screenshots/app_window.png)

## Why "Reson"?

The name "Reson" is derived from combining the words "resource" and "monitor."

## Features

- **CPU Usage Dashboard:** Displays a bar chart for per-CPU usage with horizontal scrolling.
- **Memory Gauges:** Shows memory (RAM) and swap usage with gauges.
- **Processes Table:** Lists running processes, sorted by a combined CPU and memory score. Supports vertical scrolling.
- **Disk Usage:** Displays disk usage details and sorts disks by usage. Supports vertical scrolling.
- **Network Widget:** Displays network throughput and packet counts along with other network details. Supports vertical scrolling.
- **Keyboard Navigation:**
  - Use arrow keys (or h/j/k/l) to scroll the active widget.
  - Press Tab/Shift+Tab to switch between tabs (CPU, Processes, Disks, Networks).
  - Press `q` or `Esc` (or Ctrl+c) to quit the application.
- **Responsive Layout:** Automatic layout update based on terminal size.
- **Mouse Support:**
  - Move mouse on tabs to switch between them.
  - Scroll to scroll within widgets.
- **Process Sorting**

## Process Sorting

The process table features comprehensive sorting capabilities:

**Available Sort Criteria:**
- **1:** Username - Sort by process owner
- **2:** PID - Sort by Process ID
- **3:** PPID - Sort by Parent Process ID
- **4:** CPU Usage - Sort by processor utilization
- **5:** Memory Usage - Sort by RAM consumption
- **6:** Start Time - Sort by process launch time
- **7:** Process Name - Sort alphabetically by name

**How to Sort:**
1. Navigate to the Processes tab
2. Press the number key (1-7) corresponding to your desired sort criterion
3. Each keypress cycles through the sort states:
   - First press: Ascending order (▲)
   - Second press: Descending order (▼)
   - Third press: Returns to unsorted state

The current sort criterion and direction are indicated in the column header with an arrow symbol.

## Installation

1. Download the appropriate archive for your platform from the [Releases page](https://github.com/yourusername/your-repo/releases)
2. Extract the archive:
   - For Linux/macOS: `tar -xzf reson_version_Platform.tar.gz`
   - For Windows: Extract the zip file
3. Run the executable:
   - Linux/macOS: `./reson`
   - Windows: Double-click `reson.exe` or run from command prompt

## Usage

Run the application:
   cargo run --release

Once running, you can use the following keys:
- Use ←/→ or h/l to scroll horizontally (CPU tab).
- Use ↑/↓ or j/k to scroll vertically (Processes, Disks, Networks).
- Use 1-7 to change sorting order.
- Press Tab or Shift+Tab to change the active tab.
- Press `q` (or Esc) to quit.

## Contributing

Contributions, issues, and feature requests are welcome!
Feel free to check the [issues page](https://github.com/lczerniawski/reson/issues) if you want to contribute.

## License

Distributed under the MIT License. See [LICENSE](LICENSE) for more information.
