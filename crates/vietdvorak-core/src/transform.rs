//! Một bước Telex: đưa một phím vào buffer.
//!
//! Port của `Transform()` trong `VietDvorak.ahk:581-727`. Thứ tự các nhánh ở
//! đây là NGỮ NGHĨA, không phải phong cách: nhánh `w` muộn phải chạy trước
//! nhánh phím thanh, và cả hai phải chạy trước escape cặp dấu phụ, nếu không
//! `dduwowngf` và `oww` sẽ tranh nhau cùng một phím.

use alloc::vec::Vec;

use crate::nucleus::{
    apply_w_modifier_at, is_normalized_nucleus, match_horn_nucleus, normalize_nucleus,
    trailing_nucleus,
};
use crate::tables::{diacritic_pair, pair_escape, tone_index};
use crate::tone::{apply_tone, current_tone, find_tone_target, reseat_tone, strip_tone};
use crate::vowel::{compose, find_vowel_info, is_vowel_char, lower1, upper1};

/// Máy trạng thái Telex.
///
/// Mang đúng MỘT bit trạng thái ngoài buffer, và bit đó là thật chứ không phải
/// tối ưu: xem [`Telex::transform`].
#[derive(Debug, Clone, Default)]
pub struct Telex {
    /// Phím NGAY TRƯỚC có sinh ra `ư`/`Ư` bằng shortcut `w` đơn của Unikey
    /// không (buffer rỗng hoặc chỉ có phụ âm)?
    ///
    /// Dùng để `ww` thoát thành `w` chứ không phải `uw`: khi `ư` đến từ một
    /// phím `w` đơn thì KHÔNG có `u` ngầm nào để khôi phục, khác với khi nó đến
    /// từ cặp `uw`.
    last_w_was_shortcut: bool,
}

impl Telex {
    /// Máy trạng thái mới.
    #[must_use]
    pub fn new() -> Self {
        Self {
            last_w_was_shortcut: false,
        }
    }

    /// Đưa `key` vào `buffer`, trả buffer mới.
    ///
    /// Nhận `&mut self` vì Telex có trạng thái vượt ra ngoài buffer — cờ
    /// shortcut `w`. Bản AHK để nó ở biến global `g_lastWWasShortcut`, nghĩa là
    /// `Transform()` bên đó cũng không thuần; ở đây nó được kéo vào struct để
    /// không có trạng thái ẩn ở phạm vi module.
    pub fn transform(&mut self, buffer: &[char], key: char) -> Vec<char> {
        // Chụp cờ rồi xoá ngay. Chỉ các nhánh Unikey bên dưới bật lại nó, nên
        // mọi phím khác tự động dập cờ — đúng nghĩa "phím NGAY TRƯỚC".
        let prev_was_shortcut = self.last_w_was_shortcut;
        self.last_w_was_shortcut = false;

        if lower1(key) == Some('w')
            && let Some(handled) = self.late_w(buffer, key, prev_was_shortcut)
        {
            return handled;
        }

        if let Some(out) = tone_key_path(buffer, key) {
            return out;
        }

        if let Some(out) = pair_escape_path(buffer, key) {
            return out;
        }

        normalize_nucleus(&append_with_tone_migration(buffer, key))
    }

