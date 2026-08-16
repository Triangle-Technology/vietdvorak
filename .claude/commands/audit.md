---
description: Audit một thay đổi theo PRINCIPLES.md + CLAUDE.md + AD log — verdict từng mục (COMPLIES/VIOLATES/N/A), dẫn chứng cụ thể, không rubber-stamp
argument-hint: "[target — vd một file/crate/commit; trống = thay đổi trên working tree]"
---

Audit thay đổi dưới đây theo ba tầng governance của **VietDvorak**, và ba tầng đó nói ba loại chuyện khác nhau — đừng gộp:

| Tầng | Là gì |
|---|---|
| [PRINCIPLES.md](../../PRINCIPLES.md) | **tiên đề** — tiếng Việt / sản phẩm vốn dĩ như vậy |
| [CLAUDE.md](../../CLAUDE.md) | **chỉ dẫn** — khi sửa repo thì đừng làm gì |
| [docs/architecture-decisions.md](../../docs/architecture-decisions.md) | **quyết định** — ngày D chốt X vì Y (**TOÀN BỘ AD đang có trong file**; số AD tăng theo thời gian, đừng chốt dải số ở đâu cả) |

Cho verdict từng mục (COMPLIES / VIOLATES / N/A) kèm dẫn chứng cụ thể. **KHÔNG rubber-stamp; chủ động đi tìm vi phạm TRƯỚC.**

**Target**: $ARGUMENTS
(Trống → audit thay đổi đang có trên working tree.)

