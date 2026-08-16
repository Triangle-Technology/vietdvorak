# Bất biến của repo này

Chỉ ghi những gì **riêng của dự án này**. Bốn nguyên tắc chung (long-term first,
first-principles, single source of truth, root-cause-clean) đã ở global — không
chép lại ở đây.

Lý do kiến trúc: [ARCHITECTURE.md](ARCHITECTURE.md). Trạng thái:
[ROADMAP.md](ROADMAP.md).

---

## 1. `telex-conformance.txt` là spec. Không phải code nào cả.

Ba engine implement cùng một hành vi trên ba runtime không chia sẻ code được.
Spec là thứ chúng chia sẻ.

- **Đổi hành vi quan sát được ⇒ sửa spec trong CÙNG commit.** Không có ngoại lệ.
- **Một parser, một verdict.** `check-parity.mjs` là chỗ duy nhất đọc file spec
  và là chỗ duy nhất quyết định pass/fail. `vdconform` tính và in, không phán
  xét. **Đừng thêm `tests/conformance.rs` tự parse file `.txt`** — đó là parser
  thứ hai và verdict thứ hai cho một spec, và chúng sẽ trôi khỏi nhau.
- Test trong crate chỉ pin những bất biến mà một vector **không phát biểu được**
  — cờ `consumed`, "một âm tiết không bao giờ mang hai dấu". Không nhân bản spec.
- Không phải section nào cũng áp dụng cho cả ba engine. Bảng `SECTION_ENGINES`
  trong `check-parity.mjs` phải **khớp** phần văn xuôi trong header file spec —
  hai chỗ đó nói cùng một điều, đừng để một bên đổi mà bên kia không.

## 2. Đừng tuyên bố gì trước khi chạy `node check-parity.mjs`

Nó là verdict. `cargo test` xanh không có nghĩa engine đúng — nó chỉ có nghĩa các
bất biến ở §1 còn giữ.

## 3. `VietDvorak.ahk` là oracle. Đừng sửa ngữ nghĩa của nó.

Nó vừa là bộ gõ đang dùng hàng ngày, vừa là bên thứ hai trong phép so ba chiều.
Sửa hành vi của nó là **dịch chuyển baseline mà bản port đang cố khớp**.

Nó có một defect đã biết (backspace tạo được hai dấu — §6 của ARCHITECTURE.md).
Đó là **cố ý chưa sửa**, và lý do nằm ở đó. Nếu bug cắn thật khi dùng hàng ngày:
backport thiết kế replay *đã validate bằng Rust* ngược sang AHK. Đừng phát minh
ngữ nghĩa hai lần.

## 4. Không dependency nào vào lõi

`crates/vietdvorak-core` có **zero dependency**, và CI enforce điều đó bằng
`cargo tree`. Đây là thuộc tính **bảo mật**, không phải thẩm mỹ: mọi crate trên
đường đi của phím đọc được mọi thứ người dùng gõ.

Shell được phép có dependency, nhưng tối thiểu, và mỗi cái phải trả lời được câu
"crate này thấy được gì".

## 5. Suy ra, đừng chép

`VOWELS` và `DIACRITIC_PAIRS` là hai bảng **được suy ra TỪ** — code gọi chúng là
"bảng gốc". Chữ hoa suy ra bằng Unicode (`vowel.rs`); escape và w-modifier đọc
ngược `DIACRITIC_PAIRS` (`tables.rs`). Chúng KHÔNG phải hai bảng viết tay duy
nhất: `TONE_KEYS`, `HORN_NUCLEUS`, `NUCLEUS_NORMALIZE`, `GLIDE_PAIRS`
(`tables.rs`) và `MODIFIED` (`tone.rs`) cũng viết tay, chỉ là không có gì phái
sinh từ chúng.

Thêm một cặp ở `DIACRITIC_PAIRS` thì escape và w-modifier tự đúng theo, nhưng
**chưa đủ**: nguyên âm mới còn phải vào `VOWELS` (sáu dạng thanh —
`find_vowel_info` chỉ tra ở đó) và vào `MODIFIED`.

Ở đâu thấy một bảng thứ hai đang được giữ đồng bộ bằng tay, đó là bug đang chờ.
*(Hai cái đang tồn tại: `MODIFIED` trong `tone.rs` — đúng bằng phần nguyên âm của
`DIACRITIC_PAIRS`, suy ra được nhưng đang chép tay; và bảng phím Dvorak, hiện bị
chép ở CẢ `practice.html :: KEYBOARD_LAYOUT` lẫn `viet-corpus.mjs` — §3 của
ARCHITECTURE.md.)*

## 6. Đừng xây seam cho consumer chưa tồn tại

Một seam chỉ có một consumer là seam chưa được kiểm chứng. Nhánh injection của
`View` **chưa được xây** dù `View.committed` đã có chỗ cho nó — vì
`VietDvorak.ahk :: SyncToScreen()` đã là bản implement tham chiếu chạy được,
port khi có shell Windows thật.

Cùng lý do: chưa có `deny.toml` (chưa có dependency để ban), chưa có
`ACCEPTANCE.md` (chưa có shell để test thủ công).

## 7. Kiểm chứng bằng phần cứng, không bằng tài liệu

§9 của ARCHITECTURE.md liệt kê những khẳng định về GNOME/IBus **chưa từng được
quan sát trên máy Ubuntu thật**. Một trong số đó đã đảo chiều **hai lần** qua ba
vòng nghiên cứu (IBus giao keycode hệ nào).

Đừng hardcode một dữ kiện nền tảng vì tài liệu nói vậy. Pin nó bằng một assert
lúc bring-up.

## 8. Guard phải được chứng minh là cắn

Thêm một guard thì thêm luôn cách chứng minh nó chặn thật, không chỉ chứng minh
đường xanh vẫn xanh. Tiền lệ trong repo:

- CI có bước assert `--skip-ahk` / `--skip-rust` **không bao giờ** exit 0
  (step "A skipped engine can never be green")
- CI trỏ `AUTOHOTKEY_EXE` vào đường dẫn không tồn tại và đòi parity phải đỏ, để
  một bước cài hỏng không lặng lẽ tụt xuống còn hai engine (step "A broken
  AutoHotkey install must turn the build red")

Bước thứ hai được thêm SAU khi file này lần đầu khẳng định nó tồn tại trong khi
nó không. Hành vi vẫn luôn đúng, nhưng nó mới chỉ được kiểm bằng tay một lần —
đúng thứ mục này cấm.

## 9. Ngôn ngữ

Chú thích lý do, **doc comment** (`///`, `//!`) và **thông báo lỗi**: tiếng Việt
— kể cả doc API của Rust và stderr của `vdconform` / `VietDvorak.ahk`. Chỉ **tên
định danh** là tiếng Anh (`find_tone_target`, `Session::backspace`,
`SECTION_ENGINES`…).

Ngoại lệ có sẵn, giữ nguyên: `check-parity.mjs` và `practice.html` viết chú
thích và thông báo bằng tiếng Anh.
