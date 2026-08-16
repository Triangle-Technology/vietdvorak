//! Vần: chuẩn hoá vị trí dấu móc, và áp dấu móc theo CỤM.

use alloc::vec::Vec;

use crate::tables::{HORN_NUCLEUS, NUCLEUS_NORMALIZE, w_modifier};
use crate::tone::{apply_tone, current_tone, strip_tone};
use crate::vowel::{find_vowel_info, set_vowel_base_at, vowel_base};

/// Cụm nguyên âm ở CUỐI buffer: `(chỉ số bắt đầu, chuỗi chữ cái gốc)`.
///
/// Chuỗi rỗng nghĩa là ký tự cuối không phải nguyên âm.
#[must_use]
pub(crate) fn trailing_nucleus(chars: &[char]) -> (usize, Vec<char>) {
    let mut start = chars.len();
    while start > 0 {
        let Some(prev) = start.checked_sub(1) else {
            break;
        };
        match chars.get(prev).copied().and_then(vowel_base) {
            Some(_) => start = prev,
            None => break,
        }
    }
    let shape = chars
        .iter()
        .skip(start)
        .filter_map(|c| vowel_base(*c))
        .collect();
    (start, shape)
}

/// Hậu tố DÀI NHẤT của cụm nguyên âm có trong [`HORN_NUCLEUS`].
#[must_use]
pub(crate) fn match_horn_nucleus(shape: &[char]) -> Option<(&'static str, &'static str)> {
    for len in (1..=shape.len()).rev() {
        let from = shape.len().checked_sub(len)?;
        let Some(tail) = shape.get(from..) else {
            continue;
        };
        for (from_shape, horned) in &HORN_NUCLEUS {
            if from_shape.chars().eq(tail.iter().copied()) {
                return Some((from_shape, horned));
            }
        }
    }
    None
}

/// Cụm nguyên âm này có phải chính là DẠNG ĐÍCH của một luật chuẩn hoá không —
/// tức dấu móc đã nằm đúng chỗ, không còn gì để sửa?
///
/// # Nguyên tắc hàm này phục vụ
///
/// Escape (gõ lại phím trigger) chỉ được phép HUỶ DẤU MÓC DO CHÍNH NGƯỜI DÙNG
/// GÕ RA, không bao giờ được huỷ dấu móc do ENGINE tự chèn khi chuẩn hoá vần.
///
/// Người dùng gõ `dduwo` thì engine đã tự sửa `ưo` thành `ươ` (buffer `đươ`):
/// dấu móc trên `ơ` là của engine. Phím `w` kế tiếp vì thế là THỪA — yêu cầu
/// "thêm móc" đã được đáp ứng — chứ không phải lệnh huỷ, nên phải nuốt nó
/// (`dduwowngf` → `đường`). Ngược lại trong `oww`/`uww`, cụm cuối `ơ`/`ư` do
/// CHÍNH người dùng gõ ra bằng cặp ow/uw, nên escape vẫn phải chạy như thường.
///
/// Đọc thẳng từ bảng chứ không viết lại `ươ` ở đây — thêm vần mới vào
/// [`NUCLEUS_NORMALIZE`] là chỗ này tự đúng theo.
#[must_use]
pub(crate) fn is_normalized_nucleus(shape: &[char]) -> bool {
    NUCLEUS_NORMALIZE
        .iter()
        .any(|r| r.fixed.chars().eq(shape.iter().copied()))
}

/// Áp dấu móc/trăng của phím `w` lên nguyên âm tại `pos`, giữ nguyên dấu thanh.
///
/// `ọ` → `ợ`, `ụ` → `ự`, `à` → `ằ`. Việc giữ dấu thanh là thứ làm cho thứ tự
/// "dấu trước, móc sau" (`đuojcw` → `được`) cho kết quả y hệt chiều ngược lại.
pub(crate) fn apply_w_modifier_at(chars: &mut [char], pos: usize) -> bool {
    let Some(info) = chars.get(pos).copied().and_then(find_vowel_info) else {
        return false;
    };
    let Some(horned) = w_modifier(info.base) else {
        return false;
    };
    set_vowel_base_at(chars, pos, horned)
}

