# Kiến trúc — bản đồ

Hệ thống **là gì**, file **ở đâu**, và cái gì **chưa được kiểm trên phần cứng thật**.

**Tiên đề của bài toán** — điều đúng về tiếng Việt và về sản phẩm, độc lập với code —
nằm ở [PRINCIPLES.md](PRINCIPLES.md).

**Vì sao các quyết định là như vậy thì KHÔNG nằm ở đây** — nó sống ở
[docs/architecture-decisions.md](docs/architecture-decisions.md), append-only, mỗi entry
một mỏ neo vĩnh viễn (`AD-7`). File này **trỏ** vào đó và không chép lại. Thấy một lý do
xuất hiện ở cả hai chỗ → chỗ này là chỗ sai.

Đọc AD log trước khi đề xuất đổi một quyết định lớn: phần lớn các lựa chọn nghe hợp lý đã
được cân nhắc và bác bỏ, kèm lý do.

**Trạng thái:** lõi Telex đã port xong và xanh trên cả ba engine. Chưa có shell nào cho
OS. Xem [ROADMAP.md](ROADMAP.md).

---

## Hai layer

| | Việc | Ai làm | |
|---|---|---|---|
| **Layer 1** | phím vật lý → ký tự Dvorak | **tầng layout của OS**, trừ Windows | [AD-4](docs/architecture-decisions.md#ad-4) |
| **Layer 2** | ký tự Dvorak → tiếng Việt có dấu | `crates/vietdvorak-core` | — |

Bản AutoHotkey chỉ làm layer 2 và bắt người dùng tự bật layout *United States-Dvorak*.
Mục tiêu của bản Rust là **bỏ yêu cầu đó** — nhưng "bỏ yêu cầu" không có nghĩa là "tự
implement Dvorak"; xem [AD-4](docs/architecture-decisions.md#ad-4).

## Ba engine, một spec

Engine Telex tồn tại **ba bản** trên ba runtime không chia sẻ code được:

| | Là gì |
|---|---|
| `VietDvorak.ahk` | bộ gõ đang chạy trên Windows, AutoHotkey v2 |
| `practice.html` | trainer, mirror JS (khối giữa `// BEGIN TELEX ENGINE` và `// END TELEX ENGINE` — đúng hai marker `check-parity.mjs` cắt theo) |
| `crates/vietdvorak-core` | bản Rust, tương lai của dự án |

`node check-parity.mjs` là verdict duy nhất
([AD-1](docs/architecture-decisions.md#ad-1), [AD-2](docs/architecture-decisions.md#ad-2)).
Không phải section nào cũng áp dụng cho cả ba
([AD-3](docs/architecture-decisions.md#ad-3)).

## Shell theo nền tảng

| Nền tảng | Shell | Trạng thái | |
|---|---|---|---|
| Linux | IBus engine qua `zbus` | đang tới | [AD-5](docs/architecture-decisions.md#ad-5), [AD-6](docs/architecture-decisions.md#ad-6) |
| Windows | `WH_KEYBOARD_LL` + hybrid `SendInput` | hoãn | [AD-15](docs/architecture-decisions.md#ad-15) |
| macOS | InputMethodKit | hoãn, chưa có máy | [AD-16](docs/architecture-decisions.md#ad-16) |

Seam giữa lõi và shell là `View`, mang **hai** lowering
([AD-8](docs/architecture-decisions.md#ad-8)). Mô hình xuất chữ trên Linux/macOS là
**preedit**, có gạch chân, và đó là rủi ro sản phẩm cao nhất chưa được trả lời
([AD-7](docs/architecture-decisions.md#ad-7)).

---

## Cấu trúc

```
crates/vietdvorak-core/src/ lõi Telex — no_std, zero dependency (AD-10)
  tables.rs                 hai bảng gốc; các bảng khác suy ra từ đây
  vowel.rs                  nhận diện + dựng lại nguyên âm
  tone.rs                   đặt dấu thanh
  nucleus.rs                chuẩn hoá vần
  transform.rs              một bước Telex
  lib.rs                    Session, View, ends_syllable
crates/vdconform/src/       line protocol -> engine thứ ba cho check-parity.mjs

telex-conformance.txt       SPEC. Nguồn sự thật duy nhất (AD-1).
check-parity.mjs            harness duy nhất, verdict duy nhất (AD-2)
VietDvorak.ahk              bộ gõ đang chạy trên Windows — baseline mà bản port phải
                            khớp, TRỪ Backspace (AD-9)
practice.html               trainer + mirror JS của engine + bảng phím Dvorak

docs/architecture-decisions.md   AD log — LÝ DO sống ở đây
.github/workflows/ci.yml    cổng duy nhất — pin SHA256 AutoHotkey (AD-11), gate zero
                            dependency (AD-10), chặn engine bị skip
Cargo.toml, Cargo.lock      workspace; lint deny bật cho TOÀN workspace.
                            Lock ĐƯỢC commit, có chủ ý — xem .gitignore
rust-toolchain.toml         pin 1.95 + target musl; lý do `check` chứ không `build` (AD-12)
clippy.toml                 nới unwrap/expect/indexing trong test

corpus.txt                  ngữ liệu tiếng Việt cho bốn script dưới
viet-corpus.mjs             âm tiết + bố cục Dvorak gõ tay -> LEFT_HAND/RIGHT_HAND
tone-stats.mjs              tần suất dấu thanh
bigram-analysis.mjs         mô hình gõ dấu SỚM — kết luận NGƯỢC, giữ có chủ ý
bigram-late-tone.mjs        mô hình gõ dấu MUỘN — cơ sở của ngã=x / nặng=j
```

---

## Nợ đã biết

**Bảng phím Dvorak đang bị chép ở hai chỗ**, không phải một:

- `practice.html :: KEYBOARD_LAYOUT` — nhưng đây là bảng **hiển thị 47 phím + space**,
  chia theo hàng, giữ cả 14 điểm bất động, và trainer đọc nó ra `HINT_CANDIDATE_KEYS`,
  `HOME_KEYS`, bàn phím ảo.
- `viet-corpus.mjs` — gõ tay bố cục Dvorak theo hàng để suy ra `LEFT_HAND`/`RIGHT_HAND`,
  và chính bảng gán tay đó là **cơ sở của lựa chọn phím thanh đang ship** (ngã=x /
  nặng=j). Đây mới là bản copy mà drift đắt nhất.

`SHIFTED_KEYS` bên cạnh **KHÔNG** thuộc layer 1 — nó là tầng shift US thuần, giống hệt
trên QWERTY. Để yên.

Tách ra `data/dvorak-permutation.tsv` **không phải nửa giờ**: `practice.html` phải chạy
offline qua `file://` và không fetch được, nên nó vẫn cần một bản nhúng. Việc thật là dựng
đúng cơ chế mà `telex-conformance.txt` đã có — nguồn + khối SINH RA có marker + kiểm drift
trong `--sync-selftests` — và phải sinh bảng hiển thị **TỪ** hoán vị, giữ nguyên entry
space và các điểm bất động, nếu không hint sẽ tắt ở ranh giới âm tiết đúng như comment tại
chỗ cảnh báo. Ước lượng nửa buổi, chỉ đáng làm khi shell Linux cần đọc bảng đó.

**`MODIFIED` trong `tone.rs`** đúng bằng phần nguyên âm của `DIACRITIC_PAIRS` (bỏ `đ`) —
suy ra được, đang chép tay.

**`deny.toml` + kiểm binary không link network** — chưa làm vì chưa có dependency nào để
ban ([AD-10](docs/architecture-decisions.md#ad-10)).

---

## Chưa kiểm trên phần cứng thật

Mọi khẳng định về GNOME / mutter / GTK / IBus trong repo này đến từ nghiên cứu có dẫn
nguồn, **không phải từ quan sát trên máy Ubuntu**. Đủ tốt để thiết kế, không đủ tốt để
hardcode — luật đầy đủ ở [AD-14](docs/architecture-decisions.md#ad-14).

| Phải xác minh | Chặn cái gì |
|---|---|
| Keycode IBus (`a` → 30?) — **đã đảo chiều hai lần** | assert lúc bring-up, M6 |
| `<layout_variant>` có bị gnome-shell nuốt không | [AD-6](docs/architecture-decisions.md#ad-6) đang là `draft` vì câu này |
| Gạch chân có sống được không | [AD-7](docs/architecture-decisions.md#ad-7) — rủi ro cao nhất; ROADMAP M0.a |
| `ibus-daemon` chạy trong WSL2 tới mức nào | M6 |
