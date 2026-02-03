mod gui;

use anyhow::Result;

fn main() -> Result<()> {
    // Run the GUI application
    gui::run_app()?;
    Ok(())
}