/// Chuẩn hoá vần theo [`NUCLEUS_NORMALIZE`]: dấu móc phải nằm đúng chỗ chính tả
/// quy định, và dấu thanh phải đi theo.
///
/// Việc "có phải chờ ký tự phía sau không" đọc từ cờ `lazy` của TỪNG luật — hàm
/// này không biết, và không cần biết, vần nào là vần nào.
#[must_use]
pub(crate) fn normalize_nucleus(chars: &[char]) -> Vec<char> {
    if chars.len() < 2 {
        return chars.to_vec();
    }
    let mut out = chars.to_vec();
    let mut changed = false;

    for i in 0..out.len().saturating_sub(1) {
        let Some(next) = i.checked_add(1) else {
            continue;
        };
        let (Some(b0), Some(b1)) = (
            out.get(i).copied().and_then(vowel_base),
            out.get(next).copied().and_then(vowel_base),
        ) else {
            continue;
        };
        let Some(rule) = NUCLEUS_NORMALIZE
            .iter()
            .find(|r| r.shape.chars().eq([b0, b1]))
        else {
            continue;
        };
        // Vần đang ở cuối buffer và luật phải chờ đuôi: nó còn có thể là một
        // dạng mở hợp lệ, đừng phá thứ người dùng vừa gõ đúng.
        if rule.lazy && next == out.len().saturating_sub(1) {
            continue;
        }
        let mut fixed = rule.fixed.chars();
        let (Some(f0), Some(f1)) = (fixed.next(), fixed.next()) else {
            continue;
        };
        set_vowel_base_at(&mut out, i, f0);
        set_vowel_base_at(&mut out, next, f1);
        changed = true;
    }

    if !changed {
        return out;
    }
    // Vần đổi thì chỗ đặt dấu thanh cũng đổi theo: "ngừo" + i phải ra "người"
    // chứ không phải "ngừơi". Đặt lại dấu bằng đúng quy tắc chuẩn.
    if let Some((_, tone)) = current_tone(&out)
        && let Some(retoned) = apply_tone(&strip_tone(&out), tone)
    {
        return retoned;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::String;

    fn v(s: &str) -> Vec<char> {
        s.chars().collect()
    }
    fn j(c: &[char]) -> String {
        c.iter().collect()
    }

    #[test]
    fn trailing_nucleus_stops_at_the_first_consonant() {
        let (start, shape) = trailing_nucleus(&v("đươ"));
        assert_eq!(start, 1);
        assert_eq!(j(&shape), "ươ");

        let (start, shape) = trailing_nucleus(&v("đươc"));
        assert_eq!(start, 4);
        assert_eq!(j(&shape), "");
    }

    #[test]
    fn horn_nucleus_prefers_the_longest_suffix() {
        // "uou" phải thắng "uu" khi cả hai đều là hậu tố hợp lệ về hình dạng.
        assert_eq!(match_horn_nucleus(&v("uou")), Some(("uou", "ươu")));
        assert_eq!(match_horn_nucleus(&v("uu")), Some(("uu", "ưu")));
        assert_eq!(match_horn_nucleus(&v("oa")), None);
    }

    #[test]
    fn eager_rule_fixes_even_at_the_end_of_the_buffer() {
        // "ưo" không hợp lệ ở mọi độ dài -> sửa ngay, không chờ.
        assert_eq!(j(&normalize_nucleus(&v("đưo"))), "đươ");
    }

    #[test]
    fn lazy_rule_waits_because_the_open_form_is_valid() {
        // "uơ" là vần mở hợp lệ (thuơ/huơ/quơ) -> không được sửa khi ở cuối.
        assert_eq!(j(&normalize_nucleus(&v("thuơ"))), "thuơ");
        // Có đuôi rồi thì mới chắc, và mới sửa.
        assert_eq!(j(&normalize_nucleus(&v("thuơng"))), "thương");
    }

    #[test]
    fn normalisation_moves_the_tone_with_the_nucleus() {
        // "ngừo" + i phải ra "người", không phải "ngừơi".
        assert_eq!(j(&normalize_nucleus(&v("ngừoi"))), "người");
    }

    #[test]
    fn normalized_shape_is_read_from_the_table() {
        assert!(is_normalized_nucleus(&v("ươ")));
        assert!(!is_normalized_nucleus(&v("ưo")));
        assert!(!is_normalized_nucleus(&v("ơ")));
    }
}