    /// Các đường của phím `w` gõ MUỘN. `None` = không xử lý, để phím rơi xuống
    /// các nhánh sau và cuối cùng thành `w` thường.
    fn late_w(&mut self, buffer: &[char], key: char, prev_was_shortcut: bool) -> Option<Vec<char>> {
        // Buffer rỗng: shortcut Unikey, `w` đơn ra thẳng `ư`.
        if buffer.is_empty() {
            self.last_w_was_shortcut = true;
            return Some(alloc::vec![horn_u(key)]);
        }

        let mut chars = buffer.to_vec();
        let last_lo = lower1(*chars.last()?)?;
        let (nuc_start, nuc_shape) = trailing_nucleus(&chars);

        // Dấu móc áp lên CẢ CỤM nguyên âm cuối, không lên từng ký tự: với
        // "uu"/"uou" dấu móc không rơi vào nguyên âm cuối ("ưu" chứ không "uư",
        // "ươu" chứ không "uoư"). Tra bảng thay vì đặc-cách từng ca, nhờ vậy
        // "huouw" -> hươu và "ruouwj" -> rượu chạy đúng như "luuwj".
        if let Some((horn_shape, horned)) = match_horn_nucleus(&nuc_shape) {
            let consumed = horn_shape.chars().count();
            let offset = nuc_start.checked_add(nuc_shape.len().checked_sub(consumed)?)?;
            for (i, c) in horned.chars().enumerate() {
                crate::vowel::set_vowel_base_at(&mut chars, offset.checked_add(i)?, c);
            }
            return Some(settle(&chars));
        }

        // Vần đã ở ĐÚNG dạng có móc mà chuẩn hoá nhắm tới ("đưo" vừa thành
        // "đươ") thì phím "w" tiếp theo không còn gì để làm. Nuốt nó, KHÔNG để
        // rơi xuống escape — nguyên tắc đầy đủ ở `is_normalized_nucleus`.
        if is_normalized_nucleus(&nuc_shape) {
            return Some(buffer.to_vec());
        }

        // Nếu cặp tức thời (aw/ow/uw) sắp thành hình thì để nhánh cặp dấu phụ
        // thường lo. Ngược lại, quét NGƯỢC tìm nguyên âm chưa móc mà móc được.
        if diacritic_pair(last_lo, 'w').is_some() {
            return None;
        }
        for i in (0..chars.len()).rev() {
            if apply_w_modifier_at(&mut chars, i) {
                return Some(settle(&chars));
            }
        }

        // Không có nguyên âm nào móc được. Escape cặp trước: "ư" + w -> "uw".
        // Ngoại lệ: nếu "ư" cuối đến từ shortcut `w` đơn ở phím ngay trước
        // (không phải từ cặp "uw") thì không có "u" ngầm nào để khôi phục, nên
        // escape về đúng chữ "w".   "ww" -> "w",  "chww" -> "chw"
        if let Some((first, second)) = pair_escape(*chars.last()?)
            && lower1(second) == lower1(key)
        {
            let mut out = chars.get(..chars.len().checked_sub(1)?)?.to_vec();
            if !prev_was_shortcut {
                out.push(first);
            }
            out.push(key);
            return Some(out);
        }

        // Shortcut Unikey: buffer chỉ có phụ âm -> nối "ư".
        // "gi" đầu từ là cụm phụ âm (như "qu") nên "i" bị bỏ qua khi tìm nguyên
        // âm: "giw" -> "giư" (giữ, giường...). Bỏ qua nếu có nguyên âm khác,
        // nên "câw" vẫn ra "câw" nguyên văn.
        let scan_start = if chars.len() >= 2
            && chars.first().copied().and_then(lower1) == Some('g')
            && chars.get(1).copied().and_then(lower1) == Some('i')
        {
            2
        } else {
            0
        };
        if !chars.iter().skip(scan_start).any(|c| is_vowel_char(*c)) {
            self.last_w_was_shortcut = true;
            let mut out = buffer.to_vec();
            out.push(horn_u(key));
            return Some(out);
        }

        // Có nguyên âm không móc được — để phím thành chữ "w" thường.
        None
    }
}

/// Đưa buffer vừa đổi hình về trạng thái hợp lệ: dấu thanh đúng target, vần đúng
/// chính tả. Mọi đường không đi qua [`append_with_tone_migration`] kết thúc ở đây.
fn settle(chars: &[char]) -> Vec<char> {
    normalize_nucleus(&reseat_tone(chars))
}

/// `ư` hay `Ư`, theo kiểu chữ của phím vừa gõ.
fn horn_u(key: char) -> char {
    if lower1(key) == Some(key) { 'ư' } else { 'Ư' }
}

