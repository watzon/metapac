use std::collections::{BTreeMap, BTreeSet};

use color_eyre::Result;
use serde::{Deserialize, Serialize};

use crate::cmd::{command_found, run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Flatpak;

#[derive(Debug, Clone)]
pub struct FlatpakQueryInfo {
    pub systemwide: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FlatpakInstallOptions {}

impl Backend for Flatpak {
    type QueryInfo = FlatpakQueryInfo;
    type InstallOptions = FlatpakInstallOptions;

    fn map_managed_packages(
        packages: BTreeMap<String, Self::InstallOptions>,
        _: &Config,
    ) -> Result<BTreeMap<String, Self::InstallOptions>> {
        Ok(packages)
    }

    fn query_installed_packages(_: &Config) -> Result<BTreeMap<String, Self::QueryInfo>> {
        if !command_found("flatpak") {
            return Ok(BTreeMap::new());
        }

        let sys_explicit_out = run_command_for_stdout(
            [
                "flatpak",
                "list",
                "--system",
                "--app",
                "--columns=application",
            ],
            Perms::Same,
        )?;
        let sys_explicit = sys_explicit_out
            .lines()
            .map(|x| (x.trim().to_owned(), FlatpakQueryInfo { systemwide: true }));

        let user_explicit_out = run_command_for_stdout(
            [
                "flatpak",
                "list",
                "--user",
                "--app",
                "--columns=application",
            ],
            Perms::Same,
        )?;
        let user_explicit = user_explicit_out
            .lines()
            .map(|x| (x.trim().to_owned(), FlatpakQueryInfo { systemwide: false }));

        let sys_explicit_runtimes_out =
            run_command_for_stdout(["flatpak", "pin", "--system"], Perms::Same)?;
        let sys_explicit_runtimes = sys_explicit_runtimes_out.lines().skip(1).map(|x| {
            (
                x.trim().split('/').nth(1).unwrap().to_owned(),
                FlatpakQueryInfo { systemwide: true },
            )
        });

        let user_explicit_runtimes_out =
            run_command_for_stdout(["flatpak", "pin", "--user"], Perms::Same)?;
        let user_explicit_runtimes = user_explicit_runtimes_out.lines().skip(1).map(|x| {
            (
                x.trim().split('/').nth(1).unwrap().to_owned(),
                FlatpakQueryInfo { systemwide: false },
            )
        });

        let all = sys_explicit
            .chain(user_explicit)
            .chain(sys_explicit_runtimes)
            .chain(user_explicit_runtimes)
            .collect();

        Ok(all)
    }

    fn install_packages(
        packages: &BTreeMap<String, Self::InstallOptions>,
        no_confirm: bool,
        config: &Config,
    ) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                [
                    "flatpak",
                    "install",
                    if config.flatpak_systemwide {
                        "--system"
                    } else {
                        "--user"
                    },
                ]
                .into_iter()
                .chain(Some("--assumeyes").filter(|_| no_confirm))
                .chain(packages.keys().map(String::as_str)),
                Perms::Sudo,
            )?;
        }

        Ok(())
    }

    fn remove_packages(
        packages: &BTreeSet<String>,
        no_confirm: bool,
        config: &Config,
    ) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                [
                    "flatpak",
                    "uninstall",
                    if config.flatpak_systemwide {
                        "--system"
                    } else {
                        "--user"
                    },
                ]
                .into_iter()
                .chain(Some("--assumeyes").filter(|_| no_confirm))
                .chain(packages.iter().map(String::as_str)),
                Perms::Sudo,
            )?;
        }

        Ok(())
    }
}
