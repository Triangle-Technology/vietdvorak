# VietDvorak

Bộ gõ Telex tiếng Việt tối ưu cho bàn phím **Dvorak**.

Hai thứ trong một repo:

- **`VietDvorak.ahk`** — bản đang chạy, AutoHotkey v2, **chỉ Windows**. Dùng
  được hàng ngày, với một defect đã biết và đã khoanh vùng: bấm Backspace vào
  giữa một âm tiết đang gõ rồi gõ tiếp phím dấu có thể để lại hai dấu (`muafn`
  ⌫ `f` → `mùà`). Cố ý chưa sửa; lý do và đường sửa ở §6 của
  [ARCHITECTURE.md](ARCHITECTURE.md). Bản Rust không thể chạm tới lỗi này về mặt
  cấu trúc. Yêu cầu bật sẵn layout *United States-Dvorak* của Windows.
- **`crates/`** — bản viết lại bằng Rust, đích là chạy được cả Linux và macOS
  mà không bắt người dùng tự cấu hình layout. **Đang làm, chưa có shell nào cho
  OS** — lõi đã xong và đã được chứng minh khớp bản AHK.

---

## Kiểm tra mọi thứ

Một lệnh, và nó là lệnh duy nhất đáng chạy:

```bash
node check-parity.mjs
```

```
PASS  281 vectors — all three engines: JS 266/266, AHK 266/266, RUST 281/281
```

Nó chạy mọi vector trong `telex-conformance.txt` qua **mọi engine mà section đó
là spec cho**, trong một lần chạy, rồi exit khác 0 nếu bất kỳ bên nào lệch: 266
vector đi qua cả **ba** bản implement của cùng một engine — `Transform()` của
AutoHotkey, bản mirror JS trong `practice.html`, và crate Rust — còn 15 vector
`@section backspace` chỉ đi qua bản Rust, vì AHK truncate còn Rust replay, và đó
là khác biệt **kiến trúc** chứ không phải một bên sai (§6 của ARCHITECTURE.md).
Ba bản không thể trôi khỏi nhau mà không ai biết.

Cần Node, Rust (`cargo`) và AutoHotkey v2 (Windows). `--skip-ahk` / `--skip-rust` bỏ qua một
engine cho một lần chạy yên tĩnh hơn — nhưng **vẫn exit khác 0**, có chủ ý: bỏ
qua một engine thì đó không còn là một lần kiểm đầy đủ, và exit code là thứ duy
nhất CI đọc.

Phía Rust:

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo check --target x86_64-unknown-linux-musl   # gate Linux, chạy từ Windows
```

---

## Cách gõ

Telex chuẩn. Đặt dấu thanh thông minh — gõ phím dấu ở đâu trong âm tiết cũng
được, dấu tự về đúng nguyên âm:

| gõ | ra | |
|---|---|---|
| `tieengs` | tiếng | dấu trên ê đã sửa |
| `ngoaif` | ngoài | dấu trên nguyên âm chính, không phải nguyên âm cuối |
| `soajn` | soạn | phím dấu gõ giữa chừng, phụ âm cuối gõ sau — dấu vẫn đúng chỗ |
| `dduwowngf` | đường | mọi cách viết đều hội tụ |
| `ass` | as | bấm đôi phím dấu để huỷ |

Phím dấu: `s` sắc · `f` huyền · `r` hỏi · `x` ngã · `j` nặng.
Dấu phụ: `aa ee oo` · `aw ow uw` · `dd`.

Mở `practice.html` trong trình duyệt để có bàn phím tương tác, bảng phím tắt và
bài luyện gõ. Nó cũng chính là help của dự án.

---

## Tài liệu

| | |
|---|---|
| [ARCHITECTURE.md](ARCHITECTURE.md) | Vì sao dự án có hình dạng này, và **cái gì đã bị bác bỏ** |
| [ROADMAP.md](ROADMAP.md) | Đang ở đâu, cái gì chặn cái gì |
| [CLAUDE.md](CLAUDE.md) | Bất biến phải giữ khi sửa repo này |
| `telex-conformance.txt` | **Spec.** Nguồn sự thật duy nhất cho hành vi Telex |

Đọc `ARCHITECTURE.md` trước khi đề xuất đổi một quyết định lớn — phần lớn các
lựa chọn nghe hợp lý đã được cân nhắc và bác bỏ, kèm lý do.

---

## Giấy phép

[MIT](LICENSE)
