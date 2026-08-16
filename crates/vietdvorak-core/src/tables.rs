//! Bảng dữ liệu Telex. Mọi bảng khác trong crate được SUY RA từ đây.
//!
//! Bản AHK dựng `UPPER_VOWELS`, `PAIR_ESCAPE`, `W_MODIFIER` và `ALL_VOWEL_CHARS`
//! lúc khởi động bằng `BuildUpperVowels()`/`BuildPairEscape()`/... — không chép
//! tay bảng thứ hai. Bản Rust giữ đúng nguyên tắc đó nhưng suy ra tại chỗ dùng
//! thay vì dựng sẵn: [`VOWELS`] và [`DIACRITIC_PAIRS`] là hai bảng gốc duy nhất,
//! chữ HOA lấy bằng `to_uppercase` của Unicode, escape lấy bằng cách đọc ngược
//! `DIACRITIC_PAIRS`. Thêm một cặp ở đây là mọi thứ tự đúng theo.
//!
//! Không dùng `HashMap`: crate này `no_std` và các bảng chỉ vài chục phần tử,
//! nên quét tuyến tính vừa đủ nhanh vừa khỏi kéo thêm dependency vào đường đi
//! của phím.

/// Nguyên âm gốc → [không dấu, sắc, huyền, hỏi, ngã, nặng].
///
/// Chỉ có bản THƯỜNG. Bản hoa suy ra bằng [`crate::vowel::upper1`].
pub(crate) const VOWELS: [(char, [char; 6]); 12] = [
    ('a', ['a', 'á', 'à', 'ả', 'ã', 'ạ']),
    ('ă', ['ă', 'ắ', 'ằ', 'ẳ', 'ẵ', 'ặ']),
    ('â', ['â', 'ấ', 'ầ', 'ẩ', 'ẫ', 'ậ']),
    ('e', ['e', 'é', 'è', 'ẻ', 'ẽ', 'ẹ']),
    ('ê', ['ê', 'ế', 'ề', 'ể', 'ễ', 'ệ']),
    ('i', ['i', 'í', 'ì', 'ỉ', 'ĩ', 'ị']),
    ('o', ['o', 'ó', 'ò', 'ỏ', 'õ', 'ọ']),
    ('ô', ['ô', 'ố', 'ồ', 'ổ', 'ỗ', 'ộ']),
    ('ơ', ['ơ', 'ớ', 'ờ', 'ở', 'ỡ', 'ợ']),
    ('u', ['u', 'ú', 'ù', 'ủ', 'ũ', 'ụ']),
    ('ư', ['ư', 'ứ', 'ừ', 'ử', 'ữ', 'ự']),
    ('y', ['y', 'ý', 'ỳ', 'ỷ', 'ỹ', 'ỵ']),
];

/// Phím thanh → chỉ số thanh trong [`VOWELS`].
///
/// Đo bằng `bigram-late-tone.mjs`, kiểu gõ dấu-muộn, n=510 âm tiết có dấu: x/j
/// luân phiên tay 59.0% so với 54.1% của biến thể z/b toàn tay phải, +4.9pp.
/// Toàn bộ phần hơn đến từ nặng=j (+7.8pp tự nó). ngã=x đạt -2.9pp và được giữ
/// KHÔNG PHẢI vì luân phiên mà vì tương thích cơ bắp với Telex chuẩn.
pub(crate) const TONE_KEYS: [(char, usize); 5] = [
    ('s', 1), // sắc    — R-pinky home
    ('f', 2), // huyền  — R-index top
    ('r', 3), // hỏi    — R-ring top
    ('x', 4), // ngã    — L-index stretch, bottom
    ('j', 5), // nặng   — L-middle bottom
];

/// Cặp gõ → ký tự có dấu phụ. Bảng gốc thứ hai và cuối cùng.
pub(crate) const DIACRITIC_PAIRS: [(char, char, char); 7] = [
    // (phím 1, phím 2, kết quả)
    ('a', 'a', 'â'),
    ('e', 'e', 'ê'),
    ('o', 'o', 'ô'),
    ('a', 'w', 'ă'),
    ('o', 'w', 'ơ'),
    ('u', 'w', 'ư'),
    ('d', 'd', 'đ'),
];

/// Dấu móc đặt theo CỤM nguyên âm, không theo từng ký tự.
///
/// Chỉ liệt kê những cụm mà dấu móc KHÔNG rơi vào nguyên âm cuối; mọi cụm khác
/// đi theo đường mặc định (cặp aw/ow/uw, hoặc quét ngược).
///
/// - `uu` → `ưu` — lựu, hưu, cứu (`uư` không tồn tại trong tiếng Việt)
/// - `uou` → `ươu` — rượu, hươu, bướu (`uoư` không tồn tại)
pub(crate) const HORN_NUCLEUS: [(&str, &str); 2] = [("uu", "ưu"), ("uou", "ươu")];

/// Một luật chuẩn hoá vần: dạng sai → dạng đúng chính tả, kèm điều kiện chờ.
pub(crate) struct NucleusRule {
    /// Cụm nguyên âm gốc, viết thường, đã bỏ dấu thanh.
    pub shape: &'static str,
    /// Dạng đúng.
    pub fixed: &'static str,
    /// CHỜ ĐUÔI: chỉ sửa khi vần còn ký tự phía sau.
    pub lazy: bool,
}

