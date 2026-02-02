slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    ui.on_compress_clicked(|| {
        println!("Compress button clicked!");
        // Logic to trigger compression
    });

    ui.on_decompress_clicked(|| {
        println!("Decompress button clicked!");
        // Logic to trigger decompression
    });

    ui.run()
}
