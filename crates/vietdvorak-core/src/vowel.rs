//! Nhận diện và biến đổi nguyên âm.
//!
//! Bản AHK dựng sẵn `UPPER_VOWELS` và `ALL_VOWEL_CHARS` lúc khởi động, cả hai
//! đều SUY RA từ `VOWELS`. Ở đây phép suy ra chạy tại chỗ dùng: chữ hoa đi qua
//! `to_lowercase` của Unicode rồi tra bảng thường, nên vẫn chỉ có một bảng gốc
//! và không tốn bộ nhớ dựng bảng thứ hai lúc khởi động.

use crate::tables::VOWELS;

/// Ký tự này viết thường ra gì, nếu phép đổi là 1→1.
///
/// Trả `None` khi Unicode đổi một ký tự thành nhiều (ví dụ `İ`). Không ký tự
/// nào trong [`VOWELS`] rơi vào trường hợp đó, nên `None` ở đây luôn nghĩa là
/// "không phải thứ ta quan tâm" chứ không phải mất dữ liệu.
#[must_use]
pub(crate) fn lower1(ch: char) -> Option<char> {
    let mut it = ch.to_lowercase();
    let first = it.next()?;
    if it.next().is_some() {
        None
    } else {
        Some(first)
    }
}

/// Ký tự này viết hoa ra gì, nếu phép đổi là 1→1. Xem [`lower1`].
#[must_use]
pub(crate) fn upper1(ch: char) -> Option<char> {
    let mut it = ch.to_uppercase();
    let first = it.next()?;
    if it.next().is_some() {
        None
    } else {
        Some(first)
    }
}

/// Nguyên âm đã tách thành ba phần độc lập.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct VowelInfo {
    /// Chữ cái gốc, viết thường: `a ă â e ê i o ô ơ u ư y`.
    pub base: char,
    /// 0 = không dấu, 1..=5 theo thứ tự sắc/huyền/hỏi/ngã/nặng.
    pub tone: usize,
    /// Ký tự gốc có viết hoa không.
    pub is_upper: bool,
}

/// Tách một ký tự thành `(chữ cái gốc, dấu thanh, hoa/thường)`.
#[must_use]
pub(crate) fn find_vowel_info(ch: char) -> Option<VowelInfo> {
    if let Some(info) = lookup(ch, false) {
        return Some(info);
    }
    // Không khớp bảng thường: thử coi nó là chữ hoa. Điều kiện `lo != ch` loại
    // bỏ những ký tự vốn không có phân biệt hoa/thường, tránh tra bảng hai lần
    // cho cùng một giá trị.
    let lo = lower1(ch)?;
    if lo == ch {
        return None;
    }
    lookup(lo, true)
}

fn lookup(ch: char, is_upper: bool) -> Option<VowelInfo> {
    for (base, tones) in &VOWELS {
        for (tone, form) in tones.iter().enumerate() {
            if *form == ch {
                return Some(VowelInfo {
                    base: *base,
                    tone,
                    is_upper,
                });
            }
        }
    }
    None
}

/// Đây có phải một nguyên âm (kể cả đã mang dấu phụ hoặc dấu thanh) không?
#[must_use]
pub(crate) fn is_vowel_char(ch: char) -> bool {
    find_vowel_info(ch).is_some()
}

/// Chữ cái gốc (viết thường, đã bỏ dấu thanh) của một nguyên âm.
///
/// `ừ` → `ư`, `Ộ` → `ô`, `c` → `None`.
#[must_use]
pub(crate) fn vowel_base(ch: char) -> Option<char> {
    find_vowel_info(ch).map(|i| i.base)
}

/// Dựng lại một ký tự từ ba phần đã tách.
#[must_use]
pub(crate) fn compose(base: char, tone: usize, is_upper: bool) -> Option<char> {
    let tones = VOWELS.iter().find(|(b, _)| *b == base).map(|(_, t)| t)?;
    let form = *tones.get(tone)?;
    if is_upper { upper1(form) } else { Some(form) }
}

/// Đổi chữ cái gốc của nguyên âm tại `pos` sang `base`, GIỮ NGUYÊN dấu thanh và
/// kiểu hoa/thường đang có: `ọ` + `ơ` → `ợ`, `U` + `ư` → `Ư`.
///
/// Nhờ vậy mọi phép sửa vần đều tone-preserving mà không phải viết lại ở từng
/// chỗ gọi. Trả `false` nếu `pos` không phải nguyên âm hoặc `base` không hợp lệ,
/// và khi đó `chars` không bị đụng tới.
pub(crate) fn set_vowel_base_at(chars: &mut [char], pos: usize, base: char) -> bool {
    let Some(slot) = chars.get_mut(pos) else {
        return false;
    };
    let Some(info) = find_vowel_info(*slot) else {
        return false;
    };
    let Some(new) = compose(base, info.tone, info.is_upper) else {
        return false;
    };
    *slot = new;
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_a_toned_modified_vowel_into_its_three_parts() {
        let i = find_vowel_info('ự').expect("ự là nguyên âm");
        assert_eq!(i.base, 'ư');
        assert_eq!(i.tone, 5);
        assert!(!i.is_upper);
    }

    #[test]
    fn uppercase_is_derived_not_tabulated() {
        // Bảng chỉ có bản thường; bản hoa phải suy ra được cho MỌI ô.
        for (base, tones) in &VOWELS {
            for (tone, form) in tones.iter().enumerate() {
                let up = upper1(*form).expect("nguyên âm phải hoa được 1-1");
                let i = find_vowel_info(up).expect("bản hoa phải tra ngược được");
                assert_eq!(i.base, *base);
                assert_eq!(i.tone, tone);
                assert!(i.is_upper, "{up:?} phải được nhận là chữ hoa");
                assert_eq!(compose(*base, tone, true), Some(up));
            }
        }
    }

    #[test]
    fn consonants_are_not_vowels() {
        for c in ['c', 'đ', 'Đ', 'n', 'g', 'h', ' ', '-', '1'] {
            assert!(!is_vowel_char(c), "{c:?} không phải nguyên âm");
        }
    }

    #[test]
    fn set_base_preserves_tone_and_case() {
        let mut chars = ['ọ'];
        assert!(set_vowel_base_at(&mut chars, 0, 'ơ'));
        assert_eq!(chars.first(), Some(&'ợ'));

        let mut chars = ['U'];
        assert!(set_vowel_base_at(&mut chars, 0, 'ư'));
        assert_eq!(chars.first(), Some(&'Ư'));

        let mut chars = ['à'];
        assert!(set_vowel_base_at(&mut chars, 0, 'ă'));
        assert_eq!(chars.first(), Some(&'ằ'));
    }

    #[test]
    fn set_base_refuses_a_non_vowel_without_touching_it() {
        let mut chars = ['c'];
        assert!(!set_vowel_base_at(&mut chars, 0, 'ơ'));
        assert_eq!(chars.first(), Some(&'c'));
    }
}
