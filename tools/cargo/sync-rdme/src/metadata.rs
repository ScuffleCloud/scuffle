use std::{collections::HashMap, convert::Infallible, fmt, str::FromStr, sync::Arc};

use serde::de::{Error, Visitor};

use super::de;

#[derive(Debug, Clone, Default, serde_derive::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Metadata {
    #[serde(default)]
    pub(crate) cargo_sync_rdme: CargoSyncRdme,
}

#[derive(Debug, Clone, Default, serde_derive::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct CargoSyncRdme {
    #[serde(default, deserialize_with = "de::string_or_seq")]
    pub(crate) extra_targets: Vec<String>,
    #[serde(default)]
    pub(crate) badge: Badge,
    #[serde(default)]
    pub(crate) rustdoc: Rustdoc,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Badge {
    pub(crate) style: Option<BadgeStyle>,
    pub(crate) badges: HashMap<Arc<str>, Arc<[BadgeItem]>>,
}

#[derive(Debug, Clone, Default, serde_derive::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) enum BadgeStyle {
    #[default]
    Plastic,
    Flat,
    FlatSquare,
    ForTheBadge,
    Social,
}

impl BadgeStyle {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Plastic => "plastic",
            Self::Flat => "flat",
            Self::FlatSquare => "flat-square",
            Self::ForTheBadge => "for-the-badge",
            Self::Social => "social",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BadgeItem {
    Maintenance,
    License(License),
    CratesIo,
    DocsRs,
    RustVersion,
    GithubActions(GithubActions),
    Codecov,
}

#[derive(Debug, Clone)]
enum BadgeKind {
    Maintenance,
    License,
    CratesIo,
    DocsRs,
    RustVersion,
    GithubActions,
    Codecov,
}

impl FromStr for BadgeKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let kind = match s {
            "maintenance" => Self::Maintenance,
            "license" => Self::License,
            "crates-io" => Self::CratesIo,
            "docs-rs" => Self::DocsRs,
            "rust-version" => Self::RustVersion,
            "github-actions" => Self::GithubActions,
            "codecov" => Self::Codecov,
            _ => {
                if s.starts_with("maintenance-") {
                    Self::Maintenance
                } else if s.starts_with("license-") {
                    Self::License
                } else if s.starts_with("crates-io-") {
                    Self::CratesIo
                } else if s.starts_with("docs-rs-") {
                    Self::DocsRs
                } else if s.starts_with("rust-version-") {
                    Self::RustVersion
                } else if s.starts_with("github-actions-") {
                    Self::GithubActions
                } else if s.starts_with("codecov-") {
                    Self::Codecov
                } else {
                    return Err(());
                }
            }
        };
        Ok(kind)
    }
}

impl BadgeKind {
    fn expecting() -> &'static [&'static str] {
        &[
            "maintenance",
            "license",
            "crates-io",
            "docs-rs",
            "rust-version",
            "github-actions",
            "codecov",
            "maintenance-*",
            "license-*",
            "crates-io-*",
            "docs-rs-*",
            "rust-version-*",
            "github-actions-*",
            "codecov-*",
        ]
    }
}

impl<'de> serde::Deserialize<'de> for Badge {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        fn deserialize_badge_list<'de, D>(deserializer: D) -> Result<Arc<[BadgeItem]>, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct BadgeList;

            impl<'de> Visitor<'de> for BadgeList {
                type Value = Arc<[BadgeItem]>;

                fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    formatter.write_str("map")
                }

                fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
                where
                    M: serde::de::MapAccess<'de>,
                {
                    let mut data = vec![];
                    while let Some(key) = map.next_key::<&str>()? {
                        let kind = BadgeKind::from_str(key)
                            .map_err(|_| M::Error::unknown_variant(key, BadgeKind::expecting()))?;
                        #[derive(serde_derive::Deserialize)]
                        #[serde(bound = "T: Default + serde::Deserialize<'de>")]
                        struct Wrap<T>(#[serde(deserialize_with = "de::bool_or_map")] Option<T>);

                        match kind {
                            BadgeKind::Maintenance => {
                                if map.next_value::<bool>()? {
                                    data.push(BadgeItem::Maintenance);
                                }
                            }
                            BadgeKind::License => {
                                if let Wrap(Some(license)) = map.next_value::<Wrap<License>>()? {
                                    data.push(BadgeItem::License(license));
                                }
                            }
                            BadgeKind::CratesIo => {
                                if map.next_value::<bool>()? {
                                    data.push(BadgeItem::CratesIo);
                                }
                            }
                            BadgeKind::DocsRs => {
                                if map.next_value::<bool>()? {
                                    data.push(BadgeItem::DocsRs);
                                }
                            }
                            BadgeKind::RustVersion => {
                                if map.next_value::<bool>()? {
                                    data.push(BadgeItem::RustVersion);
                                }
                            }
                            BadgeKind::GithubActions => {
                                if let Wrap(Some(github_actions)) =
                                    map.next_value::<Wrap<GithubActions>>()?
                                {
                                    data.push(BadgeItem::GithubActions(github_actions));
                                }
                            }
                            BadgeKind::Codecov => {
                                if map.next_value::<bool>()? {
                                    data.push(BadgeItem::Codecov);
                                }
                            }
                        }
                    }
                    Ok(data.into())
                }
            }

            deserializer.deserialize_any(BadgeList)
        }

        struct Badges;
        impl<'de> Visitor<'de> for Badges {
            type Value = Badge;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("map")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: serde::de::MapAccess<'de>,
            {
                #[derive(serde_derive::Deserialize)]
                struct BadgeList(
                    #[serde(deserialize_with = "deserialize_badge_list")] Arc<[BadgeItem]>,
                );

                let mut data = Badge::default();

                while let Some(key) = map.next_key::<String>()? {
                    let expected = &["badges", "badges-*", "style"];
                    match key.as_str() {
                        "style" => {
                            data.style = map.next_value()?;
                        }
                        _ => {
                            let key = if key == "badges" {
                                String::new()
                            } else if let Some(rest) = key.strip_prefix("badges-") {
                                rest.to_owned()
                            } else {
                                return Err(M::Error::unknown_field(&key, expected));
                            };
                            let value = map.next_value::<BadgeList>()?;
                            data.badges.entry(key.into()).or_insert(value.0);
                        }
                    }
                }

                Ok(data)
            }
        }

        deserializer.deserialize_any(Badges)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, serde_derive::Deserialize)]
pub(crate) struct License {
    #[serde(default)]
    pub(crate) link: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, serde_derive::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct GithubActions {
    #[serde(default, deserialize_with = "de::string_or_map_or_seq")]
    pub(crate) workflows: Vec<GithubActionsWorkflow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, serde_derive::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct GithubActionsWorkflow {
    #[serde(default)]
    pub(crate) name: Option<String>,
    pub(crate) file: String,
}

impl FromStr for GithubActionsWorkflow {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            name: None,
            file: s.to_string(),
        })
    }
}

#[derive(Debug, Clone, Default, serde_derive::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct Rustdoc {
    #[serde(default)]
    pub(crate) html_root_url: Option<String>,
    #[serde(default)]
    pub(crate) mappings: HashMap<String, String>,
}
