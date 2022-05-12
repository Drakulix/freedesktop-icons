//! # freedesktop-incons
//!
//! This crate provides a [freedesktop icon](https://specifications.freedesktop.org/icon-theme-spec/icon-theme-spec-latest.html#implementation_notes) lookup implementation.
//!
//! It exposes a single [`lookup`] function to find icon based on their, `name`, `theme`, `size` and `scale`.
//!
//! ## Example
//!
//! **Simple lookup:**
//!
//! The following snippet get an icon from the default 'hicolor' theme
//! with the default scale (`1`) and the default size (`24`).
//!
//! ```rust
//! # fn main() {
//! use freedesktop_icons::lookup;
//!
//! let icon = lookup("firefox").find();
//! # }
//!```
//!
//! **Complex lookup:**
//!
//! If you have specific requirement for your lookup you can use the provided builder functions:
//!
//! ```rust
//! # fn main() {
//! use freedesktop_icons::lookup;
//!
//! let icon = lookup("firefox")
//!     .with_size(48)
//!     .with_scale(2)
//!     .with_theme("Arc")
//!     .find();
//! # }
//!```
use crate::cache::CACHE;
use crate::theme::{try_build_icon_path, THEMES};
use std::path::PathBuf;

mod cache;
mod theme;

/// The lookup builder struct, holding all the lookup query parameters.
pub struct LookupBuilder<'a> {
    name: &'a str,
    scale: u16,
    size: u16,
    theme: Option<&'a str>,
}

/// Build an icon lookup for the given icon name.
///
/// ## Example
/// ```rust
/// # fn main() {
/// use freedesktop_icons::lookup;
///
/// let icon = lookup("firefox").find();
/// # }
pub fn lookup(name: &str) -> LookupBuilder {
    LookupBuilder::new(name)
}

impl<'a> LookupBuilder<'a> {
    /// Restrict the lookup to the given icon size.
    ///
    /// ## Example
    /// ```rust
    /// # fn main() {
    /// use freedesktop_icons::lookup;
    ///
    /// let icon = lookup("firefox")
    ///     .with_size(48)
    ///     .find();
    /// # }
    pub fn with_size(mut self, scale: u16) -> Self {
        self.scale = scale;
        self
    }

    /// Restrict the lookup to the given scale.
    ///
    /// ## Example
    /// ```rust
    /// # fn main() {
    /// use freedesktop_icons::lookup;
    ///
    /// let icon = lookup("firefox")
    ///     .with_scale(2)
    ///     .find();
    /// # }
    pub fn with_scale(mut self, size: u16) -> Self {
        self.size = size;
        self
    }

    /// Add the given theme to the current lookup :
    /// ## Example
    /// ```rust
    /// # fn main() {
    /// use freedesktop_icons::lookup;
    ///
    /// let icon = lookup("firefox")
    ///     .with_theme("Papirus")
    ///     .find();
    /// # }
    pub fn with_theme<'b: 'a>(mut self, theme: &'b str) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Execute the current lookup
    /// if no icon is found in the current theme fallback to
    /// `/usr/shar/hicolor` theme and then to `/usr/share/pixmaps`.
    pub fn find(self) -> Option<PathBuf> {
        // Lookup for an icon in the given theme and fallback to 'hicolor' default theme
        let icon = self
            .theme
            .and_then(|theme| self.lookup_in_theme(theme))
            .or_else(|| self.lookup_in_theme("hicolor"));

        // Return the icon if found
        if icon.is_some() {
            return icon;
        };

        // Last chance
        try_build_icon_path(self.name, "/usr/share/pixmaps")
    }

    fn new<'b: 'a>(name: &'b str) -> Self {
        Self {
            name,
            scale: 1,
            size: 24,
            theme: None,
        }
    }

    // Recursively lookup for icon in the given theme and its parents
    fn lookup_in_theme(&self, theme: &str) -> Option<PathBuf> {
        let cached = CACHE.get(theme, self.size, self.scale, self.name);
        if cached.is_some() {
            return cached;
        }

        THEMES.get(theme).and_then(|icon_theme| {
            icon_theme
                .try_get_icon(self.name, self.size, self.scale)
                .or_else(|| {
                    icon_theme
                        .inherits()
                        .and_then(|parent| self.lookup_in_theme(parent))
                })
                .map(|icon| {
                    CACHE.insert(theme, self.size, self.scale, self.name, &icon);
                    icon
                })
        })
    }
}

#[cfg(test)]
mod test {
    use crate::lookup;
    use speculoos::prelude::*;
    use std::path::PathBuf;

    #[test]
    fn simple_lookup() {
        let firefox = lookup("firefox").find();
        assert_that!(firefox).is_some();
    }

    #[test]
    fn compare_to_linincon_with_theme() {
        let lin_wireshark = linicon::lookup_icon("wireshark")
            .next()
            .unwrap()
            .unwrap()
            .path;

        let wireshark = lookup("wireshark")
            .with_size(16)
            .with_scale(1)
            .with_theme("Papirus")
            .find();

        assert_that!(wireshark).is_some().is_equal_to(lin_wireshark)
    }

    #[test]
    fn compare_to_linicon_in_pixmap() {
        let archlinux_logo = lookup("archlinux-logo")
            .with_size(16)
            .with_scale(1)
            .with_theme("Papirus")
            .find();

        assert_that!(archlinux_logo)
            .is_some()
            .is_equal_to(PathBuf::from("/usr/share/pixmaps/archlinux-logo.png"));
    }
}