//! Lõi Telex + Dvorak — logic thuần, không gọi OS, không dependency.
//!
//! # Hai layer
//!
//! - **Layer 1** — phím vật lý → ký tự Dvorak. KHÔNG nằm ở đây. Trên cả ba nền
//!   tảng, layer 1 thuộc tầng layout của OS (Windows là ngoại lệ duy nhất tự
//!   làm được, vì `WH_KEYBOARD_LL` nằm thượng nguồn tầng dịch layout). Lõi này
//!   nhận ký tự ĐÃ qua Dvorak.
//! - **Layer 2** — ký tự Dvorak → tiếng Việt có dấu. Chính là crate này.
//!
//! # Nguồn sự thật
//!
//! `telex-conformance.txt` là spec, không phải `VietDvorak.ahk` và cũng không
//! phải crate này. Vector ở đó chạy qua ba engine — `Transform()` của AHK, bản
//! mirror JS trong `practice.html`, và crate này qua `vdconform` — nhưng KHÔNG
//! phải section nào cũng áp dụng cho cả ba. `@section backspace` chỉ dành cho
//! crate này, vì AHK truncate còn crate này replay, và đó là khác biệt kiến
//! trúc chứ không phải một bên sai; lý do đầy đủ nằm ở header của file spec và
//! ở chú thích của [`Session`].
//!
//! Test trong crate chỉ pin những bất biến mà một vector đơn lẻ không phát biểu
//! được — cờ `consumed`, hay "một âm tiết không bao giờ mang hai dấu". Chúng
//! không nhân bản spec, vì hai verdict cho một spec sẽ trôi khỏi nhau.

#![no_std]

extern crate alloc;

mod nucleus;
mod tables;
mod tone;
mod transform;
mod vowel;

pub use transform::Telex;

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
/// các vector oa/oe/uy trong đó và không đổi gì khác — nên việc lật nó vốn đã
/// là thao tác test được, và đó là lý do đây là một `const` có tên chứ chưa
/// phải một file config.
pub const TONE_PLACEMENT_MODERN: bool = true;

/// Phím này có KẾT THÚC âm tiết không?
///
/// Luật: chỉ `A-Za-z` tiếp tục một âm tiết, mọi thứ khác kết thúc nó và được
/// commit nguyên văn. Chữ HOA phải tiếp tục âm tiết, không được kết thúc —
/// `Vieetj-Nams` ra `Việt-Nam` chứ không phải `Viết-Nam`.
///
/// Luật này được viết ba lần trong dự án (ở đây, `isResetChar` trong
/// `practice.html`, và `VkEndsSyllable` trong `VietDvorak.ahk`).
/// `check-parity.mjs` quét `VkEndsSyllable` trên toàn dải VK và so từng phím
/// với `isResetChar` — nên hai bản kia được canh lẫn nhau ở mức PHÍM, còn bản
/// này được canh ở mức TEXT bởi các vector `@section integration`.
#[must_use]
pub fn ends_syllable(key: char) -> bool {
    !key.is_ascii_alphabetic()
}

/// Kết quả của một lần gõ phím, ở dạng shell nào cũng hạ xuống được.
///
/// Đây là seam đa nền tảng. Hai nền tảng tiêu thụ nó theo hai cách khác hẳn
/// nhau, nên `committed` là field RIÊNG BIỆT khỏi `composing`, không phải một
/// chuỗi duy nhất mà shell tự đoán:
///
/// - **IME (IBus, và sau này InputMethodKit)** — `composing` vào
///   `UpdatePreeditText`, `committed` vào `CommitText`.
/// - **Injection (Windows hook)** — diff `composing` rồi phát backspace + text,
///   theo đúng thuật toán `VietDvorak.ahk :: SyncToScreen()`.
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
/// # Backspace là REPLAY, không phải truncate
///
/// Bản AHK cắt buffer bằng `SubStr(g_buffer, 1, -1)` mà không chuẩn hoá lại.
/// Nó phải làm thế: trong mô hình injection, `g_buffer` CHÍNH LÀ mô hình của
/// text mà app sở hữu, nên không có gì khác để cắt. Hệ quả là bất biến "tối đa
/// một dấu, đúng tại target" bị phá, và `muafn` ⌫ `f` ra `mùà` — hai dấu, một
/// chuỗi tiếng Việt không biểu đạt được.
///
/// Dưới preedit, IME sở hữu vùng đang gõ tuyệt đối, nên [`Session`] không còn
/// phải mô hình hoá màn hình và giữ được thứ đúng đắn hơn: **lịch sử phím thô**.
/// Backspace pop lịch sử rồi chạy lại [`Telex::transform`] từ rỗng. Không có
/// trạng thái nào để hỏng, nên hai dấu là điều KHÔNG THỂ ĐẠT TỚI chứ không phải
/// điều được canh chừng.
///
/// Đây là hành vi MỚI, không phải port: AHK không thể làm oracle cho nó, vì
/// truncate và replay khác nhau ở gần như mọi input (`tieengs` ⌫ cho `tiến` so
/// với `tiêng`). TRƯỚC `@section backspace`, `telex-conformance.txt` không có
/// một vector backspace nào, nên không có luật cũ nào bị vi phạm; các vector
/// trong section đó là spec MỚI, và chỉ cho crate này.
///
/// # Vì sao pop tới khi TEXT ĐỔI, không phải pop đúng một phím
///
/// Luật này lấy từ `undoKey()` trong `practice.html`, nơi nó đã được lập luận
/// đầy đủ. Phím bị NUỐT vẫn phải được ghi log: chữ `w` thừa trong `dduwowngf`
/// không sinh ra text nào, nhưng nó là một keystroke, và bỏ nó khỏi log sẽ làm
/// replay sau đó lệch khỏi thứ người dùng thực sự đã gõ. Nhưng nếu Backspace
/// chỉ pop đúng một phím thì lần bấm ấy sẽ **tỏ ra là không làm gì cả**. Nên:
/// pop cho tới khi text đổi.
///
/// # Khác biệt với `practice.html`, có chủ ý
///
/// Log của trainer trải trên CẢ text đã chốt, nên undo bên đó có thể pop một
/// dấu cách và kéo text đã commit trở lại buffer. Engine này không thể và không
/// nên: text đã chốt thuộc về app, không thuộc về IME. Khi log rỗng,
/// [`Session::backspace`] trả `consumed: false` và để app tự xoá text của nó.
#[derive(Debug, Clone, Default)]
pub struct Session {
    buf: Vec<char>,
    /// Chuỗi phím đã TẠO RA `buf`, chỉ của âm tiết đang gõ.
    ///
    /// `buf` được suy ra từ đây, không bao giờ ngược lại — đó chính là thứ cho
    /// phép Backspace undo một KEYSTROKE thay vì một ký tự.
    keys: Vec<char>,
    telex: Telex,
}

