# Kiến trúc

Vì sao dự án có hình dạng này. Đọc file này trước khi đề xuất đổi một quyết định
lớn — phần lớn các lựa chọn "hợp lý" đã được cân nhắc và bác bỏ, và §7 ghi rõ
từng cái cùng lý do.

**Trạng thái:** lõi Telex đã port xong và xanh trên ba engine. Chưa có shell nào
cho OS. Xem [ROADMAP.md](ROADMAP.md).

---

## 1. Hai layer

| | Việc | Ai làm |
|---|---|---|
| **Layer 1** | phím vật lý → ký tự Dvorak | **tầng layout của OS**, trừ Windows |
| **Layer 2** | ký tự Dvorak → tiếng Việt có dấu | `crates/vietdvorak-core` |

Bản AutoHotkey chỉ làm layer 2 và bắt người dùng tự bật layout
*United States-Dvorak* của Windows. Mục tiêu của bản Rust là **bỏ yêu cầu đó**.

Nhưng "bỏ yêu cầu" không có nghĩa là "tự implement Dvorak". Xem §3.

---

## 2. `telex-conformance.txt` là spec, không phải code nào cả

Đây là quyết định gốc, mọi thứ khác treo vào nó.

Engine Telex hiện tồn tại **ba bản**, ba runtime không chia sẻ code được:

- `VietDvorak.ahk` — bộ gõ đang chạy trên Windows, AutoHotkey v2
- `practice.html` — trainer gõ phím, bản mirror JS (khối giữa hai marker
  `// BEGIN TELEX ENGINE` và `// END TELEX ENGINE` — đúng hai marker mà
  `check-parity.mjs` cắt theo)
- `crates/vietdvorak-core` — bản Rust, tương lai của dự án

`node check-parity.mjs` chạy mọi vector qua mọi engine mà section đó là spec
cho, **trong một lần chạy**, và exit khác 0 nếu bất kỳ bên nào lệch. Đó là lý do
tồn tại của cả cấu trúc này: ba bản implement không thể trôi khỏi nhau mà không
ai biết.

**Hệ quả cứng:** không được thêm parser thứ hai hay verdict thứ hai cho spec.
`vdconform` cố ý không đọc `telex-conformance.txt` và không bao giờ quyết định
pass/fail — nó tính và in, `check-parity.mjs` một mình phán xét. Một
`tests/conformance.rs` tự parse file `.txt` sẽ là parser thứ hai, và hai thứ đó
sẽ trôi khỏi nhau; khi chúng bất đồng thì không ai biết bên nào đúng.

**Không phải section nào cũng áp dụng cho cả ba engine**, và điều đó được ghi rõ
theo từng section thay vì mặc định. Xem `@section backspace` trong file spec:
AHK truncate còn Rust replay, và đó là khác biệt kiến trúc chứ không phải một
bên sai.

---

## 3. Layer 1 thuộc về OS, không phải về chúng ta

### Phát hiện làm mọi thứ rẻ đi

US-Dvorak là một **hoán vị song ánh chính xác** của US-QWERTY trên 47 phím khối
chính: **14 điểm bất động, 33 phím đổi chỗ, zero sai lệch ở tầng shift**. Đúng y
hệt trên cả ba nền tảng:

| Nền tảng | Kiểm bằng |
|---|---|
| Windows | đối chiếu `KBDDV.DLL` |
| Linux | `xkeyboard-config` `us(dvorak)` |
| macOS | dump `.keylayout`, **4 tầng** kể cả cờ dead-key |

Bài test macOS khó hơn Windows: US trên Windows không có tầng AltGr nào, còn US
trên macOS có tầng Option sống với 5 dead key — và tầng Option của Dvorak đi
theo *ký tự* chứ không theo *vị trí*. Cả 5 dead key di trú đúng.

→ Layer 1 là **một bảng 33 entry**, không phải bảng ký tự.

### Nhưng cơ chế chỉ miễn phí trên Windows

`WH_KEYBOARD_LL` nằm **thượng nguồn** tầng dịch layout *và* thượng nguồn dispatch
phím tắt. Đổi scancode rồi phát lại → shift, CapsLock, AltGr, dead key và Ctrl+C
rơi ra miễn phí từ OS. Bữa trưa miễn phí này **không tồn tại ở đâu khác**:

- **Linux** — engine IBus nằm *hạ nguồn* tầng layout. Hoán vị keycode rồi trả
  `false` cho phím không xử lý → IBus đẩy phím **gốc** đi, app nhận QWERTY.
