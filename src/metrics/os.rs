use std::fs;


#[derive(Debug, Clone, Default)]
pub struct OsRelease {
    pub pretty_name: String,
    pub name: String,
    pub version_id: String,
    pub version: String,
    pub codename: String,
    pub id: String,
    pub id_like: String,
    pub home_url: String,
    pub support_url: String,
    pub bug_report_url: String,
    pub privacy_policy_url: String,
    pub logo: String,
}


#[derive(Debug, Clone, Default)]
pub struct OsSnapshot {
    pub release: OsRelease,
}


pub struct OsCollector;


impl OsCollector {
    pub fn new() -> Self {
        Self
    }

    pub fn collect(&self) -> OsSnapshot {
        let mut snapshot = OsSnapshot::default();

        if let Ok(params) = fs::read_to_string("/etc/os-release") {
            for p in params.lines() {
                let p = p.trim();
                if p.is_empty() || p.starts_with('#') {
                    continue;
                }

                if let Some((key, value)) = p.split_once("=") {
                    let value = value.trim_matches('"').to_string();

                    match key {
                        "PRETTY_NAME" => snapshot.release.pretty_name = value,
                        "NAME" => snapshot.release.name = value,
                        "VERSION_ID" => snapshot.release.version_id = value,
                        "VERSION" => snapshot.release.version = value,
                        "VERSION_CODENAME" | "UBUNTU_CODENAME" => snapshot.release.codename = value,
                        "ID" => snapshot.release.id = value,
                        "ID_LIKE" => snapshot.release.id_like = value,
                        "HOME_URL" => snapshot.release.home_url = value,
                        "SUPPORT_URL" => snapshot.release.support_url = value,
                        "BUG_REPORT_URL" => snapshot.release.bug_report_url = value,
                        "PRIVACY_POLICY_URL" => snapshot.release.privacy_policy_url = value,
                        "LOGO" => snapshot.release.logo = value,
                        _ => {}
                    }

                }

            }
        }

        snapshot
    }
}