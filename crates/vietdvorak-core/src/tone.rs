//! Đặt dấu thanh: chọn nguyên âm nào nhận dấu, và gắn/gỡ dấu.

use alloc::vec::Vec;

use crate::TONE_PLACEMENT_MODERN;
use crate::tables::GLIDE_PAIRS;
use crate::vowel::{compose, find_vowel_info, is_vowel_char, lower1, vowel_base};

/// Nguyên âm đã mang dấu móc/trăng — chúng được ưu tiên nhận dấu thanh.
const MODIFIED: [char; 6] = ['â', 'ă', 'ê', 'ô', 'ơ', 'ư'];

/// Cặp âm đệm + nguyên âm chính mà hai quy ước đặt dấu khác nhau?
#[must_use]
fn is_glide_pair(prev: char, last: char) -> bool {
    GLIDE_PAIRS.iter().any(|(p, l)| *p == prev && *l == last)
}

/// Nguyên âm nào trong `chars` nhận dấu thanh? Trả về chỉ số, `None` nếu không
/// có nguyên âm nào.
///
/// Quy tắc, theo đúng thứ tự:
/// 1. Bỏ qua cụm phụ âm đầu `qu`/`gi` khi sau nó còn nguyên âm.
/// 2. Có nguyên âm đã mang dấu móc/trăng (â ă ê ô ơ ư) → lấy cái CUỐI CÙNG.
/// 3. Âm tiết ĐÓNG (kết thúc bằng phụ âm) → nguyên âm cuối.
/// 4. Âm tiết MỞ → cặp lướt oa/oe/uy theo [`TONE_PLACEMENT_MODERN`]; còn lại
///    là nguyên âm áp chót.
#[must_use]
pub(crate) fn find_tone_target(chars: &[char]) -> Option<usize> {
    let start = cluster_skip(chars);

    // `get(start..)` chứ không phải `[start..]`: một cụm dài hơn cả buffer là
    // chuyện không xảy ra, nhưng cách viết này khiến nó không thể panic dù có.
    let mut vowels = collect_vowels(chars, start);
    if vowels.is_empty() && start > 0 {
        // Cụm phụ âm đầu đã nuốt mất nguyên âm duy nhất — quét lại từ đầu.
        vowels = collect_vowels(chars, 0);
    }

    match vowels.len() {
        0 => return None,
        1 => return vowels.first().map(|(pos, _)| *pos),
        _ => {}
    }

    // Ưu tiên nguyên âm CÓ DẤU MÓC cuối cùng — xử lý "ươ" → ơ.
    if let Some((pos, _)) = vowels
        .iter()
        .rev()
        .find(|(_, base)| MODIFIED.contains(base))
    {
        return Some(*pos);
    }

    let closed = chars.last().is_some_and(|c| !is_vowel_char(*c));
    let last = vowels.last()?;
    if closed {
        return Some(last.0);
    }

    let prev = vowels.get(vowels.len().checked_sub(2)?)?;
    if TONE_PLACEMENT_MODERN && is_glide_pair(prev.1, last.1) {
        return Some(last.0);
    }
    Some(prev.0)
}

/// Chỉ số bắt đầu quét nguyên âm, sau khi bỏ qua cụm phụ âm đầu `qu`/`gi`.
fn cluster_skip(chars: &[char]) -> usize {
    if chars.len() < 3 {
        return 0;
    }
    let (Some(c0), Some(c1), Some(c2)) = (chars.first(), chars.get(1), chars.get(2)) else {
        return 0;
    };
    if !is_vowel_char(*c2) {
        return 0;
    }
    let lo0 = lower1(*c0);
    let lo1 = lower1(*c1);
    if (lo0 == Some('q') && lo1 == Some('u')) || (lo0 == Some('g') && lo1 == Some('i')) {
        return 2;
    }
    0
}

/// `(chỉ số, chữ cái gốc)` của mọi nguyên âm từ `start` tới hết.
fn collect_vowels(chars: &[char], start: usize) -> Vec<(usize, char)> {
    chars
        .iter()
        .enumerate()
        .skip(start)
        .filter_map(|(i, c)| vowel_base(*c).map(|b| (i, b)))
        .collect()
}

/// Gắn dấu thanh `tone` vào đúng nguyên âm của `chars`.
///
/// Trả `None` khi không có nguyên âm nào để gắn — chỗ gọi dựa vào đó để biết
/// phím thanh phải rơi xuống thành ký tự thường.
///
/// GHI CHÚ về bất biến: hàm này ghi dấu tại target mà KHÔNG gỡ dấu cũ ở nơi
/// khác, y như bản AHK (`ApplyToneToBuffer`, chú thích "Always apply"). Nó an
/// toàn CHỈ KHI buffer giữ được bất biến "tối đa một dấu, và dấu đó nằm đúng
/// tại `find_tone_target`". Đường nối phím giữ được bất biến ấy vì nó
/// [`strip_tone`] trước rồi mới gắn lại; các đường khác giữ nó bằng
/// [`reseat_tone`]. Bản AHK phá bất biến này ở đường backspace — xem chú thích của
/// [`crate::Session`].
#[must_use]
pub(crate) fn apply_tone(chars: &[char], tone: usize) -> Option<Vec<char>> {
    let pos = find_tone_target(chars)?;
    let info = find_vowel_info(*chars.get(pos)?)?;
    let new = compose(info.base, tone, info.is_upper)?;
    let mut out = chars.to_vec();
    *out.get_mut(pos)? = new;
    Some(out)
}

