use windows::core::PCWSTR;

#[inline]
pub fn string_to_pcwstr(value: &str) -> PCWSTR {
    PCWSTR(Box::into_raw(
        value
            .encode_utf16()
            .chain([0])
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    ) as _)
}
