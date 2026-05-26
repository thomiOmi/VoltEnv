use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

/// Modul untuk menangani semua kebutuhan path file di VoltEnv.
/// Kami menggunakan folder khusus di home directory (~/.voltenv/bin)
/// untuk menyimpan binary on-demand.
pub struct VoltPath;

impl VoltPath {
    /// Mengambil path root untuk VoltEnv (biasanya ~/.voltenv atau setara di OS lain)
    pub fn root() -> PathBuf {
        ProjectDirs::from("com", "voltenv", "app")
            .map(|proj| proj.data_local_dir().to_path_buf())
            .unwrap_or_else(|| {
                // Fallback jika ProjectDirs gagal
                let mut path = dirs::home_dir().unwrap_or_default();
                path.push(".voltenv");
                path
            })
    }

    /// Path ke direktori binary (~/.voltenv/bin)
    pub fn bin_dir() -> PathBuf {
        let mut path = Self::root();
        path.push("bin");
        path
    }

    /// Memastikan folder binary ada, jika tidak maka dibuat
    pub fn ensure_dirs() -> std::io::Result<()> {
        let bin_path = Self::bin_dir();
        if !bin_path.exists() {
            fs::create_dir_all(&bin_path)?;
        }
        Ok(())
    }

    /// Mendapatkan path lengkap untuk sebuah binary tertentu dengan ekstensi otomatis
    pub fn get_binary_path(name: &str) -> PathBuf {
        let mut path = Self::bin_dir();

        #[cfg(target_os = "windows")]
        let filename = format!("{}.exe", name);

        #[cfg(not(target_os = "windows"))]
        let filename = name.to_string();

        path.push(filename);
        path
    }

    /// Mengecek apakah binary sudah terinstall (didownload)
    #[allow(dead_code)]
    pub fn is_binary_present(name: &str) -> bool {
        Self::get_binary_path(name).exists()
    }
}