- **macOS trong IME** — Cocoa dispatch `performKeyEquivalent:` **trước**
  `keyDown:`, nên input method không bao giờ thấy ⌘-chord một cách đáng tin. Và
  phím passthrough (chế độ EN) không đi qua IME nên không được hoán vị.
- **macOS trong CGEventTap** — phải tự tính ký tự bằng `UCKeyTranslate` và **sở
  hữu ngữ nghĩa dead-key của Apple vĩnh viễn** — đúng thứ mẹo Windows tránh được.

**Phát biểu lại cho gọn: layer 1 thuộc tầng layout của OS trên cả ba nền tảng;
Windows là ngoại lệ duy nhất, và chỉ vì hook nằm thượng nguồn.** Điều này làm
thiết kế *đối xứng hơn* chứ không phải "macOS đặc biệt".

### Khẩu hiệu phải diễn giải lại

*"App tự sở hữu layer 1"* là **sai** ở 2/3 nền tảng: ta chỉ sở hữu việc **CHỌN**
một layout của OS. Theo single-source-of-truth thì đây mới là đúng — dữ liệu
Dvorak của Microsoft, Apple và xkeyboard-config *đều đã tồn tại* và *đều đã được
chứng minh là cùng một hoán vị 14/33*, nên tự cài lại ở runtime là **sao chép**.

Hoán vị 33 entry vẫn cần một nhà riêng, và nó đang bị chép ở **hai** chỗ, không
phải một:

- `practice.html :: KEYBOARD_LAYOUT` — nhưng đây là bảng **hiển thị 47 phím +
  space**, chia theo hàng, giữ cả 14 điểm bất động, và trainer đọc nó ra
  `HINT_CANDIDATE_KEYS`, `HOME_KEYS` và bàn phím ảo.
- `viet-corpus.mjs` — gõ tay bố cục Dvorak theo hàng để suy ra
  `LEFT_HAND`/`RIGHT_HAND`, và chính bảng gán tay đó là **cơ sở của lựa chọn
  phím thanh đang ship** (ngã=x / nặng=j). Đây mới là bản copy mà drift đắt
  nhất.

`SHIFTED_KEYS` bên cạnh **KHÔNG** thuộc layer 1: nó là tầng shift US thuần,
giống hệt trên QWERTY — đúng như "zero sai lệch ở tầng shift" ở trên. Để yên.

Và tách ra `data/dvorak-permutation.tsv` **không phải nửa giờ**: `practice.html`
phải chạy offline qua `file://` và không fetch được, nên nó vẫn cần một bản
nhúng. Việc thật là dựng đúng cơ chế mà `telex-conformance.txt` đã có — nguồn +
khối SINH RA có marker + kiểm drift trong `check-parity.mjs --sync-selftests` —
và phải sinh bảng hiển thị TỪ hoán vị, giữ nguyên entry space và các điểm bất
động, nếu không hint sẽ tắt ở ranh giới âm tiết đúng như comment tại chỗ cảnh
báo. Là nợ SSOT có thật, ước lượng nửa buổi, chỉ đáng làm khi shell Linux cần
đọc bảng đó.

---

## 4. Linux: IBus engine, và vì sao không phải evdev

**Chọn:** IBus engine qua `zbus`, thuần Rust.

**Bác bỏ — và đây là khẳng định đã bị phản biện bác trực diện:**
*"đọc raw qua evdev để né Wayland + ghi qua `commit_text` để né layout"*.

Nó là hai đường input đánh nhau:

- **Không `EVIOCGRAB`** — kernel giao event cho cả reader của ta lẫn compositor →
  app nhận phím *và* nhận text ta commit → **gõ đúp**.
- **Có `EVIOCGRAB`** — compositor không bao giờ thấy phím → IME framework không
  có gì để xử lý → ta đã thành một keyboard driver hoàn chỉnh và không còn cần
  `commit_text` nữa.

Nó cũng **thừa**: IBus đã đưa hardware keycode ngay trong
`process_key_event(engine, keyval, keycode, state)`. Và điểm chí mạng độc lập:
**`uinput` không inject được Unicode** — nó chỉ mang `KEY_*`, rồi compositor áp
XKB layout *của người dùng* lên đó, nên không có đường nào ra `ế`.

