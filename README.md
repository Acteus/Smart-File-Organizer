# Smart File Organizer

A modern, cross-platform desktop application for automatically organizing and managing your files. Built with Tauri, Svelte, and Rust for performance, small build size, and a great user experience.

![Smart File Organizer](https://via.placeholder.com/800x450.png?text=Smart+File+Organizer)

## Features

- **🔍 Real-Time Folder Monitoring**: Automatically detect and organize new files as they appear
- **🏷️ Custom Tagging System**: Create, assign, and filter by tags to easily find files
- **⚡ Intelligent File Organization**: Automatically sort files based on extension or custom rules
- **☁️ Cloud Backup (Optional)**: Securely backup important files to AWS S3
- **🖥️ Native Performance**: Blazing fast operations with minimal resource usage
- **⚙️ Extensible Rules**: Define custom organization rules based on file patterns
- **🧩 Drag-and-Drop Interface**: Intuitive UI for manual file management
- **🔄 Cross-Platform**: Works on macOS and Windows

## Installation

### Requirements

- [Node.js](https://nodejs.org/) (v16 or higher)
- [Rust](https://www.rust-lang.org/tools/install) (stable)
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites)

### Setup

1. Clone the repository:

```bash
git clone https://github.com/yourusername/smart-file-organizer.git
cd smart-file-organizer
```

2. Run the installation script:

```bash
# Make the script executable
chmod +x install.sh

# Run the script
./install.sh
```

3. Start the development server:

```bash
npm run tauri dev
```

## Usage

### Monitoring Folders

1. Click "Select Folder" to choose a directory to monitor
2. Click "Start Monitoring" to begin watching for new files
3. New files will be automatically organized based on your rules

### Organizing Files Manually

1. Use the drag-and-drop interface to organize files manually
2. Drag files into the application to trigger automatic organization

### Tagging Files

1. Select files in the file browser
2. Click the tag icon to add or remove tags
3. Use the sidebar to filter files by tag

### Cloud Backup

1. Navigate to the Cloud Backup section
2. Configure your AWS S3 credentials
3. Select folders to back up
4. Choose backup frequency

## Building for Production

To create a production-ready build:

```bash
npm run tauri build
```

This will create installable packages for your current platform in the `src-tauri/target/release/bundle` directory.

## Architecture

The application is built using:

- **Tauri**: Core framework connecting Rust backend with Svelte frontend
- **Svelte**: Fast, reactive UI framework with minimal overhead
- **Rust**: High-performance backend handling file operations and database
- **SQLite**: Local database for storing file metadata and tags
- **AWS SDK for Rust**: Optional cloud functionality

## Directory Structure

```
smart-file-organizer/
├── src/                     # Svelte frontend
│   ├── routes/              # Svelte pages
│   └── app.css              # Global styles (Tailwind)
├── src-tauri/               # Rust backend
│   ├── src/                 # Rust source code
│   │   ├── commands.rs      # Tauri command handlers
│   │   ├── database.rs      # SQLite integration
│   │   ├── file_ops.rs      # File monitoring and operations
│   │   ├── cloud_sync.rs    # AWS S3 integration
│   │   ├── utils.rs         # Utility functions
│   │   └── main.rs          # Application entry point
│   ├── Cargo.toml           # Rust dependencies
│   └── tauri.conf.json      # Tauri configuration
├── tailwind.config.js       # Tailwind CSS configuration
├── package.json             # NPM dependencies
└── README.md                # You are here!
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
