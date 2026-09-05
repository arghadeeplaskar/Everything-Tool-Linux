use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

/// Known virtual / pseudo filesystems in Linux that should not be indexed
pub const VIRTUAL_FS_TYPES: &[&str] = &[
    "proc",
    "sysfs",
    "devtmpfs",
    "devpts",
    "cgroup",
    "cgroup2",
    "pstore",
    "bpf",
    "configfs",
    "securityfs",
    "debugfs",
    "tracefs",
    "hugetlbfs",
    "mqueue",
    "fusectl",
    "autofs",
    "ramfs",
    "rpc_pipefs",
    "binfmt_misc",
    "efivarfs",
    "nsfs",
];

/// Storage filesystem types commonly encountered
pub const REAL_FS_TYPES: &[&str] = &[
    "ext4", "ext3", "ext2", "btrfs", "xfs", "f2fs", "zfs", "vfat", "msdos", "fat", "exfat",
    "ntfs", "ntfs3", "fuseblk", "hfsplus", "iso9660", "udf", "reiserfs",
];

/// Known snapshot or internal container paths to skip to avoid indexing millions of duplicates
pub const EXCLUDED_SUBPATH_PATTERNS: &[&str] = &[
    "/.snapshots",
    "/@snapshots",
    "/.btrfs-snapshots",
    "/timeshift/snapshots",
    "/var/lib/docker",
    "/var/lib/containerd",
    "/var/lib/flatpak/runtime",
    "/snap/",
];

#[derive(Debug, Clone)]
pub struct MountEntry {
    pub device: String,
    pub mount_point: PathBuf,
    pub fs_type: String,
    pub options: HashSet<String>,
    pub is_removable: bool,
    pub is_btrfs: bool,
    pub is_real_storage: bool,
}

#[derive(Debug, Clone, Default)]
pub struct MountTable {
    pub entries: Vec<MountEntry>,
}

impl MountTable {
    /// Loads current mount points from /proc/mounts (or fallback)
    pub fn load() -> Self {
        let mut entries = Vec::new();
        let file = match File::open("/proc/mounts") {
            Ok(f) => f,
            Err(_) => return Self { entries },
        };

        let reader = BufReader::new(file);
        for line in reader.lines().flatten() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 4 {
                continue;
            }

            let device = parts[0].to_string();
            let mount_point = PathBuf::from(parts[1]);
            let fs_type = parts[2].to_string();
            let options: HashSet<String> = parts[3].split(',').map(|s| s.to_string()).collect();

            let is_btrfs = fs_type == "btrfs";
            let is_real_storage = REAL_FS_TYPES.contains(&fs_type.as_str());

            // Check if mounted under typical removable drive paths
            let is_removable = mount_point.starts_with("/media")
                || mount_point.starts_with("/run/media")
                || mount_point.starts_with("/mnt")
                || options.contains("user")
                || options.contains("users");

            entries.push(MountEntry {
                device,
                mount_point,
                fs_type,
                options,
                is_removable,
                is_btrfs,
                is_real_storage,
            });
        }

        Self { entries }
    }

    /// Checks if a path is located on a virtual/pseudo filesystem
    pub fn is_virtual_fs(&self, path: &Path) -> bool {
        let mut best_match: Option<&MountEntry> = None;
        let mut best_len = 0;

        for entry in &self.entries {
            if path.starts_with(&entry.mount_point) {
                let len = entry.mount_point.as_os_str().len();
                if len > best_len {
                    best_len = len;
                    best_match = Some(entry);
                }
            }
        }

        if let Some(entry) = best_match {
            VIRTUAL_FS_TYPES.contains(&entry.fs_type.as_str())
        } else {
            false
        }
    }

    /// Checks if a path is on a removable storage device
    pub fn is_removable_drive(&self, path: &Path) -> bool {
        path.starts_with("/media") || path.starts_with("/run/media") || path.starts_with("/mnt")
    }

    /// Returns list of all active removable storage mount points
    pub fn removable_mounts(&self) -> Vec<PathBuf> {
        self.entries
            .iter()
            .filter(|e| e.is_removable && e.is_real_storage)
            .map(|e| e.mount_point.clone())
            .collect()
    }
}
