# Lộ trình

Nhịp giả định **8–10 giờ/tuần**, một người. Full-time thì chia cho 4.

Lý do của mọi lựa chọn kiến trúc nằm ở [ARCHITECTURE.md](ARCHITECTURE.md).

---

## Trạng thái

| | Việc | Xong? |
|---|---|---|
| **M0** | Live USB Ubuntu — trả lời câu gạch chân | ⛔ **CHẶN**, việc của người, không phải của code |
| **M1** | Harness ba engine, lõi còn stub | ✅ |
| **M2** | Port `Transform()` | ✅ |
| **M3** | Luật ngắt âm tiết + `Session` | ✅ — xong ở **M1**, trước cả `Transform()`: 4/8 vector xanh ở giai đoạn stub chính là vector integration, chúng chứng minh luật ngắt + đường apply đã đúng khi engine còn rỗng |
| **M4** | Khoá CI | ✅ |
| **M5** | Backspace bằng replay | ✅ |
| **M6** | IBus shell hello-world, WSL2 | ⬜ cần WSL2 |
| **M7** | Dual boot — ngày dogfood | ⬜ cần máy Ubuntu |
| **M8** | Ma trận app — tuần không viết code | ⬜ |
| **M9** | Đóng gói `.deb` / PPA | ⬜ chỉ khi có người dùng thứ hai |

```
node check-parity.mjs
  PASS  298 vectors — all three engines: JS 283/283, AHK 283/283, RUST 298/298
```

**v0 = tới hết M7** — chạy được trên máy mình, `<layout>default</layout>`, zero
packaging. **v1 = M8 + M9**, và chỉ mở khi có người dùng thứ hai; named XKB
layout và `.deb` sống ở đó.

M1–M5 land ngày 2026-08-16, ba ngày sau baseline. **Ước lượng dưới đây là CÔNG
SỨC CÒN LẠI, không phải lịch** — số tuần đếm từ lúc bắt đầu chặng đó, không phải
từ đầu dự án.

---

## M0 — hai cổng chặn. Không viết dòng code nào.

**Đây là việc của anh, không phải việc tôi làm hộ được.** Cả hai chặn M6, và cả
hai rẻ hơn rất nhiều so với việc phát hiện muộn.

### M0.a — Gạch chân có chịu được không? *(20 phút)*

