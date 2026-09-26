use crate::permissions;

pub(crate) fn permission_status_str(status: &permissions::OSPermissionStatus) -> &'static str {
    match status {
        permissions::OSPermissionStatus::NotNeeded => "not_needed",
        permissions::OSPermissionStatus::Empty => "not_requested",
        permissions::OSPermissionStatus::Granted => "granted",
        permissions::OSPermissionStatus::Denied => "denied",
    }
}