/// Chuẩn hoá vần: cụm nguyên âm đặt dấu móc SAI CHỖ → dạng đúng chính tả.
///
/// Cả hai mục đều quy về `ươ`, và `ươ` là dạng đúng DUY NHẤT, nên phép sửa là
/// xác định. Hai mục khác nhau ở đúng một điểm, và điểm đó là thuộc tính CHÍNH
/// TẢ của từng vần chứ không phải một lựa chọn của thuật toán:
///
/// - `ưo` → `lazy: false`, SỬA NGAY. Vì `ưo` không hợp lệ ở MỌI độ dài: không
///   âm tiết tiếng Việt nào chứa `ưo`, dù nó đứng cuối (`đưo`) hay còn đuôi
///   (`đưoc`). Không có dạng hợp lệ nào để bảo vệ, nên không có gì phải chờ —
///   chờ chỉ tổ để người dùng nhìn thấy `đưo`, thứ không tồn tại.
/// - `uơ` → `lazy: true`, PHẢI CHỜ. Vì `uơ` tự nó là một vần MỞ hợp lệ: thuơ
///   (thuở), huơ, quơ. Sửa ngay là phá đúng chính tả người dùng vừa gõ ra. Chỉ
///   khi có ký tự tiếp theo mới biết chắc vần không dừng ở dạng mở đó
///   (`thuơng` → `thương`).
///
/// Luật chung — *eager khi vần sai ở mọi độ dài, lazy khi vần còn là dạng mở hợp
/// lệ* — là tiên đề **A1.2** (`PRINCIPLES.md`), không viết lại ở đây. Hệ quả cho
/// bảng này: cờ `lazy` là **dữ liệu của từng vần**, nên thêm một vần mới chỉ là
/// thêm một dòng, không phải thêm một nhánh if-else hỏi tên vần.
pub(crate) const NUCLEUS_NORMALIZE: [NucleusRule; 2] = [
    NucleusRule {
        shape: "ưo",
        fixed: "ươ",
        lazy: false,
    },
    NucleusRule {
        shape: "uơ",
        fixed: "ươ",
        lazy: true,
    },
];

/// Cặp âm đệm + nguyên âm chính mà hai quy ước đặt dấu khác nhau.
///
/// Xem [`crate::TONE_PLACEMENT_MODERN`]. Cả ba cặp phải được xử lý GIỐNG NHAU —
/// trước đây chỉ `uy` được ưu tiên nên bản AHK tự mâu thuẫn: thuỷ theo kiểu mới
/// nhưng hòa/khỏe theo kiểu cũ.
pub(crate) const GLIDE_PAIRS: [(char, char); 3] = [('o', 'a'), ('o', 'e'), ('u', 'y')];

/// Nguyên âm gốc có nhận được dấu móc/trăng từ phím `w` không, và ra gì.
///
/// SUY RA từ [`DIACRITIC_PAIRS`]: chính là các cặp có phím thứ hai là `w`.
pub(crate) fn w_modifier(base: char) -> Option<char> {
    DIACRITIC_PAIRS
        .iter()
        .find(|(first, second, _)| *second == 'w' && *first == base)
        .map(|(_, _, result)| *result)
}

/// Nghịch đảo của [`DIACRITIC_PAIRS`]: ký tự kết quả → (phím 1, phím 2).
///
/// Cho phép gõ lại phím trigger để huỷ dấu phụ: `oww` → `ow`, `aaa` → `aa`,
/// `ddd` → `dd`. Nhận cả ký tự hoa; khi đó phím 1 trả về cũng hoa.
pub(crate) fn pair_escape(ch: char) -> Option<(char, char)> {
    if let Some((first, second, _)) = DIACRITIC_PAIRS.iter().find(|(_, _, r)| *r == ch) {
        return Some((*first, *second));
    }
    // Thử bản hoa. `to_lowercase` của Unicode là nghịch đảo của `StrUpper` mà
    // BuildPairEscape dùng, nên không cần bảng hoa riêng.
    let lo = crate::vowel::lower1(ch)?;
    if lo == ch {
        return None;
    }
    let (first, second, _) = DIACRITIC_PAIRS.iter().find(|(_, _, r)| *r == lo)?;
    Some((crate::vowel::upper1(*first)?, *second))
}

/// Cặp `(phím 1, phím 2)` này có tạo ra dấu phụ không?
pub(crate) fn diacritic_pair(first: char, second: char) -> Option<char> {
    DIACRITIC_PAIRS
        .iter()
        .find(|(f, s, _)| *f == first && *s == second)
        .map(|(_, _, r)| *r)
}

/// Chỉ số thanh của phím này, nếu nó là phím thanh.
pub(crate) fn tone_index(key: char) -> Option<usize> {
    let lo = crate::vowel::lower1(key)?;
    TONE_KEYS.iter().find(|(k, _)| *k == lo).map(|(_, i)| *i)
}