Rủi ro sản phẩm cao nhất của dự án. IBus vẽ gạch chân dưới âm tiết đang gõ, và
trên Ubuntu 26.04 **không tắt được** ([AD-7](docs/architecture-decisions.md#ad-7)). Bản AHK anh đang
dùng thì gõ thẳng, không có gạch chân — nên đây là thay đổi anh sẽ nhìn thấy mọi
lúc, mọi ngày.

Live USB Ubuntu 26.04, rồi:

```bash
sudo apt install ibus-unikey
gsettings set org.gnome.desktop.input-sources sources \
  "[('xkb','us+dvorak'),('ibus','Unikey')]"
```

Gõ tiếng Việt **thật** 20 phút trong Text Editor, Firefox, VS Code. Một câu hỏi
duy nhất: sống được với nó không?

- **Được** → preedit là spec. Toàn bộ kế hoạch dưới đây chạy nguyên vẹn.
- **Không** → cần đường non-preedit (`CommitText` + `DeleteSurroundingText`).
  ibus-bamboo chứng minh nó chạy được, giá phải trả là sleep 30–50 ms quanh mỗi
  fake backspace và một **ma trận tương thích per-app phải bảo trì đời đời**.
  Chi phí đổi hướng ở tuần 1 ≈ 3 tuần; phát hiện ở M7 ≈ 3 tháng.

### M0.b — `<layout_variant>` có bị nuốt thật không? *(5 phút)*

Có thể lật đổ [AD-6](docs/architecture-decisions.md#ad-6) (và cùng với nó là nhu
cầu ship named XKB layout ở M9) — đó là lý do AD đó đang ở trạng thái `draft`. Lựa chọn IBus-thay-vì-evdev và
IBus-thay-vì-fcitx5 KHÔNG phụ thuộc vào kết quả này. Kết luận hiện tại là **suy
luận từ source code gnome-shell, không phải quan sát**.

Trên máy 26.04 thật: lấy một component XML có sẵn, thay bằng một block `<engine>`
inline với `<layout>us</layout>` + `<layout_variant>dvorak</layout_variant>`,
`sudo ibus write-cache --system && ibus restart`, kích hoạt, rồi chạy `wev` và
bấm phím vật lý `;`.

- ra keysym `s` → variant chạy, thiết kế named-layout là thừa
- ra `;` → named-layout là bắt buộc

**Nghi thức nên giữ mãi:** trước mỗi phiên gõ, `wev` + phím `;` phải ra `s`. Mọi
lỗi layer-1 đều im lặng và trông y hệt bug engine.

---

## M6 — IBus shell hello-world *(≈2 tuần ở nhịp 8–10 h/tuần)*

WSL2 headless là đủ cho phần này: contract D-Bus, codec `IBusText`, sequencing
preedit/commit, integration test. CI của chính IBus chạy suite tương đương trên
GitHub Actions không cần X server.

- zbus Factory + Engine + Service
- **Unit-test codec `IBusText` `(sa{sv}sv)` TRƯỚC lần gửi live đầu tiên.** Một
  struct sai shape crash *toàn bộ* `ibus-daemon`, và maintainer đã đóng issue đó
  "not planned". Triệu chứng không phải "chương trình tôi panic" mà là "cả session
  ngừng nhận tiếng Việt".
- Lần gửi live đầu tiên làm trong VM hoặc TTY bỏ được, **không phải session duy
  nhất**.
- Assert lúc bring-up: bấm `a`, keycode phải về **30**. Xem
  [AD-14](docs/architecture-decisions.md#ad-14) — dữ kiện này đã đảo chiều hai lần.
- Thêm `deny.toml` ban crate network cùng lúc `zbus` vào ([AD-10](docs/architecture-decisions.md#ad-10)).

**WSL2 không làm được layer 1**, về mặt cấu trúc: cơ chế nằm trong gnome-shell,
còn WSLg chạy Weston. Đừng cố test nó ở đó.

---

## M7 — Dual boot, ngày dogfood *(≈2 tuần)*

Đích: `Tieengs Vieejt` ra `Tiếng Việt` trên Ubuntu.

**Dual boot trên chính máy này — không phải VM, và không thay thế Windows.**

- **Không VM:** máy có 15.8 GB RAM và một NVMe 512 GB duy nhất *(kiểm
  2026-08-16: Patriot M.2 P320 512GB — chạy lại `Get-PhysicalDisk` trước khi tin
  dòng này)*. Phán xét *cảm
  giác* preedit trong một VM giật là đo bằng thước cong. Thêm nữa mọi transport
  remote (VNC/RDP/Hyper-V enhanced session) inject event *hạ nguồn* tầng layout,
  nên layer 1 có thể trông như không chạy trong khi thực ra nó chạy.
- **Mua thêm một thanh M.2** (~40–60 USD) thay vì thu nhỏ partition trên ổ duy
  nhất. Khoản chi tốt nhất của dự án.
- **Không chuyển hẳn sang Ubuntu.** Bẫy chicken-and-egg: AHK là bộ gõ tiếng Việt
  hàng ngày của anh và nó chỉ có trên Windows. Chuyển trước khi engine chạy nghĩa
  là dùng ibus-bamboo hàng ngày — mà bamboo là *dialect Telex khác, đặt phím
  khác*. Nó sẽ làm hỏng cả muscle memory lẫn **phán đoán của anh về việc engine
  mới nên cảm giác thế nào**, và phán đoán đó là tài sản không ai thay thế được.

Vòng lặp sau khi cài một lần (symlink `/usr/libexec/` vào build tree):

```bash
cargo build --release && pkill -f ibus-engine-vietdvorak && ibus engine vietdvorak
```

Chỉ cần `ibus write-cache --system` khi component XML đổi. **Đừng dùng `ibus
restart`** trên 26.04 — có bug đang mở khiến IM ngừng hoạt động trong Shell trên
Wayland trong khi app dùng `GTK_IM_MODULE=ibus` vẫn chạy. Một failure *một phần*
tốn cả buổi để chẩn đoán. Chỉ dẫn đúng là **log out rồi log in**.

---

## M8 — Tuần không viết code *(1 tuần trọn, không nén được)*

Gõ thật cả tuần. Ghi bảng: GNOME Text Editor, Firefox, Ptyxis, VS Code, Slack,
một app Qt. Gạch chân render thế nào, commit-on-space có đúng nhịp không, app nào
ăn mất preedit.

**Thu thập dữ liệu, không thêm feature.** Nếu không kỷ luật giữ tuần này, bảng đó
sẽ không bao giờ được viết ra — và nó là input cho mọi quyết định v1.

Đây cũng là lúc viết `ACCEPTANCE.md`: checklist thủ công chỉ có nghĩa khi đã có
thứ để test thủ công.

---

## M9+ — chỉ khi có người dùng thứ hai

Named XKB layout + `.deb` + PPA. Zero maintainer script (`ibus-unikey`,
`ibus-anthy`, `ibus-libpinyin` đều ship 0). `debian/NEWS` ghi "log out", **không**
"ibus restart".

**Quyết định tường minh cần chốt trước, đừng để biết qua bug report:**

- **fcitx5** — user Việt Nam dùng fcitx5 sẽ không thấy engine, vì fcitx5 không
  nạp được engine IBus. Hoặc budget một shell nữa, hoặc tuyên bố loại trừ công
  khai.
- **Ubuntu 24.04** — đường upgrade 24.04→26.04 chưa mở, nên installed base thật
  vẫn ở 24.04 (GNOME 46, còn X11, libxkbcommon quá cũ) tới sâu 2027. Có trong
  scope hay không?

---

## Đã hoãn có chủ ý

Mỗi mục kèm **một** quyết định giữ cửa cho nó, để thêm sau là *cộng thêm* chứ
không phải thiết kế lại.

| Hoãn | Quyết định giữ cửa | Giá hôm nay |
|---|---|---|
| Shell Windows (injection) | `View.committed` là field **riêng biệt** khỏi `composing`, kèm ba mệnh đề trong doc comment | ~20 dòng |
| macOS | Lõi zero dependency, không kiểu dữ liệu nào của nền tảng | 0 |
| fcitx5 | Shell là crate riêng; lõi không có **type** hay **dependency** nào của nền tảng (tên IBus chỉ xuất hiện trong chú thích lý do — đó là chỗ đúng của nó) | 0 |
| Named XKB layout, `.deb` | v0 khai `<layout>default</layout>` — không gì phụ thuộc vào cơ chế engine-declares-layout | 0 |
| File config | `TONE_PLACEMENT_MODERN` giữ là một `const` có tên | 0 — đừng thêm serde+toml+watcher cho một boolean |
| `deny.toml` | Chưa có dependency nào để ban | 0 |
| `ACCEPTANCE.md` | Chưa có shell nào để test thủ công | 0 |

**Cắt vĩnh viễn:** double-tap LCtrl = copy, LShift = paste. Vốn từ của một IBus
engine không có clipboard. Đây là feature macro bàn phím tồn tại chỉ vì AHK tình
cờ là công cụ macro; nhà của chúng trên Linux là GNOME custom shortcuts.

**Cắt khỏi v0, có rủi ro:** double-tap CapsLock = toggle EN/VN. Trong mô hình
IBus, EN và VN là *hai input source*, chuyển bằng `Super+Space`. Đây là thay đổi
muscle memory hàng ngày của anh và là deferral dễ quay lại nhất; nếu quay lại thì
là shortcut cấu hình được qua `IBusProperty`, **không bao giờ** là grab CapsLock
hard-code.