**Lợi thế bảo mật đi kèm, đáng nêu thẳng:** IBus engine chạy như user thường —
không root, không cần group `input`, không udev rule. Đường evdev đòi đưa user
vào group `input`, tức trao quyền keylog toàn hệ thống cho *mọi* process của user
đó. Packager sẽ đúng khi từ chối.

**Chọn IBus thay vì fcitx5** dù fcitx5 tốt hơn về kỹ thuật: fcitx5 là C++ addon
không có C ABI ổn định → cần `cxx` bridge + toolchain C++ trong CI. IBus tốn
đúng một D-Bus interface và là mặc định trên Ubuntu/Fedora GNOME. Một người hỗ
trợ tốt một framework, hoặc hỗ trợ tệ hai cái. *Rủi ro đã biết:* user dùng fcitx5
sẽ không thấy engine. Phải quyết định tường minh ở M7, đừng để biết qua bug
report.

### Cơ chế layer 1 trên Ubuntu

Hai cách hiển nhiên **đều hỏng, và hỏng im lặng**:

- `<layout>us(dvorak)</layout>` — IBus parse `layout` là string thô, **không
  parse ngoặc**. gnome-shell tra hash key `us+dvorak`, miss, rồi `if (!info)
  return;`.
- `<layout>us</layout>` + `<layout_variant>dvorak</layout_variant>` —
  gnome-shell đọc `engineDesc.variant`, nhưng property thật tên là
  `layout-variant`. Biểu thức trả `undefined`, variant rơi mất, session nhận US
  QWERTY thuần.

Cả hai cho **cùng một triệu chứng**: engine chạy, layout là QWERTY, không error
không log. Người dùng sẽ báo "engine Telex hỏng" và ta sẽ đi debug FSM.

**Cách đúng khi đóng gói (v1 = M9, chỉ khi có người dùng thứ hai):** ship một XKB layout tên riêng `vietdvorak` mà thân là
`include "us(dvorak)"`, khai `<layout>vietdvorak</layout>` với variant rỗng.
Đăng ký vào `evdev.xml`, **không phải** `evdev.extras.xml`.

