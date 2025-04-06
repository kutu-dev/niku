const BYTES_IN_A_KIBIBYTE: u64 = 1024;

pub(crate) fn format_bytes_to_string(size: u64) -> String {
    if size < BYTES_IN_A_KIBIBYTE {
        format!("{} B", size)
    } else {
        format!("{} KiB", size / BYTES_IN_A_KIBIBYTE)
    }
}
