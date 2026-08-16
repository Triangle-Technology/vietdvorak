# Architecture Decisions (AD log) — VietDvorak

> **Append-only.** Mỗi quyết định kiến trúc lớn là một entry. Không sửa entry cũ; thay
> đổi một quyết định ⇒ **thêm entry mới**, và ghi `superseded by AD-x` vào entry cũ.
>
> **Vì sao append-only, và vì sao đánh số:** `AD-7` là một mỏ neo **vĩnh viễn**. Code,
> commit message và tài liệu khác trỏ vào nó thì con trỏ không bao giờ hỏng. Trước file
> này, lý do kiến trúc sống trong `ARCHITECTURE.md` dưới dạng `§6`, `§9` — mà **thêm một
> mục vào giữa file là mọi con trỏ sau nó lệch, im lặng**. Đúng cùng lớp lỗi với số dòng,
> thứ repo này đã tự cấm hai lần (`viet-corpus.mjs`: *"cite the SYMBOL, never a line
> range"*; `check-parity.mjs`: *"never line numbers, which go stale silently"*) và vẫn
> tái phạm ở lượt viết tài liệu đầu tiên.
>
> **Trạng thái entry:**
> - `draft` — đang viết, **sửa tự do**, kể cả xoá cả section. Dùng khi công việc mà entry
>   mô tả CHƯA xong. Số AD cố định ngay từ đầu nên con trỏ trỏ vào nó vẫn hợp lệ.
> - `locked` — ảnh chụp cố định. Từ đây không sửa một ký tự nào; đổi ⇒ AD mới + con trỏ.
> - Chỉ chuyển `draft` → `locked` ở commit **cuối** của công việc mà entry mô tả.
>
> Cơ chế `draft`/`locked` lấy từ [nightly](../../nightly/docs/architecture-decisions.md),
> nơi nó sinh ra vì một nguyên nhân đo được: khi mọi entry `locked` ngay lúc ra đời, một
> công việc trải nhiều commit **không có cách nào hợp lệ để ghi AD**.
>
> **Ranh giới với các file khác:** *lý do* sống ở đây và chỉ ở đây.
> [PRINCIPLES.md](../PRINCIPLES.md) giữ **tiên đề** — điều đúng về tiếng Việt và về sản
> phẩm, độc lập với code. AD **phái sinh từ** tiên đề và trỏ ngược lên khi cần; tiên đề
> KHÔNG được chép xuống đây.
> [ARCHITECTURE.md](../ARCHITECTURE.md) là bản đồ định hướng (hệ thống là gì, file ở đâu,
> cái gì chưa kiểm) và **trỏ** vào đây. [CLAUDE.md](../CLAUDE.md) là bất biến khi sửa
> repo. [ROADMAP.md](../ROADMAP.md) là trạng thái. Thấy một lý do bị chép ở hai chỗ →
> chỗ không phải file này là chỗ sai.
>
> **Đừng chốt dải số AD** (`AD-1…AD-16`) ở bất kỳ file ra lệnh nào. Dải cứng luôn bỏ sót
> AD mới nhất — tức đúng phần vừa viết. Bảo *"đọc file này"*.

---

## AD-1 — `telex-conformance.txt` là spec; một parser, một verdict
**Ngày**: 2026-08-13 (đã sống trong code từ baseline `de52017`; ghi lại 2026-08-16) · **Trạng thái**: locked

**Bối cảnh**: engine Telex tồn tại nhiều bản trên nhiều runtime không chia sẻ code được.
Không có bản nào có tư cách làm chuẩn cho bản khác.

**Quyết định**: spec là một file dữ liệu, không phải code nào cả. `check-parity.mjs` là
chỗ **duy nhất** đọc nó và chỗ **duy nhất** quyết định pass/fail.

**Hệ quả**: `vdconform` cố ý không đọc file spec và không bao giờ phán xét — nó tính và
in. Một `tests/conformance.rs` tự parse file `.txt` là **parser thứ hai và verdict thứ
hai** cho một spec; hai thứ đó sẽ trôi khỏi nhau, và khi bất đồng thì không ai biết bên
nào đúng.

**Ranh giới**: test trong crate được phép pin những bất biến mà một vector **không phát
biểu được** — cờ `consumed`, "một âm tiết không bao giờ mang hai dấu". Đó không phải nhân
bản spec.

---

## AD-2 — Oracle vi sai: mọi engine chạy trong MỘT lần, không phải nhiều suite
**Ngày**: 2026-08-13 (JS+AHK); mở rộng cho Rust 2026-08-16 · **Trạng thái**: locked

**Quyết định**: `node check-parity.mjs` chạy mọi vector qua mọi engine mà section đó là
spec cho, **trong một lần chạy**, và exit khác 0 nếu bất kỳ bên nào lệch.

**Vì sao một lần chạy**: giá trị nằm ở chỗ các engine được so **với nhau**, không chỉ với
spec. Hai engine cùng sai một kiểu là bug spec; sai khác kiểu là drift. Tách ra nhiều job
CI là vứt bỏ đúng thứ nó đo — xem `.github/workflows/ci.yml`, job `spec` cố ý không tách.

**Hệ quả**: `--skip-ahk` / `--skip-rust` mua được một lần chạy yên tĩnh hơn, **không** mua
được một lần chạy xanh. Exit code là thứ duy nhất CI đọc.

---

## AD-3 — Phạm vi áp dụng khai theo từng section, không mặc định "tất cả"
**Ngày**: 2026-08-16 · **Trạng thái**: locked

**Bối cảnh**: Backspace là chỗ các bản implement **không phải nhiều phiên bản của một
hàm** — xem [AD-9](#ad-9).

**Quyết định**: mỗi section trong spec khai rõ nó là spec CHO engine nào.
`SECTION_ENGINES` trong `check-parity.mjs` và phần văn xuôi trong header spec nói **cùng
một điều**; đổi một bên phải đổi bên kia.

**Vì sao không mặc định tất cả**: một section áp dụng cho mọi engine buộc phải viết theo
**engine yếu nhất**, và hành vi mạnh nhất khi đó **không có spec nào cả**.

**Hệ quả**: mỗi engine được chấm chỉ trên phần nó được hỏi. Không có cái này thì
`@section backspace` đọc thành 15 lỗi AHK — đúng nghĩa ngược.

---

## AD-4 — Layer 1 thuộc tầng layout của OS, không thuộc về chương trình này
**Ngày**: 2026-08-16 · **Trạng thái**: locked

**Bối cảnh**: mục tiêu của bản Rust là bỏ **yêu cầu** người dùng tự bật layout US-Dvorak.
Cám dỗ tự nhiên là tự implement Dvorak.

**Phát hiện làm mọi thứ rẻ đi**: US-Dvorak là **hoán vị song ánh chính xác** của
US-QWERTY trên 47 phím khối chính — **14 điểm bất động, 33 phím đổi chỗ, zero sai lệch ở
tầng shift**. Đúng y hệt trên Windows (`KBDDV.DLL`), Linux (`xkeyboard-config`) và macOS
(dump `.keylayout` 4 tầng, kể cả cờ dead-key; bài test macOS khó hơn vì tầng Option của
Dvorak đi theo *ký tự* chứ không theo *vị trí*, và cả 5 dead key di trú đúng).

**Quyết định**: **không** tự cài lại bảng đó ở runtime. Dữ liệu Dvorak của Microsoft,
Apple và xkeyboard-config đều đã tồn tại và đã chứng minh là cùng một hoán vị — tự làm
lại là **sao chép**.

**Hệ quả — khẩu hiệu phải diễn giải lại**: *"app tự sở hữu layer 1"* **sai** ở 2/3 nền
tảng. Ta chỉ sở hữu việc **CHỌN** một layout của OS. Windows là ngoại lệ duy nhất, và chỉ
vì `WH_KEYBOARD_LL` nằm **thượng nguồn** tầng dịch layout — đổi scancode rồi phát lại thì
shift/CapsLock/AltGr/dead-key/Ctrl+C rơi ra miễn phí. Bữa trưa đó không tồn tại ở đâu
khác: engine IBus nằm *hạ nguồn* tầng layout; trên macOS, Cocoa dispatch
`performKeyEquivalent:` **trước** `keyDown:` nên IME không bao giờ thấy ⌘-chord đáng tin.

**Đã bác**: hardcode bảng scancode→ký tự (sao chép dữ liệu OS đã có); inject Unicode để
làm layer 1 (`KEYEVENTF_UNICODE` **không tham gia tổ hợp modifier** — Ctrl+\<unicode\>
không ra gì).

---

## AD-5 — Linux shell là IBus engine qua zbus, KHÔNG phải evdev + uinput
**Ngày**: 2026-08-16 · **Trạng thái**: locked

**Bối cảnh**: Wayland chặn hook bàn phím toàn cục. Đề xuất đầu tiên — và nó **sai** — là
*"đọc raw qua evdev để né Wayland, ghi qua `commit_text` để né layout"*.

**Vì sao đề xuất đó không mạch lạc** (đã cho phản biện bác trực diện):
- **Không `EVIOCGRAB`** — kernel giao event cho cả reader của ta lẫn compositor → app
  nhận phím *và* nhận text ta commit → **gõ đúp**.
- **Có `EVIOCGRAB`** — compositor không bao giờ thấy phím → IME framework không có gì để
  xử lý → ta đã thành keyboard driver hoàn chỉnh và không còn cần `commit_text`.

**Và nó thừa**: IBus đã đưa hardware keycode ngay trong
`process_key_event(engine, keyval, keycode, state)`.

**Điểm chí mạng độc lập**: **`uinput` không inject được Unicode** — nó chỉ mang `KEY_*`,
rồi compositor áp XKB layout *của người dùng* lên đó. Không có đường nào ra `ế`.

**Quyết định**: IBus engine qua `zbus`, thuần Rust.

**IBus thay vì fcitx5** dù fcitx5 tốt hơn về kỹ thuật: fcitx5 là C++ addon không có C ABI
ổn định → cần `cxx` bridge + toolchain C++ trong CI. IBus tốn đúng một D-Bus interface và
là mặc định trên Ubuntu/Fedora GNOME. Một người hỗ trợ tốt một framework, hoặc hỗ trợ tệ
hai cái. *Rủi ro đã biết:* user dùng fcitx5 sẽ không thấy engine — phải quyết định tường
minh trước khi đóng gói, đừng để biết qua bug report.

**Hệ quả bảo mật, đáng nêu thẳng**: IBus engine chạy như user thường — không root, không
cần group `input`, không udev rule. Đường evdev đòi đưa user vào group `input`, tức trao
quyền keylog toàn hệ thống cho *mọi* process của user đó. Packager sẽ **đúng** khi từ chối.

---

## AD-6 — Layer 1 trên Ubuntu: named XKB layout, không phải variant
**Ngày**: 2026-08-16 · **Trạng thái**: draft *(chưa quan sát trên máy Ubuntu thật — xem [AD-14](#ad-14))*

**Bối cảnh**: hai cách hiển nhiên đều hỏng, và hỏng **im lặng**:
- `<layout>us(dvorak)</layout>` — IBus parse `layout` là string thô, **không parse
  ngoặc**. gnome-shell tra hash key `us+dvorak`, miss, rồi `if (!info) return;`.
- `<layout>us</layout>` + `<layout_variant>dvorak</layout_variant>` — gnome-shell đọc
  `engineDesc.variant`, nhưng property thật tên là `layout-variant`. Biểu thức trả
  `undefined`, variant rơi mất, session nhận US QWERTY thuần.

Cả hai cho **cùng một triệu chứng**: engine chạy, layout là QWERTY, không error không log.
Người dùng sẽ báo *"engine Telex hỏng"* và ta sẽ đi debug FSM.

**Quyết định (khi đóng gói)**: ship một XKB layout tên riêng `vietdvorak` mà thân là
`include "us(dvorak)"`, khai `<layout>vietdvorak</layout>` với variant rỗng. Đăng ký vào
`evdev.xml`, **không phải** `evdev.extras.xml` (file đó bị gate sau gsettings
`show-all-sources`, mặc định `false`).

**Quyết định (trước đó)**: `<layout>default</layout>`, người dùng tự thêm hai input
source. Zero packaging, zero rủi ro — cơ chế forcing còn có bug không có đường restore
(ibus-bamboo issue #145).

**Nghi thức chẩn đoán, nên giữ mãi**: trước mỗi phiên gõ chạy `wev`, bấm phím vật lý `;`,
phải ra keysym `s`. Mọi lỗi layer-1 đều im lặng và trông y hệt bug engine.

**`draft` vì**: kết luận về `engineDesc.variant` là **suy luận từ source gnome-shell**, chưa
phải quan sát. Thí nghiệm 5 phút có thể lật nó — xem ROADMAP M0.b.

---

## AD-7 — Mô hình preedit, và gạch chân là bắt buộc trên Ubuntu 26.04
**Ngày**: 2026-08-16 · **Trạng thái**: locked

**Bối cảnh**: IBus dùng mô hình **preedit** — âm tiết đang gõ hiện dưới dạng text tạm, có
**gạch chân**, rồi mới chốt. Bản AHK dùng **injection**: gõ thẳng vào app, không gạch chân.

**Dữ kiện**: protocol IBus có `IBUS_ATTR_UNDERLINE_NONE` và nó chạy thật — **trên Xorg**.
Nhưng Ubuntu 26.04 là bản LTS **đầu tiên không còn GNOME X.org session**, và trên Wayland
có **ba tầng độc lập** cùng ép gạch chân; tầng cuối là libibus tự convert `UNDERLINE` →
`HINT = PREEDIT_WHOLE` và **bỏ qua value**.

**Quyết định**: thiết kế theo preedit, chấp nhận gạch chân.

**Đây là rủi ro sản phẩm cao nhất của dự án và nó CHƯA được trả lời.** Nó không phải câu
hỏi kỹ thuật mà là câu hỏi **sản phẩm**: sống được với gạch chân mọi ngày không. Trả lời
bằng 20 phút gõ thật với `ibus-unikey` trên live USB, zero dòng code — ROADMAP M0.a.

**Nếu câu trả lời là không**: cần đường non-preedit (`CommitText` +
`DeleteSurroundingText`). ibus-bamboo chứng minh nó chạy được; giá là sleep 30–50 ms
quanh mỗi fake backspace và một **ma trận tương thích per-app phải bảo trì đời đời**.

---

## AD-8 — `View` mang hai lowering; `committed` là field riêng biệt
**Ngày**: 2026-08-16 · **Trạng thái**: locked

**Quyết định**: `View { composing, committed, consumed }` — `committed` **riêng biệt**
khỏi `composing`, không phải một chuỗi duy nhất mà shell tự đoán.

**Ba mệnh đề mà shell injection PHẢI theo** (doc comment của `View` giữ bản gốc):
1. diff **chỉ** trên `composing`, so với `last_composing` của chính shell;
2. append `committed` **nguyên văn**, không diff;
3. `committed` khác rỗng → reset `last_composing` về rỗng **KHÔNG phát backspace**.

**Mệnh đề (3) là thứ chắc chắn sẽ làm sai**: `composing:"mùa"` rồi Space →
`composing:"" committed:"mùa "`. Một diff ngây thơ thấy composing co 3→0 và phát 3
backspace — **phá text vừa commit**.

**Hệ quả**: giá của cánh cửa Windows là ~20 dòng, trả trước. Xem [AD-13](#ad-13) về việc
vì sao nhánh injection **chưa** được xây.

---

## AD-9 — Backspace là replay, không phải truncate; defect AHK để nguyên có chủ ý
**Ngày**: 2026-08-16 · **Trạng thái**: locked

**Bối cảnh**: `VietDvorak.ahk` cắt buffer bằng `SubStr(g_buffer, 1, -1)` và **phải** làm
thế — dưới injection, `g_buffer` CHÍNH LÀ mô hình của text mà app sở hữu, không có gì
khác để cắt. Hệ quả là một **defect thật đang ship**: bất biến "tối đa một dấu, đúng tại
`FindToneTarget`" bị phá, và `muafn` ⌫ `f` ra `mùà` — hai dấu, một chuỗi tiếng Việt không
biểu đạt được. Tám vòng code review không bắt được, vì spec có 266 vector và **zero
vector backspace**.

**Quyết định**: dưới preedit, IME sở hữu vùng đang gõ tuyệt đối nên `Session` giữ **lịch
sử phím thô** và replay từ rỗng. Không có trạng thái nào để hỏng ⇒ hai dấu là điều
**không thể đạt tới**, không phải điều được canh chừng.

**Luật "pop tới khi TEXT đổi"** lấy từ `practice.html :: undoKey()`, nơi đã lập luận đầy
đủ: phím bị **nuốt** vẫn phải ghi log (chữ `w` thừa của `dduwowngf` không sinh text nhưng
là một keystroke), nhưng nếu Backspace pop đúng một phím thì lần bấm ấy **tỏ ra không làm
gì cả**.

**Khác `practice.html` có chủ ý**: log của trainer trải trên cả text đã chốt, nên undo bên
đó pop được dấu cách và kéo text đã commit trở lại buffer. IME không thể và không nên —
text đã chốt thuộc về app. Log rỗng ⇒ `consumed: false`.

**Bản AHK chưa được sửa, và đó là quyết định**: fix replay bên đó đòi `SyncToScreen` phát
ra replacement không rút ngắn text, tức công Windows thật trên code sắp nghỉ hưu — và làm
trước sẽ **dịch chuyển baseline mà bản port đang cố khớp**. Nếu bug cắn thật khi dùng hàng
ngày: backport thiết kế replay **đã validate bằng Rust** ngược sang AHK. Đừng phát minh
ngữ nghĩa hai lần.

---

## AD-10 — Lõi zero dependency là thuộc tính BẢO MẬT, và CI enforce nó
**Ngày**: 2026-08-16 · **Trạng thái**: locked

**Bối cảnh**: chương trình này **theo năng lực là một keylogger** — nó đọc mọi phím gõ vào
mọi text field. Lời hứa không kiểm chứng được thì vô giá trị.

**Quyết định**: `crates/vietdvorak-core` có **zero dependency**, và job `spec` chạy
`cargo tree` để fail nếu có bất kỳ dependency nào.

**Vì sao là bảo mật chứ không phải thẩm mỹ**: mọi crate trên đường đi của phím đọc được
mọi thứ người dùng gõ. Một thuộc tính bảo mật chỉ sống trong comment là thuộc tính sẽ biến
mất ở PR nào đó.

**Hệ quả kỹ thuật**: `no_std + alloc`, không `HashMap` (bảng chỉ vài chục phần tử, quét
tuyến tính vừa đủ). Shell được phép có dependency, tối thiểu, và mỗi cái phải trả lời được
*"crate này thấy được gì"*.

**Còn nợ**: `deny.toml` ban toàn bộ crate network + bước CI assert binary không link
`libcurl` / không có syscall `connect`. Chưa làm vì **chưa có dependency nào để ban** —
thêm cùng lúc `zbus` vào.

---

## AD-11 — AutoHotkey trong CI pin theo SHA256, không chỉ theo version
**Ngày**: 2026-08-16 · **Trạng thái**: locked

**Quyết định**: bước cài AutoHotkey trong CI pin cả version lẫn SHA256 của zip.

**Vì sao khác mọi thứ còn lại**: đây là binary bên thứ ba mà CI đem **thực thi**. Với một
dự án mà sản phẩm đọc mọi phím người dùng gõ, *"tải bản mới nhất rồi chạy"* là bề mặt cung
ứng không tương xứng với phần còn lại. Nâng version là hành động **có chủ ý** và phải thấy
được trong diff: đổi cả hai dòng.

**Hệ quả**: pin đúng version chủ dự án đang chạy trên máy, để CI và máy dev thực thi cùng
một engine — nếu khác nhau thì *"xanh trên CI, đỏ ở nhà"* thành khả năng, và đó là loại
lỗi đắt nhất repo này có thể có.

---

## AD-12 — Gate Linux là `cargo check`, không phải `cargo build`
**Ngày**: 2026-08-16 · **Trạng thái**: locked

**Quyết định**: `cargo check --target x86_64-unknown-linux-musl`.

**Vì sao không `build`**: build cần linker ELF, tức phải trỏ `.cargo/config.toml` vào
`rust-lld` bằng **đường dẫn tuyệt đối chứa cả username lẫn tên toolchain** — hỏng ngay khi
đổi máy hoặc lên phiên bản. `check` không link.

**Gate này trả lời đúng một câu**: crate còn compile cho Linux không. Binary thật sẽ build
**ngay trên Ubuntu** khi có máy — không có nhu cầu nào cần một ELF sinh ra từ Windows, nên
không dựng máy móc cho nó.

---

## AD-13 — Không xây seam cho consumer chưa tồn tại
**Ngày**: 2026-08-16 · **Trạng thái**: locked

**Quyết định**: nhánh injection của [`View`](#ad-8) **không được xây** dù `View.committed`
đã có chỗ cho nó.

**Lý do**: một seam chỉ có một consumer là seam **chưa được kiểm chứng**, và nó thường sai
khi consumer thứ hai đến. Nhưng một seam xây sẵn cho consumer **vắng mặt** cũng thường
sai. Ở đây cái thứ hai rẻ hơn để hoãn, vì `VietDvorak.ahk :: SyncToScreen()` **đã là** một
bản implement injection chạy được để port khi có shell Windows thật.

**Áp cùng lý do cho**: `deny.toml` (chưa có dependency để ban — [AD-10](#ad-10)),
`ACCEPTANCE.md` (chưa có shell để test thủ công), file config cho
`TONE_PLACEMENT_MODERN` (đừng thêm serde+toml+watcher cho một boolean).

---

## AD-14 — Dữ kiện nền tảng phải pin bằng assert lúc bring-up, không bằng tài liệu
**Ngày**: 2026-08-16 · **Trạng thái**: locked

**Bối cảnh**: câu hỏi *"IBus giao keycode theo hệ nào"* đã cho **ba câu trả lời khác nhau
qua ba vòng nghiên cứu**:

| Vòng | Kết luận |
|---|---|
| 1 | set-1 == evdev, không cần bảng dịch |
| 2 | sai — IBus giao X11 = evdev + 8, nên `KEY_A = 38` |
| 3 | vòng 2 bị **ngược** — IBus giao evdev, `KEY_A = 30`; ba client path độc lập đều trừ 8 trước khi gọi, chiều ngược lại cộng 8 khi forward |

**Quyết định**: thiết kế theo vòng 3 (bằng chứng mạnh hơn và nhất quán hai chiều), nhưng
**không hardcode dựa trên đọc tài liệu**. Pin bằng một assert lúc bring-up: bấm `a`,
keycode phải về 30.

**Luật rút ra**: một dữ kiện đã đảo chiều thì tài liệu **không** còn là nguồn đủ tin. Mọi
khẳng định về GNOME/mutter/GTK/IBus trong repo này đến từ nghiên cứu có dẫn nguồn, **chưa
từng quan sát trên máy Ubuntu thật** — đủ tốt để thiết kế, không đủ tốt để hardcode.

---

## AD-15 — Windows shell (hoãn): hook + hybrid SendInput, không phải TSF
**Ngày**: 2026-08-16 · **Trạng thái**: draft *(chưa xây)*

**Quyết định khi tới lượt**: `WH_KEYBOARD_LL` + `SendInput` với emitter **lai** — scancode
thật cho ASCII, `KEYEVENTF_UNICODE` chỉ cho ~134 ký tự tiếng Việt có dấu. Điều đó xoá phần
lớn danh sách lỗi cổ điển của VK_PACKET (game, RDP, console VM) cho 90%+ keystroke.

**TSF text service KHÔNG bị bác vĩnh viễn — bị hoãn**: +~3.000 dòng chỉ-Windows, DLL nạp
vào *mọi* process gõ chữ trên máy (một panic hạ gục Word), cần admin, và **đổi hành vi cho
user hiện hữu**. Là impl bổ sung khi hook tỏ ra không đủ (UIPI / app elevated), không phải
điểm khởi đầu.

**Ghi chú**: bản AHK **đã có sẵn** đúng những giới hạn này hôm nay, nên ship hook là
non-regression chứ không phải thụt lùi.

---

## AD-16 — macOS (hoãn): InputMethodKit, không phải CGEventTap
**Ngày**: 2026-08-16 · **Trạng thái**: draft *(chưa xây, chưa có máy Mac)*

**Quyết định khi tới lượt**: InputMethodKit.

**Vì sao không CGEventTap**: cần quyền Accessibility (IMKit cần **zero** quyền TCC); chết
thầm sau khi re-sign — `CGEventTapCreate` trả handle non-nil, `CGEventTapIsEnabled()` báo
true, callback không bao giờ bắn; và cái đuôi workaround per-app là **chính công việc**,
không phải phần phụ.

**Điều kiện vật chất, không thương lượng**: cần một **máy Mac vật lý**. Build/sign/notarize
làm được từ Windows; **test hành vi thì không** — và cái bẫy quyết định là VNC/remote
desktop inject event ở tầng Quartz, *hạ nguồn* tầng layout, nên một `.keylayout` có thể
trông như không chạy trong khi thực ra nó chạy.

**Cửa đã giữ sẵn**: lõi zero dependency, không type nền tảng, constructor đặt tên theo
platform. Apple virtual keycode là hệ đánh số **thứ ba** (`kVK_ANSI_A = 0x00`), không có
quan hệ số học với set-1 hay evdev — nên `PhysicalKey` phải có field private
([AD-14](#ad-14) là lý do rộng hơn).

---

## AD-17 — Bộ vector phải được CHỨNG MINH có răng, không được giả định
**Ngày**: 2026-08-16 · **Trạng thái**: locked

**Bối cảnh**: 281 vector xanh chỉ chứng minh *"engine khớp spec ở những chỗ spec nói
tới"*. Nó **không** chứng minh spec sẽ đỏ khi engine sai. Cho tới AD này, chưa ai hỏi câu
thứ hai.

**Quyết định**: dùng [`scripts/dot-bien.sh`](../scripts/dot-bien.sh) — áp một đột biến vào
source, chạy cổng kiểm, hoàn nguyên byte-đúng kể cả khi Ctrl-C.

**🔴 Dán nhãn "mutation-proven" mà chưa CHẠY đột biến là một lời nói dối.** (Câu này lấy
nguyên từ [nightly](../../nightly/CLAUDE.md), nơi nó đã được trả giá.)

### Bốn luật đọc kết quả

1. **ĐỎ chỉ chứng minh "có thứ gì đó bắt được đột biến NÀY".** Đọc TÊN thứ vừa đỏ. Đột
   biến làm crash cả binary thì mọi thứ đỏ và nó không nói gì về bộ vector.
2. **Đột biến phải giữ chương trình COMPILE ĐƯỢC.** Lỗi biên dịch không phải "bộ test bắt
   được"; nó là "chưa bài nào chạy".
3. **"0 bài chạy" không phải "không có răng".**
4. 🔴 **Chuỗi NHIỀU DÒNG bị từ chối thẳng.** Luật này sinh ra ở đúng lần chạy đầu tiên của
   script, 2026-08-16: `awk`/`grep` xử lý theo từng dòng, nên phép thay im lặng không xảy
   ra — nhưng `grep -F` với pattern nhiều dòng coi **mỗi dòng là một lựa chọn riêng** nên
   bộ đếm vẫn báo tìm thấy. Script khi đó in **"ĐỘT BIẾN SỐNG SÓT"** cho một đột biến
   **chưa bao giờ được áp**. Đó là dương tính giả nói rằng bộ test không có răng trong khi
   chưa ai thử răng nó — nguy hiểm hơn mọi thứ script này định phát hiện. Nay script chặn
   thẳng, và còn `cmp` file trước/sau để chứng minh phép thay đã xảy ra thật.

### Kết quả đo, 2026-08-16

Cổng đo là `node check-parity.mjs` **một mình** (tức chỉ bộ 281 vector, không tính unit test):

| Đột biến | Kết quả |
|---|---|
| `TONE_PLACEMENT_MODERN && is_glide_pair` → `false && …` | bắt — **đúng 7 vector oa/oe/uy**, không lan |
| `MODIFIED`: bỏ `ơ` khỏi ưu tiên dấu | bắt — RUST 245/281 |
| escape phím thanh: tắt hẳn | bắt — 269/281 |
| escape cặp dấu phụ: tắt hẳn | bắt — 272/281 |
| chuẩn hoá vần: bỏ luật `lazy` | bắt — 275/281 |
| cụm `qu`/`gi`: đổi ngưỡng độ dài 3→4 | bắt — 278/281 |
| backspace: pop đúng MỘT phím thay vì pop tới khi text đổi | bắt |
| `ends_syllable`: `is_ascii_alphabetic` → `is_alphabetic` | **SỐNG SÓT** |

**Lỗ hổng duy nhất, và nó có tính cấu trúc**: bộ vector **không thể với tới**
`ends_syllable` bằng một chữ cái non-ASCII. Vector là chuỗi **phím gõ**, mà phím thì luôn
ASCII (người ta gõ `dd`, không gõ `đ`); `@section transform` còn giới hạn cứng `[A-Za-z]`.

Đây **không phải** lỗ hổng cần vá bằng vector — nó là đúng ca mà
[AD-1](#ad-1) cho phép unit test: *"bất biến mà một vector không phát biểu được"*. Bài
`only_ascii_letters_continue_a_syllable` trong `lib.rs` bắt được nó, và đó là cơ chế đúng.

**Hệ quả cho cách đọc cổng**: `cargo test` xanh không đủ, và **`check-parity.mjs` xanh
cũng không đủ**. Hai cổng phủ hai vùng khác nhau và chỉ giao nhau một phần. Cổng thật là
**cả hai**.
