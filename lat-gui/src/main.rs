slint::include_modules!();

use chrono::{DateTime, Local};
use lat_7z::SevenZCompressor;
use lat_core::{ArchiveEntry, Compressor};
use lat_format::LatCompressor;
use lat_gpu_cuda::CudaAccelerator;
use lat_gpu_vulkan::VulkanAccelerator;
use lat_paqg::PaqgCompressor;
use lat_zip::ZipCompressor;
use rfd::FileDialog;
use slint::{Color, Model, ModelRc, SharedString, VecModel};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    // --- State ---
    let files_model = Rc::new(VecModel::<FileEntry>::default());
    ui.set_files(ModelRc::new(files_model.clone()));

    // --- GPU Detection ---
    let (gpu_name, gpu_color, accelerator) = detect_gpu();
    ui.set_gpu_status(SharedString::from(gpu_name));
    ui.set_gpu_color(gpu_color);
    let accelerator: Option<Arc<dyn lat_core::GpuAccelerator>> = accelerator;

    // --- Callbacks ---

    let ui_handle = ui.as_weak();
    let files_model_clone = files_model.clone();
    ui.on_add_clicked(move || {
        let ui = ui_handle.unwrap();
        if let Some(paths) = FileDialog::new().pick_files() {
            for path in paths {
                if let Ok(metadata) = fs::metadata(&path) {
                    let name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    let size = if metadata.is_dir() {
                        "DIR".to_string()
                    } else {
                        format_size(metadata.len())
                    };
                    let date = format_date(metadata.modified().ok());

                    files_model_clone.push(FileEntry {
                        name: name.into(),
                        size: size.into(),
                        date: date.into(),
                        path: path.to_string_lossy().to_string().into(),
                    });
                }
            }
            ui.set_status_text(
                format!("Added files. Total: {}", files_model_clone.row_count()).into(),
            );
        }
    });

    let ui_handle = ui.as_weak();
    let files_model_clone = files_model.clone();
    ui.on_delete_clicked(move || {
        let ui = ui_handle.unwrap();
        let index = ui.get_selected_index();
        if index >= 0 && (index as usize) < files_model_clone.row_count() {
            files_model_clone.remove(index as usize);
            ui.set_selected_index(-1);
            ui.set_status_text("Item removed".into());
        }
    });

    let ui_handle = ui.as_weak();
    let files_model_clone = files_model.clone();
    let accel_clone = accelerator.clone();
    ui.on_compress_clicked(move |format| {
        let ui = ui_handle.unwrap();
        let format_str = format.to_string();

        let dest = FileDialog::new()
            .set_file_name(format!(
                "archive.{}",
                format_str.to_lowercase().trim_start_matches('.')
            ))
            .save_file();

        if let Some(dest_path) = dest {
            ui.set_status_text(format!("Compressing to {}...", format_str).into());

            // Bolt ⚡ Optimization: Pre-allocate the entries vector with the known number of files.
            // This avoids multiple expensive reallocations and memcpys during the collection phase.
            let count = files_model_clone.row_count();
            let mut entries = Vec::with_capacity(count);
            for i in 0..count {
                if let Some(file) = files_model_clone.row_data(i) {
                    let path = PathBuf::from(file.path.as_str());
                    if let Ok(data) = fs::read(&path) {
                        entries.push(ArchiveEntry {
                            name: file.name.to_string(),
                            data,
                        });
                    }
                }
            }

            let compressor: Box<dyn Compressor> = match format_str.as_str() {
                "7z" => Box::new(SevenZCompressor),
                ".lat" => Box::new(LatCompressor::new(accel_clone.clone())),
                "PAQG" => Box::new(PaqgCompressor::new(accel_clone.clone())),
                _ => Box::new(ZipCompressor),
            };

            match compressor.compress(&entries, None) {
                Ok(data) => {
                    if let Err(e) = fs::write(dest_path, data) {
                        ui.set_status_text(format!("Error: {}", e).into());
                    } else {
                        ui.set_status_text(
                            format!("Successfully compressed to {}", format_str).into(),
                        );
                    }
                }
                Err(e) => {
                    ui.set_status_text(format!("Compression failed: {}", e).into());
                }
            }
        }
    });

    let ui_handle = ui.as_weak();
    let accel_clone = accelerator.clone();
    ui.on_extract_clicked(move || {
        let ui = ui_handle.unwrap();
        if let Some(archive_path) = FileDialog::new().pick_file()
            && let Some(dest_dir) = FileDialog::new().pick_folder()
        {
            ui.set_status_text("Decompressing...".into());

            let ext = archive_path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            let compressor: Box<dyn Compressor> = match ext {
                "7z" => Box::new(SevenZCompressor),
                "lat" => Box::new(LatCompressor::new(accel_clone.clone())),
                "paq" => Box::new(PaqgCompressor::new(accel_clone.clone())),
                _ => Box::new(ZipCompressor),
            };

            match fs::read(&archive_path) {
                Ok(archive_data) => match compressor.decompress(&archive_data, None) {
                    Ok(entries) => {
                        // Bolt ⚡ Optimization: Use a HashSet to cache created directories.
                        // This avoids redundant and expensive create_dir_all syscalls when
                        // extracting many files into the same subdirectory.
                        let mut created_dirs = HashSet::with_capacity(entries.len() / 4);
                        for entry in entries {
                            let path = dest_dir.join(entry.name);
                            if let Some(parent) = path.parent()
                                && !created_dirs.contains(parent)
                            {
                                let _ = fs::create_dir_all(parent);
                                created_dirs.insert(parent.to_path_buf());
                            }
                            let _ = fs::write(path, entry.data);
                        }
                        ui.set_status_text("Extraction complete".into());
                    }
                    Err(e) => ui.set_status_text(format!("Decompression failed: {}", e).into()),
                },
                Err(e) => ui.set_status_text(format!("Error reading archive: {}", e).into()),
            }
        }
    });

    let ui_handle = ui.as_weak();
    let files_model_clone = files_model.clone();
    ui.on_test_clicked(move || {
        let ui = ui_handle.unwrap();
        let index = ui.get_selected_index();
        if index >= 0
            && (index as usize) < files_model_clone.row_count()
            && let Some(file) = files_model_clone.row_data(index as usize)
        {
            let path = PathBuf::from(file.path.as_str());
            ui.set_status_text(format!("Testing {}...", file.name).into());
            if let Ok(data) = fs::read(&path) {
                if ZipCompressor.decompress(&data, None).is_ok() {
                    ui.set_status_text("Archive integrity verified (ZIP)".into());
                } else if SevenZCompressor.decompress(&data, None).is_ok() {
                    ui.set_status_text("Archive integrity verified (7z)".into());
                } else {
                    ui.set_status_text("Could not verify archive format".into());
                }
            }
        }
    });

    let ui_handle = ui.as_weak();
    let files_model_clone = files_model.clone();
    ui.on_info_clicked(move || {
        let ui = ui_handle.unwrap();
        let index = ui.get_selected_index();
        if index >= 0
            && (index as usize) < files_model_clone.row_count()
            && let Some(file) = files_model_clone.row_data(index as usize)
        {
            ui.set_status_text(
                format!(
                    "File: {} | Size: {} | Path: {}",
                    file.name, file.size, file.path
                )
                .into(),
            );
        }
    });

    ui.run()
}

fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size >= GB {
        format!("{:.1} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.1} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.1} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}

fn format_date(modified: Option<std::time::SystemTime>) -> String {
    match modified {
        Some(time) => {
            let datetime: DateTime<Local> = time.into();
            datetime.format("%Y-%m-%d %H:%M:%S").to_string()
        }
        None => "Unknown".to_string(),
    }
}

fn detect_gpu() -> (
    &'static str,
    Color,
    Option<Arc<dyn lat_core::GpuAccelerator>>,
) {
    if let Ok(cuda) = CudaAccelerator::new() {
        return (
            "CUDA (Active)",
            Color::from_rgb_u8(46, 204, 113),
            Some(Arc::new(cuda)),
        );
    }

    let vulkan_future = VulkanAccelerator::new();
    if let Ok(vulkan) = pollster::block_on(vulkan_future) {
        return (
            "Vulkan (Active)",
            Color::from_rgb_u8(52, 152, 219),
            Some(Arc::new(vulkan)),
        );
    }

    ("None (CPU)", Color::from_rgb_u8(231, 76, 60), None)
}
