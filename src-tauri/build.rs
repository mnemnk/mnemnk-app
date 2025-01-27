fn main() -> Result<(), Box<dyn std::error::Error>> {
    tauri_build::build();
    Ok(())
}
