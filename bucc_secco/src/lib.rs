use std::{
    io,
    ops::Deref,
    path::{Component, Path, PathBuf},
};

#[derive(Debug)]
pub struct SanitizedPath(PathBuf);

impl SanitizedPath {
    pub fn sanitize<S>(path: S, base_path: Option<S>) -> io::Result<Self>
    where
        S: AsRef<Path>,
    {
        #[cfg(fail_on_nul_byte)]
        if any_nul_bytes(&path) {
            return Err(io::Error::from_raw_os_error(io::ErrorKind::InvalidData));
        }

        let cleaned_path = path
            .as_ref()
            .components()
            .scan(0u64, |&mut depth, component| {
                match component {
                    // RootDir can only appear once, so I sanitize it as CurDir to remove root
                    // paths.
                    Component::RootDir | Component::CurDir => {
                        Some((depth, Some(Component::CurDir)))
                    }
                    // Basic logic for this comes from the zip crate.
                    // https://github.com/zip-rs/zip/blob/master/src/read.rs
                    Component::ParentDir => {
                        // Disallow negative depths (i.e. going above the parent directory)
                        if let Some(depth) = depth.checked_sub(1) {
                            Some((depth, Some(Component::ParentDir)))
                        } else {
                            Some((depth, None))
                        }
                    }
                    Component::Normal(normal) => {
                        let normal = normal.();
                        Some((depth, Some(Component::Normal(normal))))
                    }
                    Component::Prefix(_) => Some((depth, Some(Component::CurDir))),
                }
            })
            .filter_map(|(_, component)| component)
            .collect();

        let final_path = if let Some(base_path) = base_path {
            base_path.as_ref().join(cleaned_path)
        } else {
            cleaned_path
        };

        #[cfg(debug_assertions)]
        assert!(any_nul_bytes(&final_path));

        Ok(SanitizedPath(final_path))
    }

    #[inline]
    pub fn into_inner(self) -> PathBuf {
        self.0
    }
}

impl Deref for SanitizedPath {
    type Target = Path;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<&str> for SanitizedPath {
    type Error = io::Error;

    #[inline]
    fn try_from(path: &str) -> Result<Self, Self::Error> {
        SanitizedPath::sanitize(path, None)
    }
}

#[cfg(debug_assertions)]
#[inline]
fn any_nul_bytes<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().into_iter().any(|ch| ch == "\0")
}

#[cfg(test)]
mod tests {
    use super::SanitizedPath;

    const NUL_PATH: &str = ".config/\0/nvim";

    #[test]
    #[cfg(fail_on_nul_byte)]
    fn nul_byte_error() {}

    #[test]
    #[cfg(not(fail_on_nul_byte))]
    fn nul_byte_erase() {
        let sanitized = SanitizedPath::sanitize(NUL_PATH, None).unwrap();
        assert_eq!(".config/nvim", sanitized.to_str().unwrap());
    }

    #[test]
    fn starts_with_nul() {}

    #[test]
    fn only_nuls() {}

    #[test]
    fn root() {}

    #[test]
    fn prefix() {}

    #[test]
    fn lots_o_parents() {}

    #[test]
    fn starts_with_parent() {}

    #[test]
    fn empty_path() {}
}