/// Nhánh phím thanh: escape khi bấm đôi, còn lại thì gắn dấu.
fn tone_key_path(buffer: &[char], key: char) -> Option<Vec<char>> {
    let tone = tone_index(key)?;

    // Escape: cùng một phím thanh bấm hai lần -> gỡ dấu, nối phím nguyên văn.
    //   "ass" -> "as"
    if let Some(pos) = find_tone_target(buffer)
        && let Some(info) = buffer.get(pos).copied().and_then(find_vowel_info)
        && info.tone == tone
        && let Some(bare) = compose(info.base, 0, info.is_upper)
    {
        let mut stripped = buffer.to_vec();
        *stripped.get_mut(pos)? = bare;
        stripped.push(key);
        return Some(settle(&apply_diacritic_pair(&stripped)));
    }

    apply_tone(buffer, tone)
}

/// Escape cặp dấu phụ: ký tự cuối là kết quả của một cặp, và phím vừa gõ chính
/// là phím thứ hai của cặp đó -> huỷ dấu phụ, nối phím nguyên văn.
///   `oww` -> `ow`,  `aaa` -> `aa`,  `ddd` -> `dd`
fn pair_escape_path(buffer: &[char], key: char) -> Option<Vec<char>> {
    let (first, second) = pair_escape(*buffer.last()?)?;
    if lower1(second) != lower1(key) {
        return None;
    }
    // Thay ký tự cuối mà KHÔNG chạy lại phép nhận diện cặp — nếu chạy lại,
    // "ow" vừa khôi phục sẽ lập tức thành "ơ" trở lại.
    let mut out = buffer.get(..buffer.len().checked_sub(1)?)?.to_vec();
    out.push(first);
    out.push(key);
    Some(settle(&out))
}

/// Nối phím vào buffer, có DI TRÚ dấu thanh.
///
/// Dấu thanh được gỡ ra trước khi xét cặp dấu phụ rồi gắn lại theo quy tắc
/// chuẩn, nên khi phụ âm cuối xuất hiện và làm âm tiết đóng lại, dấu tự chuyển
/// sang nguyên âm đúng: `soajn` -> `soạn` chứ không phải `sọan`.
fn append_with_tone_migration(buffer: &[char], key: char) -> Vec<char> {
    match current_tone(buffer) {
        Some((_, tone)) => {
            let mut stripped = strip_tone(buffer);
            stripped.push(key);
            let paired = apply_diacritic_pair(&stripped);
            apply_tone(&paired, tone).unwrap_or(paired)
        }
        None => {
            let mut out = buffer.to_vec();
            out.push(key);
            apply_diacritic_pair(&out)
        }
    }
}