**Đọc file thật, đừng dựa trí nhớ.** Ba file trên là nguồn; file này **cố ý không chép định nghĩa nào từ chúng sang** — bản sao thứ hai sẽ lệch âm thầm, mà đó đúng là thứ [AD-1](../../docs/architecture-decisions.md#ad-1) cấm.

**Trích dẫn bằng SYMBOL, không bằng số dòng.** Repo này đã tự cấm số dòng hai lần kèm lý do (`viet-corpus.mjs`, `check-parity.mjs`) và vẫn tái phạm ở lượt viết tài liệu đầu tiên — ba trích dẫn vào `practice.html` đúng ở working tree nhưng **đã sai ở HEAD**. Viết `practice.html :: KEYBOARD_LAYOUT`, không viết `practice.html:1276`.

---

## ⚠ BỐN BẪY — đọc trước khi cờ bất cứ gì

Đây là những thứ **trông như** vi phạm nhưng là **quyết định**. Cờ nhầm ở đây là đòi phá đúng phần đã được suy nghĩ kỹ nhất.

1. **`VietDvorak.ahk` có defect backspace tạo được hai dấu — CHƯA sửa là ĐÚNG THIẾT KẾ** ([AD-9](../../docs/architecture-decisions.md#ad-9)). Sửa nó sẽ **dịch chuyển baseline mà bản port đang cố khớp**. Đừng đòi fix.
2. **`@section backspace` chỉ chạy trên Rust — KHÔNG phải lỗ hổng phủ sóng** ([AD-3](../../docs/architecture-decisions.md#ad-3)). AHK truncate còn Rust replay; đó là khác biệt kiến trúc, không phải một bên sai. Đừng đọc thành "15 vector AHK bị bỏ".
3. **Nhánh injection của `View` chưa xây — là CỐ Ý** ([AD-13](../../docs/architecture-decisions.md#ad-13)). Cùng lý do cho `deny.toml` và `ACCEPTANCE.md` chưa tồn tại. Đừng cờ "thiếu".
4. **Lõi KHÔNG tự implement Dvorak — là ĐÚNG** ([AD-4](../../docs/architecture-decisions.md#ad-4)). Layer 1 thuộc tầng layout của OS. Đừng cờ "chưa làm layer 1".

---

## 1. Bốn nguyên tắc nền

Chấm **verdict + WHY cho TỪNG cái**, không gộp. Định nghĩa gốc ở CLAUDE.md global của user — đọc thật.

**long-term · first-principles · single-source · root-cause**

Xung đột giữa bốn → long-term thắng.

Bản chất bài toán repo này, để chấm *first-principles*: *spec là dữ liệu chứ không phải code; ba engine trên ba runtime không chia sẻ code được nên chỉ có spec là thứ chung; lõi không biết OS; trạng thái sai phải bất khả biểu diễn chứ không phải được canh chừng.* Một giải pháp mâu thuẫn mấy axiom này thì **sai kể cả khi nó chạy xanh**.

## 2. Tuân thủ PRINCIPLES.md — tiên đề

Đọc [PRINCIPLES.md](../../PRINCIPLES.md) thật. Một giải pháp mâu thuẫn tiên đề thì **sai kể cả khi nó chạy xanh** — cờ nó ngay cả khi mọi cổng đang xanh.

- **A1** — thay đổi có tạo ra được một chuỗi KHÔNG tồn tại trong tiếng Việt không, dù chỉ thoáng qua trong buffer? Có chỗ nào *canh chừng* một trạng thái xấu thay vì làm nó **không tới được** không?
- **A2** — có chỗ nào gắn dấu vào vị trí phím vừa bấm thay vì tính lại target của cả âm tiết không?
- **A3** — thêm luật mới có làm hai thứ tự gõ của cùng một từ cho hai kết quả không? Cách rẻ nhất để kiểm: một assert `typed(cách_1) == typed(cách_2)` cho các hoán vị, đúng kiểu `tone_order_does_not_matter` trong `transform.rs`.
- **A4** — thêm một phép chuẩn hoá tự động mà **không** dạy escape biết đường tránh nó?
- **A5** — đổi/thêm gán phím mà không có phép đo, hoặc có đo nhưng không nói rõ khi đo thua một ràng buộc khác?
- **Anti-principles** — thay đổi có kéo dự án về phía VNI/VIQR, layout khác, đoán ý người gõ, hay tính năng cần mạng không?

## 3. Tuân thủ CLAUDE.md

Rà từng mục. Đặc biệt:

- **§1 Một parser, một verdict** — có ai đọc `telex-conformance.txt` ngoài `check-parity.mjs` không? Có `tests/*.rs` nào tự parse spec không? `vdconform` có bắt đầu phán xét pass/fail không? → VIOLATES.
- **§1 Đổi hành vi ⇒ sửa spec CÙNG commit** — thay đổi có đổi hành vi quan sát được mà spec không đổi không?
- **§1 `SECTION_ENGINES` khớp header spec** — mở CẢ HAI ra so. Đây là hai chỗ nói cùng một điều và **không ai enforce**; lệch là lệch im lặng.
- **§2 Trỏ bằng `AD-n` không phải `§n`** — có con trỏ `§<số>` mới nào vào một file .md không? Có ai **sửa** một AD `locked` thay vì append không?
- **§5 Lõi zero dependency** — `crates/vietdvorak-core/Cargo.toml` có `[dependencies]` không?
- **§6 Suy ra, đừng chép** — có bảng thứ hai nào đang giữ đồng bộ bằng tay không? *(Hai cái ĐÃ BIẾT và đã ghi nợ: `MODIFIED` trong `tone.rs`; bảng Dvorak ở `practice.html :: KEYBOARD_LAYOUT` + `viet-corpus.mjs`. Cờ cái MỚI, đừng báo lại hai cái này như phát hiện.)*
- **§9 Guard phải chứng minh là cắn** — thay đổi có thêm guard/kiểm tra nào mà **không** kèm cách chứng minh nó chặn thật không? Đây là mục repo này đã tự vi phạm một lần: CLAUDE.md từng viện dẫn một bước CI không tồn tại, **ngay trong mục này**.
- **§10 Ngôn ngữ** — doc comment và thông báo lỗi trong `crates/` phải tiếng Việt; tên định danh tiếng Anh. `check-parity.mjs`/`practice.html` là ngoại lệ có sẵn.

## 4. Kiểm AD log

- Thay đổi có mâu thuẫn một AD đã `locked` không? Rà **mọi AD đang có** — **mở file đếm**, đừng tin dải số viết sẵn ở đâu (dải cứng luôn bỏ sót AD MỚI NHẤT, tức đúng phần code vừa viết).
- Quyết định kiến trúc mới → đã append AD chưa? Thay AD cũ → đã ghi `superseded by` chưa?
- AD `draft` nào đã xong việc mà chưa chuyển `locked`? Ngược lại, có ai `lock` một AD mà việc còn dang dở không?

## 5. Kiểm RĂNG của bộ test — không chỉ kiểm nó xanh

Xanh chỉ chứng minh *"engine khớp spec ở những chỗ spec nói tới"*. Nó **không** chứng minh spec sẽ đỏ khi engine sai.

- Thay đổi có thêm nhánh logic nào mà **không** có vector nào chạm tới không?
- Nếu thay đổi đụng lõi: chạy `scripts/dot-bien.sh` với một đột biến vào đúng nhánh vừa sửa. **Bốn luật đọc kết quả ở [AD-17](../../docs/architecture-decisions.md#ad-17) — đọc trước khi kết luận.** Luật 4 là luật đã cắn ngay lần chạy đầu tiên: chuỗi nhiều dòng bị từ chối, vì nó từng báo "SỐNG SÓT" cho một đột biến chưa bao giờ được áp.
- 🔴 **Dán nhãn "mutation-proven" mà chưa CHẠY đột biến = một lời nói dối.**
- ⚠ **`check-parity.mjs` xanh KHÔNG đủ, và `cargo test` xanh cũng KHÔNG đủ.** Hai cổng phủ hai vùng khác nhau, giao nhau một phần — đã đo ở AD-17: đột biến `ends_syllable` sống sót qua parity, chỉ unit test bắt được. Cổng thật là **cả hai**.

## 6. Kiểm dữ kiện nền tảng

- Có chỗ nào **hardcode** một dữ kiện về GNOME/mutter/GTK/IBus dựa trên tài liệu, mà không có assert lúc bring-up không? → VIOLATES [AD-14](../../docs/architecture-decisions.md#ad-14). Nhắc lại: câu "IBus giao keycode theo hệ nào" đã **đảo chiều hai lần**.
- Có khẳng định mới nào về nền tảng được viết ở thì **hiện tại quả quyết** trong khi chưa quan sát trên máy Ubuntu thật không? Bảng "Chưa kiểm trên phần cứng thật" trong ARCHITECTURE.md có cần thêm dòng không?

## 7. Kiểm tài liệu có đang nói dối không

Repo này đã có **24 lỗi dữ kiện** trong lượt viết tài liệu đầu tiên. Với MỖI khẳng định kiểm được mà thay đổi vừa thêm hoặc chạm tới: **mở file / chạy lệnh để xác minh**, đừng đọc lướt.

- Con số bị chép vào văn xuôi (số vector, số section, số bảng, số dòng) — con số sống ở output của harness, mọi chỗ chép nó đều đã từng sai.
- Lệnh được ghi trong tài liệu — chạy thử, nó có tồn tại và có chạy được không?
- Bước CI được viện dẫn — mở `ci.yml` xem nó có thật không.
- Tham chiếu file:symbol — mở ra xem symbol có ở đó không.

---

Sau đó:

- Liệt kê riêng các **điểm yếu thành thật** (nợ đã biết, bất biến assert-nhưng-chưa-enforce, khẳng định chưa kiểm phần cứng) kể cả khi chưa phải vi phạm rõ.
- Kết bằng **MỘT verdict tổng** + nêu rõ có cần fix ngay session này không, và nếu có thì fix gì.
- Audit nhỏ trong phiên → trả lời tại chỗ. **Đừng sinh file.** `docs/audit-*.md` chỉ dành cho audit lớn (cuối một milestone), và khi đó theo thứ tự: bốn nguyên tắc nền (verdict + WHY từng cái) → verdict tổng → đã xử lý → findings/nợ → cách tái hiện.