/// Đặt lại dấu thanh hiện có vào đúng target của buffer hiện tại.
///
/// Mọi phép đổi hình âm tiết mà không đi qua đường nối phím PHẢI gọi hàm này:
/// nó là thứ giữ bất biến mà [`apply_tone`] dựa vào. `tuyf` + `w` đổi target từ
/// `y` sang `ư`; để dấu lại trên `y` thì phím thanh kế tiếp ghi dấu THỨ HAI.
#[must_use]
pub(crate) fn reseat_tone(chars: &[char]) -> Vec<char> {
    let Some((_, tone)) = current_tone(chars) else {
        return chars.to_vec();
    };
    let stripped = strip_tone(chars);
    apply_tone(&stripped, tone).unwrap_or(stripped)
}

/// Dấu thanh hiện có trong buffer: `(chỉ số, chỉ số thanh)`.
#[must_use]
pub(crate) fn current_tone(chars: &[char]) -> Option<(usize, usize)> {
    chars.iter().enumerate().find_map(|(i, c)| {
        find_vowel_info(*c)
            .filter(|v| v.tone > 0)
            .map(|v| (i, v.tone))
    })
}

/// Gỡ dấu thanh khỏi mọi nguyên âm, GIỮ NGUYÊN dấu móc/trăng.
#[must_use]
pub(crate) fn strip_tone(chars: &[char]) -> Vec<char> {
    chars
        .iter()
        .map(|c| {
            find_vowel_info(*c)
                .filter(|v| v.tone > 0)
                .and_then(|v| compose(v.base, 0, v.is_upper))
                .unwrap_or(*c)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::String;

    /// LƯU Ý: tham số là BUFFER đã biến đổi, không phải chuỗi phím thô. Gõ
    /// `tieengs` cho buffer `tiêng` rồi mới hỏi dấu rơi vào đâu — truyền
    /// `tieeng` vào đây là hỏi một câu khác hẳn (khi đó chưa có nguyên âm nào
    /// mang dấu móc, nên luật "ưu tiên nguyên âm đã móc" không kích hoạt).
    fn target(buffer: &str) -> Option<char> {
        let chars: Vec<char> = buffer.chars().collect();
        find_tone_target(&chars).and_then(|p| chars.get(p).copied())
    }

    #[test]
    fn modified_vowel_wins_over_position() {
        assert_eq!(target("tiêng"), Some('ê'));
        // Hai nguyên âm đều đã móc -> lấy cái CUỐI CÙNG.
        assert_eq!(target("đương"), Some('ơ'));
    }

    #[test]
    fn closed_syllable_takes_the_last_vowel() {
        assert_eq!(target("soan"), Some('a'));
        assert_eq!(target("toan"), Some('a'));
        assert_eq!(target("hoc"), Some('o'));
    }

    #[test]
    fn open_syllable_takes_the_second_to_last() {
        // Không phải cặp lướt -> áp chót, kể cả khi còn ba nguyên âm.
        assert_eq!(target("mua"), Some('u'));
        assert_eq!(target("ngoai"), Some('a'));
    }

    #[test]
    fn qu_and_gi_are_initial_clusters_not_vowels() {
        // Cụm qu bị bỏ qua nên dấu rơi vào y, không vào u.
        assert_eq!(target("Quynh"), Some('y'));
        assert_eq!(target("gia"), Some('a'));
    }

    #[test]
    fn a_two_char_qu_or_gi_is_too_short_to_be_a_cluster() {
        // Cụm chỉ được bỏ qua khi SAU nó còn nguyên âm, nên "qu"/"gi" trần vẫn
        // phải có target — nếu không, phím thanh sẽ rơi xuống thành chữ thường.
        //
        // Hệ quả: nhánh fallback "cụm đã nuốt mất nguyên âm duy nhất" trong
        // find_tone_target là KHÔNG THỂ VỚI TỚI — điều kiện bỏ qua cụm đã đòi
        // chars[2] là nguyên âm, mà chính chars[2] là chỗ bắt đầu quét. Nó được
        // port nguyên trạng từ bản AHK để hai bên còn so được từng dòng, chứ
        // không phải vì có đường chạy tới.
        assert_eq!(target("qu"), Some('u'));
        assert_eq!(target("gi"), Some('i'));
    }

    #[test]
    fn glide_pairs_follow_the_modern_convention() {
        // Ba dòng dưới đây LÀ định nghĩa vận hành của TONE_PLACEMENT_MODERN =
        // true: hoà/khoẻ/thuỷ, dấu trên nguyên âm chính. Lật hằng đó thì cả ba
        // đỏ cùng lúc và báo đúng nguyên âm nào nhận dấu — rõ hơn một assert
        // trên chính hằng số.
        //
        // Cả BA cặp phải đi cùng nhau. Trước đây chỉ "uy" được ưu tiên nên bản
        // AHK tự mâu thuẫn: thuỷ theo kiểu mới nhưng hòa/khỏe theo kiểu cũ.
        assert_eq!(target("hoa"), Some('a'));
        assert_eq!(target("khoe"), Some('e'));
        assert_eq!(target("thuy"), Some('y'));
    }

    #[test]
    fn no_vowel_means_no_target() {
        assert_eq!(target("ngh"), None);
        assert_eq!(target(""), None);
    }

    #[test]
    fn strip_keeps_the_horn_and_drops_only_the_tone() {
        let s: String = strip_tone(&"được".chars().collect::<Vec<_>>())
            .into_iter()
            .collect();
        assert_eq!(s, "đươc");
    }
}