/// Hai ký tự cuối có tạo thành một cặp dấu phụ không — nếu có, gộp lại.
///
/// Kiểu chữ lấy theo ký tự THỨ NHẤT của cặp: `Aa` và `AA` đều ra `Â`, `aA` ra `â`.
fn apply_diacritic_pair(chars: &[char]) -> Vec<char> {
    let fallback = || chars.to_vec();
    let Some(cut) = chars.len().checked_sub(2) else {
        return fallback();
    };
    let (Some(a), Some(b)) = (
        chars.get(cut).copied(),
        chars.get(cut.saturating_add(1)).copied(),
    ) else {
        return fallback();
    };
    let (Some(la), Some(lb)) = (lower1(a), lower1(b)) else {
        return fallback();
    };
    let Some(result) = diacritic_pair(la, lb) else {
        return fallback();
    };
    let replacement = if a == la {
        result
    } else {
        upper1(result).unwrap_or(result)
    };
    let Some(head) = chars.get(..cut) else {
        return fallback();
    };
    let mut out = head.to_vec();
    out.push(replacement);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vowel::vowel_base;
    use alloc::string::String;

    /// Gõ cả chuỗi vào một buffer rỗng, như `@section transform` của spec.
    fn typed(s: &str) -> String {
        let mut t = Telex::new();
        let mut buf: Vec<char> = Vec::new();
        for k in s.chars() {
            buf = t.transform(&buf, k);
        }
        buf.into_iter().collect()
    }

    // Các test dưới đây KHÔNG nhân bản spec — chúng pin những bất biến mà một
    // vector đơn lẻ không nói ra, và mỗi cái có lý do riêng để tồn tại. Verdict
    // trên toàn bộ vector sống ở check-parity.mjs.
    //
    // Đừng chép SỐ vector vào chú thích. Con số sống đúng một chỗ — output của
    // harness. Ba chỗ trong repo này từng sai vì chép nó.

    #[test]
    fn tone_order_does_not_matter() {
        // Dấu trước móc sau phải ra y hệt móc trước dấu sau. Đây là thứ
        // set_vowel_base_at giữ dấu thanh tồn tại để bảo đảm.
        assert_eq!(typed("luujw"), typed("luuwj"));
        assert_eq!(typed("dduojcw"), typed("dduwojc"));
        assert_eq!(typed("ruoujw"), typed("ruouwj"));
    }

    #[test]
    fn every_spelling_of_the_same_word_converges() {
        let want = typed("dduwowngf");
        for spelling in ["dduwongf", "dduowngf", "dduongwf"] {
            assert_eq!(
                typed(spelling),
                want,
                "{spelling} phải hội tụ về cùng một từ"
            );
        }
    }

    #[test]
    fn engine_inserted_horns_are_not_escapable_but_user_typed_ones_are() {
        // Nguyên tắc ở is_normalized_nucleus, kiểm ở cả hai chiều.
        assert_eq!(typed("dduwowngf"), typed("dduwongf")); // w thừa bị nuốt
        assert_eq!(typed("oww"), "ow"); // móc của người dùng thì huỷ được
        assert_eq!(typed("uww"), "uw");
    }

    #[test]
    fn the_w_shortcut_flag_only_looks_at_the_immediately_previous_key() {
        assert_eq!(typed("ww"), "w"); // ư từ shortcut -> không có u ngầm
        assert_eq!(typed("chww"), "chw");
        assert_eq!(typed("uww"), "uw"); // ư từ cặp uw -> khôi phục u
    }

    #[test]
    fn case_of_the_first_key_decides_the_pair_result() {
        assert_eq!(typed("Aa"), "Â");
        assert_eq!(typed("aA"), "â");
        assert_eq!(typed("Dd"), "Đ");
    }

    /// Đi hết mọi chuỗi phím dài tới `depth` trên `KEYS`, gọi `check` ở mỗi bước.
    /// Chia sẻ tiền tố, nên mỗi nút chỉ tốn đúng một lần `transform`.
    fn walk(telex: &Telex, buf: &[char], depth: usize, check: &mut dyn FnMut(&[char])) {
        const KEYS: &str = "aeouydwsfnA";
        if depth == 0 {
            return;
        }
        for k in KEYS.chars() {
            let mut t = telex.clone();
            let next = t.transform(buf, k);
            check(&next);
            walk(&t, &next, depth.saturating_sub(1), check);
        }
    }

    #[test]
    fn one_tone_on_target_and_no_unwritable_rime_on_every_short_key_sequence() {
        // Đây là bất biến trên MỌI trạng thái, thứ một vector đơn lẻ không phát
        // biểu được. Nó tồn tại vì lỗi hai dấu từng tới được ngay trên đường gõ
        // xuôi trong khi crate này tuyên bố "không thể đạt tới".
        let mut bad: Vec<String> = Vec::new();
        walk(&Telex::new(), &[], 5, &mut |buf| {
            let toned: Vec<usize> = (0..buf.len())
                .filter(|&i| find_vowel_info(buf[i]).is_some_and(|v| v.tone > 0))
                .collect();
            let off_target = toned.len() == 1 && find_tone_target(buf) != Some(toned[0]);
            let bases: Vec<Option<char>> = buf.iter().map(|c| vowel_base(*c)).collect();
            let unwritable = bases.windows(2).any(|p| p == [Some('ư'), Some('o')]);
            if toned.len() > 1 || off_target || unwritable {
                bad.push(buf.iter().collect());
            }
        });
        assert!(
            bad.is_empty(),
            "{} trạng thái sai, ví dụ {:?}",
            bad.len(),
            bad.first()
        );
    }

    #[test]
    fn a_tone_key_with_no_vowel_stays_a_letter() {
        assert_eq!(typed("ngh"), "ngh");
        assert_eq!(typed("s"), "s");
    }
}
