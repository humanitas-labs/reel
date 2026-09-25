use crate::{feeds::microphone::MicrophoneFeed, permissions};
use cap_recording::diagnostics::{
    CameraDiagnostics, CameraFormatInfo, DisplayDiagnostics, HardwareInfo, MicrophoneDiagnostics,
    StorageInfo,
};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LogUploadDiagnostics {
    hardware: HardwareInfo,
    system: cap_recording::diagnostics::SystemDiagnostics,
    displays: Vec<DisplayDiagnostics>,
    cameras: Vec<CameraDiagnostics>,
    microphones: Vec<MicrophoneDiagnostics>,
    storage: Option<StorageInfo>,
    permissions: PermissionsInfo,
    app_state: AppStateInfo,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PermissionsInfo {
    screen_recording: String,
    camera: String,
    microphone: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppStateInfo {
    is_recording: bool,
    recordings_dir: String,
    app_data_dir: String,
}

fn collect_cameras(has_permission: bool) -> Vec<CameraDiagnostics> {
    if !has_permission {
        return vec![];
    }

    cap_camera::list_cameras()
        .map(|camera| {
            let formats = camera
                .formats()
                .unwrap_or_default()
                .into_iter()
                .take(10)
                .map(|f| CameraFormatInfo {
                    width: f.width(),
                    height: f.height(),
                    frame_rate: f.frame_rate(),
                    pixel_format: f.pixel_format_name(),
                })
                .collect();

            CameraDiagnostics {
                device_id: camera.device_id().to_string(),
                display_name: camera.display_name().to_string(),
                model_id: camera.model_id().map(|m| m.to_string()),
                formats,
            }
        })
        .collect()
}

fn collect_microphones(has_permission: bool) -> Vec<MicrophoneDiagnostics> {
    if !has_permission {
        return vec![];
    }

    MicrophoneFeed::list()
        .into_iter()
        .map(|(name, (_device, config))| MicrophoneDiagnostics {
            name,
            sample_rate: config.sample_rate().0,
            channels: config.channels(),
            sample_format: format!("{:?}", config.sample_format()),
            // The richer capability/heuristic fields belong to the diagnostic
            // report; the log upload keeps its existing shape.
            is_default: None,
            is_bluetooth: None,
            is_usb: None,
            is_builtin: None,
            supported_configs: None,
        })
        .collect()
}

fn collect_storage_info(recordings_path: &std::path::Path) -> Option<StorageInfo> {
    use sysinfo::Disks;
    let disks = Disks::new_with_refreshed_list();

    let mut best_match: Option<(&sysinfo::Disk, usize)> = None;

    for disk in disks.iter() {
        if recordings_path.starts_with(disk.mount_point()) {
            let mount_point_len = disk.mount_point().as_os_str().len();
            if best_match.is_none_or(|(_, len)| mount_point_len > len) {
                best_match = Some((disk, mount_point_len));
            }
        }
    }

    best_match.map(|(disk, _)| StorageInfo {
        // The diagnostic report redacts these same paths, and both fields ride
        // in one upload -- leaving this one raw defeats the redaction.
        recordings_path: cap_recording::diagnostics::redact_home_paths(
            &recordings_path.display().to_string(),
        ),
        available_space_mb: disk.available_space() / (1024 * 1024),
        total_space_mb: disk.total_space() / (1024 * 1024),
    })
}

pub(crate) fn permission_status_str(status: &permissions::OSPermissionStatus) -> &'static str {
    match status {
        permissions::OSPermissionStatus::NotNeeded => "not_needed",
        permissions::OSPermissionStatus::Empty => "not_requested",
        permissions::OSPermissionStatus::Granted => "granted",
        permissions::OSPermissionStatus::Denied => "denied",
    }
}