impl Session {
    /// Phiên mới, buffer rỗng.
    #[must_use]
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            keys: Vec::new(),
            telex: Telex::new(),
        }
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
        self.keys.clear();
        self.telex = Telex::new();
    }

    /// Undo keystroke cuối cùng ĐÃ SINH RA THỨ GÌ ĐÓ, và không undo gì thêm.
    ///
    /// Trả `consumed: false` khi engine không còn gì để undo — lúc đó Backspace
    /// là của app, không phải của IME. Lý do đầy đủ của cả hai luật nằm ở
    /// chú thích của [`Session`].
    pub fn backspace(&mut self) -> View {
        if self.keys.is_empty() {
            return View {
                composing: String::new(),
                committed: String::new(),
                consumed: false,
            };
        }
        let before = self.composing();
        while self.keys.pop().is_some() {
            self.replay();
            if self.composing() != before {
                break;
            }
        }
        View {
            composing: self.composing(),
            committed: String::new(),
            consumed: true,
        }
    }

    /// Dựng lại `buf` từ `keys`, bắt đầu từ rỗng với máy trạng thái mới.
    ///
    /// Đúng theo cấu trúc: [`Telex::transform`] là hàm thuần của
    /// `(buffer, key)` cộng đúng một bit trạng thái, và bit đó KHÔNG QUAN SÁT
    /// ĐƯỢC khi buffer rỗng (nó chỉ được đọc trên đường escape cặp, mà đường ấy
    /// đòi buffer khác rỗng). Nên replay từ rỗng với bit đã xoá tái hiện đúng
    /// từng trạng thái sống, keystroke cho keystroke.
    fn replay(&mut self) {
        let keys = core::mem::take(&mut self.keys);
        let mut telex = Telex::new();
        let mut buf = Vec::new();
        for k in &keys {
            buf = telex.transform(&buf, *k);
        }
        self.keys = keys;
        self.telex = telex;
        self.buf = buf;
    }

    /// Đưa một ký tự qua đường apply đầy đủ: hỏi luật ngắt âm tiết trước, rồi
    /// mới gọi [`Telex::transform`].
    ///
    /// Phím ngắt được chốt NGUYÊN VĂN cùng buffer — khớp bản AHK, nơi
    /// `xin chaof` ra `xin chào` với dấu cách nằm trong text đã chốt. Shell
    /// IBus sau này có thể chọn chốt buffer rồi *forward* phím ngắt thay vì tự
    /// chốt nó; [`View`] mang đủ thông tin cho cả hai, nên đó là quyết định của
    /// shell chứ không phải của lõi.
    pub fn feed(&mut self, key: char) -> View {
        if ends_syllable(key) {
            let mut committed: String = self.buf.iter().collect();
            committed.push(key);
            self.buf.clear();
            // Text đã chốt thuộc về app, nên log phím KHÔNG được sống qua ranh
            // giới âm tiết: giữ lại là hứa một phép undo mà engine không có
            // quyền thực hiện.
            self.keys.clear();
            // Cờ shortcut `w` mô tả quan hệ giữa hai phím LIỀN NHAU trong cùng
            // một âm tiết. Âm tiết đã chốt thì quan hệ đó không còn nghĩa.
            self.telex = Telex::new();
            return View {
                composing: String::new(),
                committed,
                consumed: true,
            };
        }
        // Ghi log TRƯỚC, và ghi cả phím bị nuốt: log phải là thứ người dùng gõ,
        // không phải thứ engine chấp nhận. Xem chú thích của Session.
        self.keys.push(key);
        self.buf = self.telex.transform(&self.buf, key);
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
        for c in "chaof".chars() {
            s.feed(c);
        }
        let v = s.feed(' ');
        assert_eq!(v.committed, "chào ".to_string());
        assert_eq!(v.composing, "");
        assert_eq!(s.composing(), "");
    }

    #[test]
    fn reset_drops_the_syllable_without_committing_it() {
        let mut s = Session::new();
        for c in "chaof".chars() {
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

    /// Gõ một chuỗi vào phiên mới; `⌫` nghĩa là Backspace. Trả composing.
    fn typed(s: &str) -> String {
        let mut sess = Session::new();
        for c in s.chars() {
            if c == '⌫' {
                sess.backspace();
            } else {
                sess.feed(c);
            }
        }
        sess.composing()
    }

    #[test]
    fn two_tone_marks_are_unreachable() {
        // Đây là bug của bản AHK, tái hiện nguyên văn đường đi của nó:
        // "muafn" cho "muàn", cắt một ký tự cho "muà" (dấu kẹt ở vị trí không
        // còn là target), rồi "f" áp dấu THỨ HAI -> "mùà".
        //
        // Replay không có trạng thái để hỏng, nên chỗ này không cần một guard —
        // nó cần một test chứng minh guard là thừa.
        let out = typed("muafn⌫f");
        assert!(
            out.chars().filter(|c| "àáảãạùúủũụ".contains(*c)).count() <= 1,
            "một âm tiết không bao giờ mang hai dấu, ra {out:?}"
        );
        assert_ne!(out, "mùà");
        // ⌫ đưa buffer về đúng trạng thái sau "muaf", tức "mùa" — nên phím "f"
        // kế tiếp là ESCAPE và gỡ luôn dấu vừa có. Kết quả "muaf" không có dấu
        // nào, và đó là câu trả lời đúng: replay không để lại trạng thái nào
        // khiến phím thanh hiểu nhầm là mình đang gõ một dấu mới.
        assert_eq!(out, typed("muaff"));
        assert_eq!(out, "muaf");
    }

    #[test]
    fn cuarn_is_the_same_defect_and_is_also_unreachable() {
        assert_ne!(typed("cuarn⌫r"), "củả");
        assert_eq!(typed("cuarn⌫r"), typed("cuarr"));
    }

    #[test]
    fn backspace_undoes_a_keystroke_not_a_character() {
        // Truncate cho "tiến" (mất chữ t); replay cho "tiêng" (mất dấu, đúng
        // phím cuối mà người dùng vừa bấm).
        assert_eq!(typed("tieengs⌫"), "tiêng");
        assert_eq!(typed("tieengs⌫"), typed("tieeng"));
    }

    #[test]
    fn a_swallowed_key_has_nothing_to_undo_so_backspace_skips_past_it() {
        // Chữ "w" thứ hai trong "dduwow" bị nuốt (vần đã ở đúng dạng có móc).
        // Nếu Backspace pop đúng một phím, lần bấm ấy sẽ không đổi gì trên màn
        // hình — người dùng thấy Backspace "hỏng".
        let swallowed = typed("dduwow");
        assert_eq!(swallowed, typed("dduwo"), "phím w thứ hai phải bị nuốt");
        assert_ne!(typed("dduwow⌫"), swallowed, "Backspace phải đổi được text");
    }

    #[test]
    fn backspace_on_an_empty_engine_belongs_to_the_app() {
        let mut s = Session::new();
        let v = s.backspace();
        assert!(
            !v.consumed,
            "engine không có gì để undo thì không được nuốt phím"
        );
        assert_eq!(v.composing, "");
        assert_eq!(v.committed, "");
    }

    #[test]
    fn backspace_never_reaches_back_across_a_commit() {
        // Text đã chốt thuộc về app. Sau dấu cách, engine không còn gì để undo,
        // nên phím phải rơi xuống app thay vì kéo "chào" trở lại buffer.
        let mut s = Session::new();
        for c in "chaof ".chars() {
            s.feed(c);
        }
        let v = s.backspace();
        assert!(!v.consumed);
        assert_eq!(s.composing(), "");
    }

    #[test]
    fn the_w_shortcut_flag_does_not_leak_across_a_syllable_break() {
        // "w" rồi cách rồi "w": phím w thứ hai mở một âm tiết mới, nên nó là
        // shortcut đầy đủ chứ không phải nửa sau của một escape "ww".
        let mut s = Session::new();
        s.feed('w');
        s.feed(' ');
        s.feed('w');
        assert_eq!(s.composing(), "ư");
    }
}
