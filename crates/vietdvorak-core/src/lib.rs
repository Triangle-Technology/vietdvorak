//! Lõi Telex + Dvorak — logic thuần, không gọi OS, không dependency.
//!
//! # Trạng thái: M1 — `transform` CÒN LÀ STUB
//!
//! Milestone 1 cố ý dựng báo động trước khi có thứ để bảo vệ: workspace, binary
//! line-protocol và arm thứ ba trong `check-parity.mjs` được xây TRƯỚC khi port
//! FSM. Chạy `node check-parity.mjs` lúc này phải cho ĐỎ, với diff chỉ đích danh
//! Rust và chỉ ở những vector thật sự cần biến đổi. Nếu nó xanh, hoặc đỏ ở chỗ
//! không liên quan, thì chính cái harness sai — và đó là thứ M1 tồn tại để phát
//! hiện, lúc còn rẻ.
//!
//! Xem [`transform`] để biết chính xác cái gì còn thiếu.
//!
//! # Hai layer
//!
//! - **Layer 1** — phím vật lý → ký tự Dvorak. KHÔNG nằm ở đây. Trên cả ba nền
//!   tảng, layer 1 thuộc tầng layout của OS (Windows là ngoại lệ duy nhất tự làm
//!   được, vì `WH_KEYBOARD_LL` nằm thượng nguồn tầng dịch layout). Lõi này nhận
//!   ký tự ĐÃ qua Dvorak.
//! - **Layer 2** — ký tự Dvorak → tiếng Việt có dấu. Chính là file này.
//!
//! # Nguồn sự thật
//!
//! `telex-conformance.txt` là spec, không phải `VietDvorak.ahk` và cũng không
//! phải file này. 266 vector ở đó chạy qua BA engine: AHK, JS trong
//! `practice.html`, và crate này qua `vdconform`.

#![no_std]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

/// Quy ước đặt dấu thanh cho âm tiết MỞ có hai nguyên âm "oa", "oe", "uy".
///
/// - `true` (hiện đại) → hoà, khoẻ, thuỷ — dấu trên nguyên âm chính
/// - `false` (truyền thống) → hòa, khỏe, thủy — dấu trên âm đệm
///
/// Cả hai đều được chấp nhận; bắt buộc là phải NHẤT QUÁN cho cả ba cặp. Đổi ở
/// đúng MỘT chỗ này, không rải if-else trong logic đặt dấu.
///
/// `telex-conformance.txt` được sinh với giá trị `true`; lật hằng này đổi đúng
/// các vector oa/oe/uy trong đó và không đổi gì khác — nên việc lật nó vốn đã là
/// thao tác test được, và đó là lý do đây là một `const` có tên chứ chưa phải
/// một file config.
pub const TONE_PLACEMENT_MODERN: bool = true;

/// Phím này có KẾT THÚC âm tiết không?
///
/// Luật: chỉ `A-Za-z` tiếp tục một âm tiết, mọi thứ khác kết thúc nó và được
/// commit nguyên văn. Chữ HOA phải tiếp tục âm tiết, không được kết thúc —
/// `Vieetj-Nams` ra `Việt-Nam` chứ không phải `Viết-Nam`.
///
/// Luật này được viết ba lần trong dự án (ở đây, `isResetChar` trong
/// `practice.html`, và `VkEndsSyllable` trong `VietDvorak.ahk`).
/// `check-parity.mjs` quét `VkEndsSyllable` trên toàn dải VK và so từng phím với
/// `isResetChar` — nên hai bản kia được canh lẫn nhau ở mức phím, còn bản này
/// được canh ở mức TEXT bởi các vector `@section integration`.
#[must_use]
pub fn ends_syllable(key: char) -> bool {
    !key.is_ascii_alphabetic()
}