**Cách dùng ở v0:** `<layout>default</layout>`, người dùng tự thêm hai input
source. Zero packaging, zero rủi ro — cơ chế forcing còn có bug không có đường
restore (ibus-bamboo issue #145).

---

## 5. Preedit, và cái gạch chân

IBus (và InputMethodKit sau này) dùng mô hình **preedit**: âm tiết đang gõ hiện ra dưới
dạng text tạm, có **gạch chân**, rồi mới chốt. Bản AHK dùng mô hình
**injection**: gõ thẳng vào app, không có gạch chân.

**Trên Ubuntu 26.04 gạch chân KHÔNG tắt được.** Protocol IBus có
`IBUS_ATTR_UNDERLINE_NONE` và nó chạy thật — *trên Xorg*. Nhưng 26.04 là bản LTS
**đầu tiên không còn GNOME X.org session**, và trên Wayland có ba tầng độc lập
cùng ép gạch chân, tầng cuối là libibus tự convert `UNDERLINE` →
`HINT = PREEDIT_WHOLE` và **bỏ qua value**.

Đây là **rủi ro sản phẩm cao nhất của dự án**, và nó chưa được trả lời. Xem
**hai** cổng chặn M0 trong [ROADMAP.md](ROADMAP.md) — M0.a (gạch chân, 20 phút)
là cái liên quan tới mục này, M0.b thêm 5 phút. Tổng 25 phút, zero dòng code.

### Seam `View`

`View` mang **hai biểu diễn**, để một lõi phục vụ cả hai mô hình:

```rust
pub struct View {
    pub composing: String,  // preedit  -> UpdatePreeditText
    pub committed: String,  // đã chốt  -> CommitText
    pub consumed: bool,
}
```

`committed` là field **riêng biệt** khỏi `composing`, và ba mệnh đề mà shell
injection phải theo được ghi trong doc comment của `View`. Mệnh đề (3) là thứ
chắc chắn sẽ làm sai: `composing:"mùa"` rồi Space → `composing:"" committed:"mùa
"`; một diff ngây thơ thấy composing co 3→0 và phát 3 backspace, **phá text vừa
commit**.

Hiện chỉ có nhánh preedit sẽ có consumer. Nhánh injection **chưa được xây** — một
seam chỉ có một consumer là seam chưa được kiểm chứng, và
`VietDvorak.ahk :: SyncToScreen()` đã là một bản implement injection chạy được
để port sau.

---

## 6. Backspace: replay, không phải truncate

Đây là chỗ ba bản implement **không phải ba phiên bản của một hàm**.

`VietDvorak.ahk` cắt buffer và **phải** làm thế: dưới injection, `g_buffer` CHÍNH
LÀ mô hình của text mà app sở hữu. Hệ quả là một **defect thật đang ship**: bất
biến "tối đa một dấu, đúng tại `FindToneTarget`" bị phá, và `muafn` ⌫ `f` ra
`mùà` — hai dấu, một chuỗi tiếng Việt không biểu đạt được.

Dưới preedit, IME sở hữu vùng đang gõ tuyệt đối nên `Session` giữ được **lịch sử
phím thô** và replay từ rỗng. Không có trạng thái nào để hỏng, nên hai dấu là
điều **không thể đạt tới** chứ không phải điều được canh chừng.

Luật *"pop tới khi TEXT đổi"* lấy từ `undoKey()` trong `practice.html`: phím bị
**nuốt** vẫn phải ghi log (chữ `w` thừa của `dduwowngf` không sinh text nhưng là
một keystroke), nhưng nếu Backspace pop đúng một phím thì lần bấm ấy **tỏ ra
không làm gì cả**.

**Khác `practice.html` có chủ ý:** log của trainer trải trên cả text đã chốt, nên
undo bên đó pop được dấu cách và kéo text đã commit trở lại buffer. IME không thể
và không nên — text đã chốt thuộc về app.

**Bản AHK chưa được sửa, và đó là quyết định.** Fix replay bên đó đòi
`SyncToScreen` phát ra replacement không rút ngắn text, tức công Windows thật
trên code sắp nghỉ hưu — và làm trước sẽ dịch chuyển baseline mà bản port đang cố
khớp. Nếu bug cắn thật khi dùng hàng ngày, backport thiết kế replay *đã validate
bằng Rust* ngược sang AHK. Đừng phát minh ngữ nghĩa hai lần.

---

## 7. Đã bác bỏ, và vì sao

Ghi lại để không ai đề xuất lại — mấy cái này đều nghe hợp lý.

| Ý | Vì sao bác |
|---|---|
| evdev + uinput trên Linux | Hai đường input đánh nhau; `uinput` không inject được Unicode. §4 |
| evdev đọc + IBus `commit_text` ghi | Không mạch lạc ở cả hai chiều. §4 |
| Tự hardcode bảng scancode→ký tự | Dữ liệu đã tồn tại trong OS và đã chứng minh là cùng một hoán vị — tự cài lại là sao chép. §3 |
| Inject Unicode để làm layer 1 | `KEYEVENTF_UNICODE` **không tham gia tổ hợp modifier** — Ctrl+\<unicode\> không ra gì |
| TSF text service trên Windows **cho v1** | KHÔNG phải bác bỏ vĩnh viễn mà là **hoãn**: +~3.000 dòng chỉ-Windows, DLL nạp vào *mọi* process gõ chữ, cần admin, đổi hành vi cho user hiện hữu. Là impl bổ sung khi hook tỏ ra không đủ |
| macOS CGEventTap | Cần quyền Accessibility; chết thầm sau re-sign; đuôi workaround per-app là *chính công việc*. InputMethodKit cần zero quyền TCC |
| `tests/conformance.rs` tự parse spec | Parser thứ hai + verdict thứ hai cho một spec. §2 |
| `cargo build` cho gate Linux | Cần linker ELF → `.cargo/config.toml` chứa đường dẫn tuyệt đối có cả username lẫn tên toolchain. `cargo check` không link |
| Cache cargo trong CI | Zero dependency, build ~4 giây — máy móc bảo vệ hư không |
| `deny.toml` **ngay bây giờ** | KHÔNG phải bác bỏ mà là **hoãn**: chưa có dependency nào để ban, `cargo tree` trong CI đã kiểm điều tương đương. Đã lên lịch cho M6 cùng `zbus` — xem §8 |

---

## 8. Lập trường bảo mật

Chương trình này **theo năng lực là một keylogger** — nó đọc mọi phím gõ vào mọi
text field. Lời hứa không kiểm chứng được thì vô giá trị, nên hai khẳng định dưới
đây được **CI enforce**, không phải ghi trong README:

- **Lõi zero dependency.** Mọi crate trên đường đi của phím đọc được mọi thứ
  người dùng gõ, nên đây là thuộc tính *bảo mật*, không phải thẩm mỹ. Job `spec`
  chạy `cargo tree` và fail nếu có bất kỳ dependency nào.
- **AutoHotkey pin theo SHA256.** CI *thực thi* binary bên thứ ba này; "tải bản
  mới nhất rồi chạy" là bề mặt cung ứng không tương xứng với phần còn lại.

Khi `zbus` vào ở M6, thêm `deny.toml` ban toàn bộ crate network và một bước CI
assert binary không link `libcurl` / không có syscall `connect`. Không network,
không telemetry, không update check — và enforce theo cách verify được.

Chưa tách `THREAT-MODEL.md` riêng: sẽ tách khi thật sự ship cho người thứ hai.

---

## 9. Những điều CHƯA được kiểm trên phần cứng thật

Mọi khẳng định về GNOME/mutter/GTK/IBus trong file này đến từ nghiên cứu có dẫn
nguồn, **không phải từ quan sát trên máy Ubuntu**. Chúng đủ tốt để thiết kế,
không đủ tốt để hardcode.

**Một dữ kiện đã đảo chiều hai lần và tuyệt đối không được tin theo tài liệu:**

> IBus giao keycode theo hệ nào?
> - Vòng 1: set-1 == evdev, không cần bảng dịch.
> - Vòng 2: sai — IBus giao X11 = evdev + 8, nên `KEY_A = 38`.
> - Vòng 3: vòng 2 bị ngược — IBus giao **evdev**, `KEY_A = 30`; ba client path
>   độc lập đều trừ 8 trước khi gọi, và chiều ngược lại cộng 8 khi forward.
>
> Bằng chứng vòng 3 mạnh hơn và nhất quán hai chiều, nên thiết kế theo nó.
> **Nhưng phải pin bằng một assert lúc bring-up: bấm `a`, keycode phải về 30.**

Danh sách phải xác minh trước khi khoá thiết kế shell:

- keycode của IBus (trên) — assert lúc bring-up
- `<layout_variant>` có thật sự bị gnome-shell nuốt không — thí nghiệm 5 phút,
  có thể lật đổ **tiểu mục cơ chế layer 1 của §4** và bỏ luôn nhu cầu named XKB
  layout
- gạch chân có tắt được không — đã kết luận là **không**, nhưng chưa ai gõ thử
- `ibus-daemon` có chạy trong WSL2 không, và tới mức nào

---

## 10. Cấu trúc

```
crates/vietdvorak-core/src/ lõi Telex — no_std, zero dependency
  tables.rs                 hai bảng gốc; mọi bảng khác suy ra từ đây
  vowel.rs                  nhận diện + dựng lại nguyên âm
  tone.rs                   đặt dấu thanh
  nucleus.rs                chuẩn hoá vần
  transform.rs              một bước Telex
  lib.rs                    Session, View, ends_syllable
crates/vdconform/src/       line protocol -> engine thứ ba cho check-parity.mjs

telex-conformance.txt       SPEC. Nguồn sự thật duy nhất.
check-parity.mjs            harness duy nhất, verdict duy nhất
VietDvorak.ahk              bộ gõ đang chạy trên Windows — baseline mà bản port
                            phải khớp, TRỪ Backspace (§6; spec nói rõ AHK không
                            làm oracle được ở đó)
practice.html               trainer + mirror JS của engine + bảng phím Dvorak

.github/workflows/ci.yml    cổng duy nhất — pin SHA256 AutoHotkey, gate zero
                            dependency, chặn engine bị skip
Cargo.toml, Cargo.lock      workspace; lint deny bật cho TOÀN workspace (§8).
                            Lock ĐƯỢC commit, có chủ ý — xem .gitignore
rust-toolchain.toml         pin 1.95 + target musl; lý do `check` chứ không
                            `build` (§7) sống ở đây
clippy.toml                 nới unwrap/expect/indexing trong test

corpus.txt                  ngữ liệu tiếng Việt cho bốn script dưới
viet-corpus.mjs             âm tiết + bố cục Dvorak gõ tay -> LEFT_HAND/RIGHT_HAND
tone-stats.mjs              tần suất dấu thanh
bigram-analysis.mjs         mô hình gõ dấu SỚM — kết luận NGƯỢC, giữ có chủ ý
bigram-late-tone.mjs        mô hình gõ dấu MUỘN — cơ sở của ngã=x / nặng=j
```
