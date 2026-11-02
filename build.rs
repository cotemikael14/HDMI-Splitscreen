fn main() {
    // Ajoute l'icône LOGO.ico dans le .exe Windows.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/LOGO.ico");
        if let Err(e) = res.compile() {
            println!("cargo:warning=Icône non ajoutée : {e}");
        }
    }
}