/// Một bước Telex: đưa `key` vào `buffer`, trả buffer mới.
///
/// # ĐÂY LÀ STUB
///
/// Bản thật port từ `VietDvorak.ahk:581-727` (`Transform()`) và phải làm:
///
/// - bảng nguyên âm × 6 dấu thanh, và bản HOA **suy ra** từ bảng thường chứ
///   không chép tay — spec có vector pin đúng phép suy ra đó;
/// - đặt dấu thông minh: `tieengs`→`tiếng`, `ngoaif`→`ngoài`, `soajn`→`soạn`,
///   `Quynhf`→`Quỳnh`, cụm `qu`/`gi` bỏ qua, di trú dấu khi phụ âm cuối xuất hiện;
/// - cặp dấu phụ `aa ee oo aw ow uw dd`, và escape khi bấm đôi (`aaa`→`aa`);
/// - escape phím thanh bấm đôi (`ass`→`as`);
/// - shortcut Unikey `w` trên buffer rỗng/phụ âm → `ư`, cùng cờ nhớ để `ww`→`w`;
/// - chuẩn hoá nhân âm tiết (`uow`→`ươ`, `uu`→`ưu`).
///
/// Stub hiện tại chỉ append. Hệ quả có chủ đích: vector nào không cần biến đổi
/// thì XANH, vector nào cần thì ĐỎ. Đó là hình dạng đúng của báo động ở M1 —
/// plumbing đã đúng, chỉ thiếu FSM.
#[must_use]
pub fn transform(buffer: &[char], key: char) -> Vec<char> {
    let mut out = Vec::with_capacity(buffer.len().saturating_add(1));
    out.extend_from_slice(buffer);
    out.push(key);
    out
}

/// Kết quả của một lần gõ phím, ở dạng shell nào cũng hạ xuống được.
///
/// Đây là seam đa nền tảng. Hai nền tảng tiêu thụ nó theo hai cách khác hẳn
/// nhau, nên `committed` là field RIÊNG BIỆT khỏi `composing`, không phải một
/// chuỗi duy nhất mà shell tự đoán:
///
/// - **IME (IBus, và sau này IMKit)** — `composing` vào `UpdatePreeditText`,
///   `committed` vào `CommitText`.
/// - **Injection (Windows hook)** — diff `composing` rồi phát backspace + text,
///   theo đúng thuật toán `SyncToScreen()` ở `VietDvorak.ahk:729`.
///
/// # Ba mệnh đề mà shell injection PHẢI theo
///
/// 1. diff **chỉ** trên `composing`, so với `last_composing` của chính shell;
/// 2. append `committed` **nguyên văn**, không diff;
/// 3. khi `committed` khác rỗng → reset `last_composing` về rỗng **KHÔNG phát
///    backspace**.
///
/// Mệnh đề (3) là thứ chắc chắn sẽ làm sai. Ví dụ: `composing:"mùa"` rồi gõ
/// Space → `composing:"" committed:"mùa "`. Một diff ngây thơ thấy `composing`
/// co từ 3 về 0 và phát 3 backspace — **phá text vừa commit**.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct View {
    /// Âm tiết đang gõ, chưa chốt. Rỗng nghĩa là không có preedit.
    pub composing: String,
    /// Text đã chốt ở đúng lần gõ phím này. Thường rỗng.
    pub committed: String,
    /// Engine có nuốt phím này không. `false` = để app nhận phím thật.
    pub consumed: bool,
}

/// Trạng thái gõ của một ô nhập liệu.
///
/// Cố ý CHƯA có `backspace()`. Bản AHK cắt buffer bằng `SubStr(g_buffer, 1, -1)`
/// mà không chuẩn hoá lại, phá bất biến "tối đa một dấu, đúng tại target" và tạo
/// ra được `mùà` (hai dấu). `telex-conformance.txt` có 266 vector và **zero
/// vector backspace**, nên AHK không thể làm oracle cho hành vi này — không có
/// đáp án đúng nào để so.
///
/// Backspace sẽ vào ở pha sau, bằng **replay** chứ không phải truncate: giữ lịch
/// sử phím thô, pop, chạy lại `transform` từ rỗng. Đó là fix nguyên nhân gốc, và
/// nó chỉ khả thi dưới preedit — nơi IME sở hữu vùng đang gõ nên buffer không
/// còn phải mô hình hoá màn hình.
#[derive(Debug, Clone, Default)]
pub struct Session {
    buf: Vec<char>,
}

