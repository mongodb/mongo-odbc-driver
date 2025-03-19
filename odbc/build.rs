#[cfg(windows)]
extern crate winres;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_version_info(winres::VersionInfo::FILETYPE, 2); // DLL file
        res.set("CompanyName", "MongoDB Inc.");
        res.compile()?;
    }
    Ok(())
}
