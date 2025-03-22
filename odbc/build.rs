#[cfg(windows)]
extern crate winres;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(windows)]
    {
        // the default for winres is to parse the version number from the neighboring Cargo.toml
        let mut res = winres::WindowsResource::new();
        res.set_version_info(winres::VersionInfo::FILETYPE, 2); // DLL file
        res.set("CompanyName", "MongoDB Inc.");
        // compile will write the rc file, and enable the cargo linker to link the compiled resource file
        res.compile()?;
    }
    Ok(())
}