impl Session {
    /// Phiên mới, buffer rỗng.
    #[must_use]
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }

    /// Âm tiết đang gõ.
    #[must_use]
    pub fn composing(&self) -> String {
        self.buf.iter().collect()
    }

    /// Bỏ trạng thái đang gõ mà KHÔNG chốt gì.
    ///
    /// Shell gọi cái này khi mất focus, đổi cửa sổ, hoặc người dùng click chỗ
    /// khác. Bản AHK phải móc ~222 virtual key và 5 nút chuột để đoán ra những
    /// thời điểm đó; IME được OS báo thẳng, nên cả khối ấy thu về một lời gọi.
    pub fn reset(&mut self) {
        self.buf.clear();
    }

    /// Đưa một ký tự qua đường apply đầy đủ: hỏi luật ngắt âm tiết trước, rồi
    /// mới gọi [`transform`].
    ///
    /// Phím ngắt được chốt NGUYÊN VĂN cùng buffer — khớp bản AHK, nơi
    /// `xin chaof` ra `xin chào` với dấu cách nằm trong text đã chốt. Shell IBus
    /// sau này có thể chọn chốt buffer rồi *forward* phím ngắt thay vì tự chốt
    /// nó; [`View`] mang đủ thông tin cho cả hai, nên đó là quyết định của shell
    /// chứ không phải của lõi.
    pub fn feed(&mut self, key: char) -> View {
        if ends_syllable(key) {
            let mut committed: String = self.buf.iter().collect();
            committed.push(key);
            self.buf.clear();
            return View {
                composing: String::new(),
                committed,
                consumed: true,
            };
        }
        self.buf = transform(&self.buf, key);
        View {
            composing: self.composing(),
            committed: String::new(),
            consumed: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    // Luật ngắt âm tiết là thật ở M1 (nó là một dòng và không mơ hồ), nên nó
    // được test thật. `transform` là stub nên KHÔNG có test hành vi Telex ở đây:
    // spec sống trong telex-conformance.txt và chạy qua vdconform, không nhân
    // bản thành một verdict thứ hai trong Rust.

    #[test]
    fn only_ascii_letters_continue_a_syllable() {
        for c in ['a', 'z', 'A', 'Z', 'q', 'M'] {
            assert!(!ends_syllable(c), "{c:?} phải TIẾP TỤC âm tiết");
        }
        for c in [' ', '-', '.', '1', '\t', '\n', 'đ', 'ế', '\u{0}'] {
            assert!(ends_syllable(c), "{c:?} phải KẾT THÚC âm tiết");
        }
    }

    #[test]
    fn uppercase_continues_it_produced_a_real_defect_once() {
        // "Vieetj-Nams" từng ra "Viết-Nam" vì chữ hoa bị coi là ngắt âm tiết.
        assert!(!ends_syllable('V'));
        assert!(!ends_syllable('N'));
        assert!(ends_syllable('-'));
    }

    #[test]
    fn break_char_commits_buffer_and_itself_then_clears() {
        let mut s = Session::new();
        for c in "abc".chars() {
            s.feed(c);
        }
        let v = s.feed(' ');
        assert_eq!(v.committed, "abc ".to_string());
        assert_eq!(v.composing, "");
        assert_eq!(s.composing(), "");
    }

    #[test]
    fn reset_drops_the_syllable_without_committing_it() {
        let mut s = Session::new();
        for c in "abc".chars() {
            s.feed(c);
        }
        s.reset();
        assert_eq!(s.composing(), "");
    }

    #[test]
    fn committed_is_separate_from_composing_clause_three() {
        // Mệnh đề (3) của View: khi committed khác rỗng, composing về rỗng và
        // shell injection KHÔNG được diff cái co lại đó thành backspace.
        let mut s = Session::new();
        s.feed('m');
        let v = s.feed(' ');
        assert!(!v.committed.is_empty());
        assert!(v.composing.is_empty());
    }
}
