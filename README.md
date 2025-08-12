[![Latest Release](https://img.shields.io/github/v/release/lungu19/shears)](https://github.com/lungu19/Shears/releases)[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

# Shears ✂️
Are you tired of each Old Siege version taking up 50-60 GB (or more) of your valuable disk space? Shears is a simple utility designed to significantly reduce the file size of your Old Siege installations while keeping the game fully functional at your desired quality level. It can free up a massive amount of disk space, often reducing the total size by 50-75%.

![Shears](https://raw.githubusercontent.com/lungu19/shears/refs/heads/main/assets/screenshot1.png)

## How?
Shears achieves this by intelligently deleting unnecessary files, such as high-quality texture ones. For instance, if you only play on 'Low' texture settings, there's no need to keep the 'Medium', 'High', and 'Ultra' textures, which consume gigabytes of space. Shears lets you remove them safely.

## 🚀Instructions
> [!WARNING]
> ⚠️ Important Note: This Process is Irreversible.
> **'Shearing' a game installation is a destructive and irreversible action.** The tool permanently deletes files from the game's installation.
>
> To reverse this process will have to verify your installation and **re-download** any affected files.

Using Shears is incredibly easy:
 1. **Select  the Siege Folder:** Select the Siege folder using the button or by dragging and dropping it into the tool
 2. **Choose what to keep:** Select the **highest** texture quality you want to **keep**. All textures for quality levels above your selection will be deleted. 
 3. **Shear!:** Click the Shear button and you're done!

## 📥 Download

You can download the latest pre-compiled version of Shears from the [**Releases Page**](https://github.com/lungu19/shears/releases).

## 🛠️ Building from Source

If you prefer to build the application yourself, follow these steps.

### Prerequisites

- A working and up-to-date [Rust installation](https://www.rust-lang.org/tools/install).
- Rust **nightly** toolchain and `x86_64-pc-windows-msvc` target installed:
	```batch
	rustup target add x86_64-pc-windows-msvc
	rustup toolchain install nightly
	rustup component add rust-src --toolchain nightly
	```

### Steps
> [!IMPORTANT]
> Note that Shears uses aggresive size-saving techniques such as building the standard library from source and using link-time optimizations.
1.  Clone the repository:
    ```batch
    git clone https://github.com/lungu19/shears.git
    cd shears
    ```
2.  Build the project:

	```batch
	set RUSTFLAGS=-Zlocation-detail=none -Zfmt-debug=none
	cargo +nightly build -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --target x86_64-pc-windows-msvc --release
	```

The executable will be located in the `target/x86_64-pc-windows-msvc/` folder.

## 🤝 Contributing

Contributions, issues, and feature requests are welcome! Feel free to check the [issues page](https://github.com/lungu19/shears/issues).

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](https://github.com/lungu19/shears/LICENSE) file for details.
