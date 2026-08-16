# Nguyên tắc — tiên đề của bài toán

Những điều **đúng về tiếng Việt và về sản phẩm này**, độc lập với code. Chúng sống sót
kể cả khi toàn bộ engine bị viết lại bằng ngôn ngữ khác — đây là thứ đưa cho người viết
engine **thứ tư**.

**Ba loại phát biểu, ba chỗ. Đừng lẫn:**

| | Chỗ |
|---|---|
| *"Tiếng Việt / Telex vốn dĩ là như vậy"* — tiên đề | **file này** |
| *"Ngày D chốt X vì Y"* — quyết định | [AD log](docs/architecture-decisions.md) |
| *"Khi sửa repo, đừng làm X"* — chỉ dẫn | [CLAUDE.md](CLAUDE.md) |

File này **chỉ trỏ xuống** AD nào phái sinh từ một tiên đề; AD **không** chép tiên đề
ngược lên. Bốn nguyên tắc kỹ thuật chung (long-term first, first-principles, single source
of truth, root-cause-clean) ở global — không ở đây.

Phân vân một quyết định thiết kế → quay về đây trước. Một giải pháp mâu thuẫn tiên đề thì
**sai kể cả khi nó chạy xanh**.

---

## A1 — Người gõ không bao giờ được thấy một chuỗi không tồn tại trong tiếng Việt

Tiên đề bao trùm. Buffer đang gõ dở là thứ người dùng **nhìn thấy**, nên nó phải luôn là
một chuỗi tiếng Việt hợp lệ hoặc một tiền tố hợp lệ của một chuỗi như vậy.

Hai hệ quả cụ thể, và cả hai đều đã cắn thật:

