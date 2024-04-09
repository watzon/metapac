use anyhow::{bail, Context, Result};
use std::collections::HashSet;

use crate::{backend::backend_trait::Switches, Package};

#[derive(Debug, Clone)]
pub struct Rustup {
    pub(crate) packages: HashSet<Package>,
}

#[derive(Debug)]
pub enum Repotype {
    Toolchain,
    Component,
}

impl Repotype {
    fn try_from<T>(value: T) -> Result<Self>
    where
        T: AsRef<str>,
    {
        let value = value.as_ref();
        let result = match value {
            "toolchain" => Self::Toolchain,
            "component" => Self::Component,
            _ => bail!("{} is neither toolchain nor component", value),
        };
        Ok(result)
    }

    pub fn get_install_switches(self) -> Switches {
        match self {
            Self::Toolchain => &["toolchain", "install"],
            Self::Component => &["component", "add", "--toolchain"],
        }
    }

    pub fn get_remove_switches(self) -> Switches {
        match self {
            Self::Toolchain => &["toolchain", "uninstall"],
            Self::Component => &["component", "remove", "--toolchain"],
        }
    }

    pub fn get_info_switches(self) -> Switches {
        match self {
            Self::Toolchain => &["toolchain", "list"],
            Self::Component => &["component", "list", "--installed", "--toolchain"],
        }
    }
}

/// A package as used exclusively in the rustup backend. Contrary to other packages, this does not
/// have an (optional) repository and a name, but is either a component or a toolchain, has a
/// toolchain version, and if it is a toolchain also a name.
#[derive(Debug)]
pub struct RustupPackage {
    /// Whether it is a toolchain or a component.
    pub repotype: Repotype,
    /// The name of the toolchain this belongs to (stable, nightly, a pinned version)
    pub toolchain: String,
    /// If it is a toolchain, it will not have a component name.
    /// If it is a component, this will be its name.
    pub component: Option<String>,
}

impl RustupPackage {
    /// Creates a new [`RustupPackage`].
    ///
    /// # Panics
    ///
    /// Panics if
    /// - repotype is Toolchain and component is Some, or
    /// - repotype is Component and component is None.
    fn new(repotype: Repotype, toolchain: String, component: Option<String>) -> Self {
        match repotype {
            Repotype::Toolchain => assert!(component.is_none()),
            Repotype::Component => assert!(component.is_some()),
        };

        Self {
            repotype,
            toolchain,
            component,
        }
    }
}

impl TryFrom<&Package> for RustupPackage {
    type Error = anyhow::Error;

    fn try_from(package: &Package) -> Result<Self> {
        let repo = package.repo.as_ref().context("getting repo from package")?;
        let repotype = Repotype::try_from(repo).context("getting repotype")?;

        let (toolchain, component) = match repotype {
            Repotype::Toolchain => (package.name.to_string(), None),
            Repotype::Component => {
                let (toolchain, component) = package
                    .name
                    .split_once('/')
                    .context("splitting package into toolchain and component")?;
                (toolchain.to_string(), Some(component.into()))
            }
        };

        Ok(Self::new(repotype, toolchain, component))
    }
}
