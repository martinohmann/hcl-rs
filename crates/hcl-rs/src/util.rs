/// Scan `s` for sequences that introduce a template interpolation or directive. Returns `true`
/// once it found one of these start markers, `false` otherwise.
///
/// This function only looks for start markers and does not check if the template is actually
/// valid.
#[inline]
pub fn is_templated(s: &str) -> bool {
    if s.len() < 3 {
        return false;
    }

    let mut skip_next = false;

    // Because calling `s.contains("${")` would also match escaped interpolations (`$${`) a
    // window iterator is used here to detect and ignore these. The same applies to escaped
    // directives.
    for window in s.as_bytes().windows(3) {
        if skip_next {
            skip_next = false;
            continue;
        }

        match window {
            [b'$', b'$', b'{'] | [b'%', b'%', b'{'] => {
                // The next window would incorrectly match the next arm, so it must be
                // skipped.
                skip_next = true;
            }
            [b'$' | b'%', b'{', _] => return true,
            _ => {}
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_templated() {
        assert!(is_templated("${a}"));
        assert!(is_templated("${\"a\"}"));
        assert!(is_templated("%{ if foo }foo%{ else }bar%{ endif }"));
        assert!(is_templated("$${ introduces an ${\"interpolation\"}"));
        assert!(!is_templated(
            "escaped directive %%{ if foo }foo%%{ else }bar%%{ endif }"
        ));
        assert!(!is_templated("escaped interpolation $${a}"));
    }
}