**A1.1 — Một âm tiết mang TỐI ĐA MỘT dấu thanh.** Không phải một guard cần nhớ bật; là
một sự thật. `mùà` không phải "kết quả xấu", nó là **chuỗi không biểu đạt được**. Thiết kế
nào để trạng thái đó *có thể* xảy ra rồi canh chừng là thiết kế sai — thiết kế đúng làm nó
không tới được. → [AD-9](docs/architecture-decisions.md#ad-9)

**A1.2 — Sửa NGAY khi dạng sai ở mọi độ dài; CHỜ khi dạng đó còn là một vần hợp lệ.**
`ưo` không hợp lệ dù đứng cuối (`đưo`) hay còn đuôi (`đưoc`) → không có gì để bảo vệ, sửa
ngay. `uơ` **tự nó là vần mở hợp lệ** (thuở, huơ, quơ) → sửa ngay là phá đúng chính tả
người dùng vừa gõ ra; phải chờ ký tự tiếp theo.

Đây là thuộc tính **chính tả của từng vần**, không phải lựa chọn của thuật toán. Vì vậy
điều kiện phải sống trong **bảng**, không trong if-else hỏi tên vần — thêm một vần mới chỉ
là thêm một dòng.

## A2 — Dấu thanh thuộc về ÂM TIẾT, không thuộc về phím vừa bấm

Người gõ bấm phím dấu **ở đâu trong âm tiết cũng được**; engine đặt nó lên đúng nguyên âm.
`tieengs` → `tiếng`, `ngoaif` → `ngoài`, `Quynhf` → `Quỳnh`.

Đây là lời hứa cốt lõi của sản phẩm, không phải một tính năng thêm vào. Hệ quả: cái thường
được gọi là "dấu di trú" **không phải tính năng** — nó là cùng một luật chạy lại sau khi
âm tiết đổi hình (`muaf` → `mùa`, rồi `n` → `muàn`).

## A3 — Mọi cách viết của cùng một từ phải HỘI TỤ

`dduwongf` = `dduowngf` = `dduwowngf` = `dduongwf` → `đường`. Bốn thứ tự gõ, một kết quả.

Hệ quả trực tiếp: **thứ tự dấu thanh và dấu móc không được ảnh hưởng kết quả**
(`luujw` = `luuwj` = `lựu`). Đó là lý do mọi phép sửa vần phải giữ nguyên dấu thanh sẵn có
thay vì viết lại ở từng chỗ gọi.

Người gõ nhanh không nghĩ theo thứ tự chuẩn. Một engine bắt họ nghĩ là engine bắt họ chậm lại.

## A4 — Escape huỷ thứ NGƯỜI DÙNG gõ ra, không bao giờ huỷ thứ ENGINE tự chèn

Gõ lại phím trigger là lệnh **huỷ** (`oww` → `ow`, `aaa` → `aa`) — nhưng chỉ huỷ được dấu
mà chính người dùng vừa tạo.

Trong `dduwo`, engine đã tự sửa `ưo` thành `ươ` (A1.2). Dấu móc trên `ơ` là **của engine**,
nên phím `w` kế tiếp là **thừa** — yêu cầu "thêm móc" đã được đáp ứng — chứ không phải lệnh
huỷ. Phải nuốt nó (`dduwowngf` → `đường`).

Ngược lại trong `oww`, dấu móc do chính người dùng gõ bằng cặp `ow`, nên escape chạy bình
thường.

Luật này khái quát hơn ca đang có: **mỗi lần thêm một phép chuẩn hoá tự động là thêm một
chỗ escape phải biết đường tránh.**

## A5 — Đặt phím bằng ĐO, không bằng thẩm mỹ; và nói rõ khi đo thua tương thích

Phím thanh `s/f/r/x/j` không phải chọn cho đẹp. Đo bằng `bigram-late-tone.mjs` trên ngữ
liệu thật (kiểu gõ dấu-muộn, n=510 âm tiết có dấu): x/j cho luân phiên tay 59.0% so với
54.1% của biến thể toàn tay phải, **+4.9pp** — và toàn bộ phần hơn đến từ `nặng=j`
(+7.8pp tự nó).

Nhưng `ngã=x` đạt **−2.9pp** và **vẫn được giữ**, vì tương thích cơ bắp với Telex chuẩn
thắng con số ở đúng chỗ này.

Nguyên tắc là cả hai vế: **đo trước**, và khi phép đo thua một ràng buộc khác thì **nói rõ
nó thua ai và vì sao**, đừng lặng lẽ chọn rồi để người sau tưởng là kết quả tối ưu.

---

## Anti-principles — điều KHÔNG theo đuổi

Kỷ luật phạm vi. Mỗi dòng ở đây là một hướng nghe hợp lý mà dự án cố ý không đi.

- **Không làm bộ gõ đa phương thức.** Chỉ Telex. Không VNI, không VIQR, không Telex mở
  rộng. Mỗi phương thức thêm vào là một bộ luật đặt dấu nữa phải giữ đồng bộ qua *ba*
  engine, và không ai trong dự án gõ chúng.
- **Dvorak là TIỀN ĐỀ, không phải tùy chọn.** Không hỗ trợ QWERTY/Colemak/AZERTY. Tên dự
  án là câu trả lời; phím thanh được chọn theo hình học Dvorak (A5) nên "hỗ trợ thêm một
  layout" không phải thêm cấu hình mà là **đo lại từ đầu**.
- **Không đua tính năng với Unikey / EVKey / OpenKey / ibus-bamboo.** Chúng phục vụ mọi
  người gõ tiếng Việt; cái này phục vụ người gõ Telex trên Dvorak. Tổng quát hoá non là
  cái bẫy đốt thời gian lớn nhất.
- **Không đoán ý người gõ.** Không tự sửa chính tả, không gợi ý từ, không từ điển, không
  học thói quen. Engine biến đổi phím theo luật xác định; nó không có ý kiến về việc người
  dùng định viết gì.
- **Không tính năng nào cần mạng.** Không cập nhật tự động, không telemetry, không đồng bộ
  cấu hình. Đây là ràng buộc **sản phẩm**, và nó có mặt vì chương trình này đọc mọi phím
  người dùng gõ. Cách nó được enforce nằm ở [AD-10](docs/architecture-decisions.md#ad-10).
- **Không tổng quát hoá cho người dùng thứ hai trước khi có người dùng thứ hai.** v0 phục
  vụ đúng một người trên đúng một máy. Xem [ROADMAP.md](ROADMAP.md).
