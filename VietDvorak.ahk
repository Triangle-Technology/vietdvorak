#Requires AutoHotkey v2.0
#SingleInstance Force

InstallKeybdHook()  ; AHK v2 function (was #InstallKeybdHook directive in v1)
                    ; needed so A_PriorKey tracks all keys for double-tap detection

; ── Ngưỡng chống-chạy-loạn ───────────────────────────────────────────────────
; AHK đếm số lần hotkey kích hoạt trong mỗi A_HotkeyInterval (mặc định 2000 ms);
; vượt ngưỡng A_MaxHotkeysPerInterval thì nó bật một HỘP THOẠI MODAL cảnh báo.
; Script này móc gần như cả không gian virtual-key ở khối "KẾT THÚC ÂM TIẾT" bên
; dưới — 0x08..0xFE trừ 26 chữ cái và 5 mã được chừa = 216 hotkey ~*, cộng 5 nút
; chuột và Backspace là ~222 — nên GIỮ PHÍM cũng đếm: auto-repeat của Windows
; sinh ra một sự kiện keydown, tức một lần kích hoạt hotkey, cho mỗi nhịp lặp.
;
; Số học (giữ Backspace để xoá một từ, hay giữ mũi tên để cuộn — thao tác bình
; thường, không phải lạm dụng):
;   trần tốc độ lặp của Windows (SPI_SETKEYBOARDSPEED nấc 31)  ≈ 30 lần/giây
;   cửa sổ đếm A_HotkeyInterval                                =  2 giây
;   -> một phím GIỮ                        30 × 2              =  60 lần
;   + gõ thường cùng lúc (người gõ nhanh ~10 phím/giây × 2 s)  =  20 lần
;   -------------------------------------------------------------------------
;   tổng cho một ca hoàn toàn bình thường                      =  80 lần
; Mặc định 70 < 80 -> gõ bình thường cũng đủ bật hộp thoại. Lấy 80 × 2.5 làm
; biên an toàn -> 200.
;
; KHÔNG dùng A_HotkeyInterval := 0: nó tắt hộp thoại bằng cách tắt luôn phép
; đếm, tức vứt bỏ lưới an toàn chống vòng lặp Send↔hotkey. Script này Send đúng
; những phím nó tự móc (SyncToScreen), nên lưới đó là bảo vệ thật. Vòng lặp thật
; sự chạy loạn kích hoạt hàng nghìn lần mỗi giây, vượt 200 trong ~0.1 s — vẫn bị
; bắt ngay, chỉ khác là gõ bình thường thì không.
A_MaxHotkeysPerInterval := 200

; Distinct tray icon + tooltip so it doesn't blend with other AHK scripts.
; The icon itself is drawn at runtime in UpdateTrayIcon() — orange square with
; "V" (Vietnamese mode) or "E" (English mode) — so mode is visible at a glance.
global TRAY_TIP_BASE := "VietDvorak for Windows — Vietnamese Telex (double-tap CapsLock to toggle EN/VN)"
A_IconTip := TRAY_TIP_BASE

; ============================================================================
; VietDvorak v1.0 - Vietnamese Telex input optimized for Dvorak
; ============================================================================
;
; PREREQUISITE: set Windows keyboard layout to "United States-Dvorak".
;   Settings -> Time & language -> Language & region -> English (US) ->
;     Language options -> Add a keyboard -> United States-Dvorak
;
; TONE KEYS (standard Telex. Measured by bigram-late-tone.mjs, late-tone style,
; n=510 toned syllables: x/j alternates 59.0% vs 54.1% for an all-right z/b
; variant, +4.9pp. The gain is all nặng=j (+7.8pp on its own) — 66% of nặng
; syllables end on a right-hand char, but only 46% do overall: huyền/hỏi/ngã end
; LEFT-hand 68/83/80% of the time. ngã=x scores -2.9pp and is kept not for
; alternation but for standard-Telex muscle-memory compatibility):
;   s = sac    (acute)    right pinky, home row        (~21%)
;   f = huyen  (grave)    right index, top row         (~22%)
;   r = hoi    (hook)     right ring, top row          (~9%)
;   x = nga    (tilde)    left index stretch, bottom   (~4%)
;   j = nang   (dot)      left middle, bottom row      (~18%)
; (Dvorak x sits on the physical QWERTY-B key = left index stretch; Dvorak j
;  sits on the physical QWERTY-C key = left middle. See the #keyboard diagram
;  in practice.html.)
;
; DIACRITIC PAIRS (standard Telex):
;   aa = a-circumflex   ee = e-circumflex   oo = o-circumflex
;   aw = a-breve        ow = o-horn         uw = u-horn
;   dd = d-stroke
;
; SHORTCUTS:
;   CapsLock (double-tap)     toggle Vietnamese / English mode
;   Left-Ctrl (double-tap)    copy   (sends Ctrl+C)
;   Left-Shift (double-tap)   paste  (sends Ctrl+V)
;   Ctrl+Shift+V              also toggles EN/VN (backup hotkey)
;   Ctrl+Shift+Q              quit script
;   Double-press tone         remove tone (escape, e.g., "ass" -> "as")
;
; SMART TONE PLACEMENT:
;   Tone is placed on the right vowel automatically, even if you type the
;   tone key in the middle of a syllable:
;     - "tieengs" -> tiếng (tone on modified ê)
;     - "ngoaif"  -> ngoài (tone on main vowel a, not the last vowel i)
;     - "soajn"   -> soạn  (tone migrates from o to a when n is typed)
;     - "Quynhf"  -> Quỳnh (qu cluster skipped; tone on y)
;     - "hoaf"    -> hoà   (oa/oe/uy: see TONE_PLACEMENT_MODERN below)
;
; ============================================================================

; ── Quy ước đặt dấu thanh ───────────────────────────────────────────────────
; Với âm tiết MỞ có hai nguyên âm "oa", "oe", "uy" (và chỉ ba cặp này) tiếng
; Việt có hai quy ước đặt dấu, Unikey gọi tuỳ chọn này là "Đặt dấu oà, uý":
;   true  (hiện đại)  -> hoà, khoẻ, thuỷ   — dấu trên nguyên âm chính
;   false (truyền thống) -> hòa, khỏe, thủy — dấu trên âm đệm
; Cả hai đều được chấp nhận; điều bắt buộc là PHẢI NHẤT QUÁN cho cả ba cặp.
; Đổi ở đúng MỘT chỗ này, không rải if-else trong FindToneTarget.
global TONE_PLACEMENT_MODERN := true

; Cặp âm đệm + nguyên âm chính mà hai quy ước trên đặt dấu khác nhau.
global GLIDE_PAIRS := ["oa", "oe", "uy"]

; Cửa sổ nhận diện double-tap (ms). Dùng ở đúng một chỗ — cả code lẫn chú thích
; đều tham chiếu tên này, không viết lại con số.
global DOUBLE_TAP_MS := 400

global g_vnMode := true
global g_buffer := ""
; True iff the previous keystroke produced "ư"/"Ư" via the Unikey "w"-only
; shortcut (empty/consonant buffer + "w"). Used so that "ww" escapes to "w"
; instead of "uw" — there's no implicit "u" to restore when the "ư" came from
; a single "w" key, only from the "uw" pair.
global g_lastWWasShortcut := false

; Đã báo lỗi vẽ icon khay chưa? UpdateTrayIcon chạy lúc khởi động và mỗi lần
; đổi chế độ; lỗi GDI thường dai dẳng nên chỉ báo một lần thay vì spam.
global g_iconErrorShown := false

; Help window is built once and hidden/shown thereafter (cheaper, and remembers
; its on-screen position between opens).
global g_helpGui := 0

; vowel -> [no_tone, sac, huyen, hoi, nga, nang]
global VOWELS := Map(
    "a", ["a", "á", "à", "ả", "ã", "ạ"],
    "ă", ["ă", "ắ", "ằ", "ẳ", "ẵ", "ặ"],
    "â", ["â", "ấ", "ầ", "ẩ", "ẫ", "ậ"],
    "e", ["e", "é", "è", "ẻ", "ẽ", "ẹ"],
    "ê", ["ê", "ế", "ề", "ể", "ễ", "ệ"],
    "i", ["i", "í", "ì", "ỉ", "ĩ", "ị"],
    "o", ["o", "ó", "ò", "ỏ", "õ", "ọ"],
    "ô", ["ô", "ố", "ồ", "ổ", "ỗ", "ộ"],
    "ơ", ["ơ", "ớ", "ờ", "ở", "ỡ", "ợ"],
    "u", ["u", "ú", "ù", "ủ", "ũ", "ụ"],
    "ư", ["ư", "ứ", "ừ", "ử", "ữ", "ự"],
    "y", ["y", "ý", "ỳ", "ỷ", "ỹ", "ỵ"]
)

; SUY RA từ VOWELS lúc khởi động (không chép tay bảng thứ hai): chỉ có MỘT
; nguồn sự thật cho danh sách nguyên âm + dấu thanh.
global UPPER_VOWELS := BuildUpperVowels()

BuildUpperVowels() {
    global VOWELS
    m := Map()
    for bv, tones in VOWELS {
        upper := []
        for t in tones
            upper.Push(StrUpper(t))
        m[StrUpper(bv)] := upper
    }
    return m
}

global TONE_KEYS := Map(
    "s", 1,  ; sac    R-pinky home
    "f", 2,  ; huyen  R-index top
    "r", 3,  ; hoi    R-ring top
    "x", 4,  ; nga    L-index stretch, bottom (standard Telex)
    "j", 5   ; nang   L-middle bottom (standard Telex)
)

; Tên tiếng Việt của từng thanh, khoá theo chỉ số thanh trong VOWELS (1..5).
; Chỉ là DỮ LIỆU HIỂN THỊ cho hộp thoại trợ giúp — ánh xạ phím -> thanh vẫn chỉ
; sống ở TONE_KEYS, hộp thoại đọc thẳng từ đó chứ không chép lại.
global TONE_NAMES := Map(
    1, "sắc", 2, "huyền", 3, "hỏi", 4, "ngã", 5, "nặng"
)

global DIACRITIC_PAIRS := Map(
    "aa", "â", "ee", "ê", "oo", "ô",
    "aw", "ă", "ow", "ơ", "uw", "ư",
    "dd", "đ"
)

; Reverse map for escape: pair-result char -> {firstChar, secondKey}
; Lets typing the same trigger key twice undo the diacritic.
;   "oww" -> "ow",  "aaa" -> "aa",  "ddd" -> "dd"
; SUY RA từ DIACRITIC_PAIRS (nghịch đảo của nó) — thêm/bớt một cặp ở trên là
; bảng này tự đúng theo, không có bản sao thứ hai để quên cập nhật.
global PAIR_ESCAPE := BuildPairEscape()

BuildPairEscape() {
    global DIACRITIC_PAIRS
    m := Map()
    for pair, result in DIACRITIC_PAIRS {
        first  := SubStr(pair, 1, 1)
        second := SubStr(pair, 2, 1)
        m[result]           := {firstChar: first,           secondKey: second}
        m[StrUpper(result)] := {firstChar: StrUpper(first), secondKey: second}
    }
    return m
}

; Nguyên âm gốc -> nguyên âm có dấu móc/trăng, cũng SUY RA từ DIACRITIC_PAIRS
; (chính là các cặp có phím thứ hai là "w"):  a->ă  o->ơ  u->ư
global W_MODIFIER := BuildWModifierMap()

BuildWModifierMap() {
    global DIACRITIC_PAIRS
    m := Map()
    for pair, result in DIACRITIC_PAIRS {
        if (SubStr(pair, 2, 1) == "w")
            m[SubStr(pair, 1, 1)] := result
    }
    return m
}

; ── Dấu móc đặt theo CỤM NGUYÊN ÂM, không phải theo từng ký tự ───────────────
; Cụm nguyên âm cuối buffer (chữ cái gốc, đã bỏ dấu thanh) -> dạng đã móc, khi
; bấm "w". Chỉ liệt kê những cụm mà dấu móc KHÔNG rơi vào nguyên âm cuối; mọi
; cụm khác đi theo đường mặc định bên dưới (cặp aw/ow/uw, hoặc quét ngược).
;   uu  -> ưu    lựu, hưu, cứu     ("uư" không tồn tại trong tiếng Việt)
;   uou -> ươu   rượu, hươu, bướu  ("uoư" không tồn tại)
global HORN_NUCLEUS := Map(
    "uu",  "ưu",
    "uou", "ươu"
)

; Chuẩn hoá vần: cụm nguyên âm đặt dấu móc SAI CHỖ -> dạng đúng chính tả. Cả hai
; mục đều quy về "ươ", và "ươ" là dạng đúng DUY NHẤT, nên phép sửa là xác định.
;   ưo -> ươ    đưoc -> được, ngưoi -> người, rưou -> rượu
;   uơ -> ươ    đuợc -> được, thuơng -> thương
;
; `lazy` = CHỜ ĐUÔI: chỉ sửa khi vần còn ký tự phía sau (chưa nằm cuối buffer).
; Hai mục khác nhau ở đúng một điểm, và điểm đó là thuộc tính CHÍNH TẢ của từng
; vần, không phải một lựa chọn của thuật toán:
;
;   "ưo" -> lazy: false, SỬA NGAY.
;        Vì "ưo" KHÔNG HỢP LỆ Ở MỌI ĐỘ DÀI: không âm tiết tiếng Việt nào chứa
;        "ưo", dù nó đứng cuối ("đưo") hay còn đuôi ("đưoc"). Không có dạng hợp
;        lệ nào để bảo vệ, nên không có gì phải chờ — chờ chỉ tổ để người dùng
;        nhìn thấy "đưo", thứ không tồn tại.
;
;   "uơ" -> lazy: true, PHẢI CHỜ.
;        Vì "uơ" TỰ NÓ LÀ MỘT VẦN MỞ HỢP LỆ: thuơ (thuở), huơ, quơ. Sửa ngay là
;        phá đúng chính tả mà người dùng vừa gõ ra. Chỉ khi có ký tự tiếp theo
;        mới biết chắc vần không dừng ở dạng mở đó ("thuơng" -> "thương").
;
; Nói gọn: eager khi vần sai ở mọi độ dài, lazy khi vần là dạng mở hợp lệ.
; Vì vậy điều kiện sống trong BẢNG, không phải if-else hỏi tên vần trong
; NormalizeNucleus: thêm một vần mới chỉ là thêm một dòng ở đây.
global NUCLEUS_NORMALIZE := Map(
    "ưo", {fixed: "ươ", lazy: false},
    "uơ", {fixed: "ươ", lazy: true}
)

global ALL_VOWEL_CHARS := BuildVowelSet()

BuildVowelSet() {
    global VOWELS, UPPER_VOWELS
    s := ""
    for k, tones in VOWELS
        for t in tones
            s .= t
    for k, tones in UPPER_VOWELS
        for t in tones
            s .= t
    return s
}

IsVowelChar(ch) {
    global ALL_VOWEL_CHARS
    return InStr(ALL_VOWEL_CHARS, ch, true) > 0
}

IsToneKey(ch) {
    global TONE_KEYS
    return TONE_KEYS.Has(StrLower(ch))
}

FindVowelInfo(ch) {
    global VOWELS, UPPER_VOWELS
    ; NOTE: property is named `baseLetter` not `base` because AHK v2 reserves
    ; `base` for the prototype chain — assigning a string to it errors out.
    for bv, tones in VOWELS {
        Loop tones.Length {
            if tones[A_Index] == ch
                return {baseLetter: bv, tone: A_Index - 1, isUpper: false}
        }
    }
    for bv, tones in UPPER_VOWELS {
        Loop tones.Length {
            if tones[A_Index] == ch
                return {baseLetter: bv, tone: A_Index - 1, isUpper: true}
        }
    }
    return ""
}

; Choose which vowel hosts the tone (Vietnamese placement rules).
;   1. Skip leading "qu" or "gi" + vowel (initial consonant clusters)
;   2. If a modified vowel (â,ă,ê,ô,ơ,ư) is present, use the LAST modified
;   3. If syllable ends in consonant: tone on last vowel
;   4. Open syllable: glide pair (oa/oe/uy) -> per TONE_PLACEMENT_MODERN;
;      otherwise second-to-last vowel
FindToneTarget(buffer) {
    global TONE_PLACEMENT_MODERN
    chars := []
    Loop Parse, buffer
        chars.Push(A_LoopField)

    startIdx := 1
    if chars.Length >= 3 {
        c0 := StrLower(chars[1])
        c1 := StrLower(chars[2])
        if (c0 == "q" && c1 == "u" && IsVowelChar(chars[3]))
            startIdx := 3
        else if (c0 == "g" && c1 == "i" && IsVowelChar(chars[3]))
            startIdx := 3
    }

    vowels := []
    Loop chars.Length - startIdx + 1 {
        i := startIdx + A_Index - 1
        if IsVowelChar(chars[i]) {
            info := FindVowelInfo(chars[i])
            baseLower := IsObject(info) ? StrLower(info.baseLetter) : StrLower(chars[i])
            vowels.Push({pos: i, baseLower: baseLower})
        }
    }

    ; Fallback: cluster skip swallowed the only vowel — scan from the start
    if vowels.Length == 0 && startIdx > 1 {
        Loop chars.Length {
            if IsVowelChar(chars[A_Index]) {
                info := FindVowelInfo(chars[A_Index])
                baseLower := IsObject(info) ? StrLower(info.baseLetter) : StrLower(chars[A_Index])
                vowels.Push({pos: A_Index, baseLower: baseLower})
            }
        }
    }

    if vowels.Length == 0
        return 0
    if vowels.Length == 1
        return vowels[1].pos

    ; Prefer the LAST modified vowel (handles "ươ" -> ơ, etc.)
    modifiedPos := 0
    for v in vowels {
        if (v.baseLower == "â" || v.baseLower == "ă" || v.baseLower == "ê"
         || v.baseLower == "ô" || v.baseLower == "ơ" || v.baseLower == "ư")
            modifiedPos := v.pos
    }
    if modifiedPos > 0
        return modifiedPos

    lastChar := chars[chars.Length]
    if !IsVowelChar(lastChar) {
        ; Closed syllable
        return vowels[vowels.Length].pos
    }

    ; Open syllable: oa/oe/uy là đúng ba cặp mà hai quy ước đặt dấu khác nhau —
    ; xử lý CẢ BA giống nhau (trước đây chỉ "uy" được ưu tiên, nên script tự mâu
    ; thuẫn: thuỷ theo kiểu mới nhưng hòa/khỏe theo kiểu cũ). Còn lại: áp chót.
    lastV := vowels[vowels.Length].baseLower
    prevV := vowels[vowels.Length - 1].baseLower
    if (TONE_PLACEMENT_MODERN && IsGlidePair(prevV, lastV))
        return vowels[vowels.Length].pos
    return vowels[vowels.Length - 1].pos
}

IsGlidePair(prevV, lastV) {
    global GLIDE_PAIRS
    for p in GLIDE_PAIRS {
        if (prevV == SubStr(p, 1, 1) && lastV == SubStr(p, 2, 1))
            return true
    }
    return false
}

ApplyToneToBuffer(buffer, toneIdx) {
    global VOWELS, UPPER_VOWELS

    targetPos := FindToneTarget(buffer)
    if targetPos == 0
        return ""

    targetChar := SubStr(buffer, targetPos, 1)
    info := FindVowelInfo(targetChar)
    if !IsObject(info)
        return ""

    ; Always apply (escape "ss" -> "as" is handled in Transform())
    table := info.isUpper ? UPPER_VOWELS : VOWELS
    newChar := table[info.baseLetter][toneIdx + 1]

    return SubStr(buffer, 1, targetPos - 1) . newChar . SubStr(buffer, targetPos + 1)
}

; Returns object {pos, toneIdx} if buffer has a toned vowel, else ""
GetCurrentTone(buffer) {
    pos := 0
    Loop Parse, buffer {
        pos += 1
        if IsVowelChar(A_LoopField) {
            info := FindVowelInfo(A_LoopField)
            if (IsObject(info) && info.tone > 0)
                return {pos: pos, toneIdx: info.tone}
        }
    }
    return ""
}

; Remove tones from all vowels (preserve diacritics)
StripTone(buffer) {
    global VOWELS, UPPER_VOWELS
    out := ""
    Loop Parse, buffer {
        ch := A_LoopField
        if IsVowelChar(ch) {
            info := FindVowelInfo(ch)
            if (IsObject(info) && info.tone > 0) {
                table := info.isUpper ? UPPER_VOWELS : VOWELS
                ch := table[info.baseLetter][1]
            }
        }
        out .= ch
    }
    return out
}

ApplyDiacriticPair(buffer) {
    global DIACRITIC_PAIRS

    if StrLen(buffer) < 2
        return buffer

    last2 := SubStr(buffer, -2)
    last2Lower := StrLower(last2)

    if !DIACRITIC_PAIRS.Has(last2Lower)
        return buffer

    replacement := DIACRITIC_PAIRS[last2Lower]
    firstChar := SubStr(last2, 1, 1)
    if firstChar !== StrLower(firstChar)
        replacement := StrUpper(replacement)

    return SubStr(buffer, 1, -2) . replacement
}

; Chữ cái gốc (viết thường, đã bỏ dấu thanh) của một nguyên âm; "" nếu không
; phải nguyên âm. "ừ" -> "ư", "Ộ" -> "ô", "c" -> "".
VowelBase(ch) {
    info := FindVowelInfo(ch)
    return IsObject(info) ? StrLower(info.baseLetter) : ""
}

; Đổi chữ cái gốc của nguyên âm tại chars[pos] sang baseLower, GIỮ NGUYÊN dấu
; thanh và kiểu hoa/thường đang có: "ọ" + "ơ" -> "ợ", "U" + "ư" -> "Ư".
; Nhờ vậy mọi phép sửa vần đều tone-preserving mà không phải viết lại ở từng chỗ.
SetVowelBaseAt(chars, pos, baseLower) {
    global VOWELS, UPPER_VOWELS
    info := FindVowelInfo(chars[pos])
    if !IsObject(info)
        return false
    if info.isUpper
        chars[pos] := UPPER_VOWELS[StrUpper(baseLower)][info.tone + 1]
    else
        chars[pos] := VOWELS[baseLower][info.tone + 1]
    return true
}

; Chuẩn hoá vần theo NUCLEUS_NORMALIZE: dấu móc phải nằm đúng chỗ chính tả quy
; định. Trước đây chỗ này chỉ vá được một trường hợp ("uơ" -> "ươ"); nay cả
; "ưo" -> "ươ" dùng chung một bảng, nên "đưoc"/"đuợc" đều ra "được".
; Việc "có phải chờ ký tự phía sau không" đọc từ cờ `lazy` của TỪNG mục trong
; NUCLEUS_NORMALIZE (lý do ở chú thích của bảng) — hàm này không biết, và không
; cần biết, vần nào là vần nào.
NormalizeNucleus(buffer) {
    global NUCLEUS_NORMALIZE
    chars := []
    Loop Parse, buffer
        chars.Push(A_LoopField)
    if chars.Length < 2
        return buffer
    changed := false
    Loop chars.Length - 1 {
        i := A_Index
        b0 := VowelBase(chars[i])
        b1 := VowelBase(chars[i + 1])
        if (b0 == "" || b1 == "")
            continue
        shape := b0 . b1
        if !NUCLEUS_NORMALIZE.Has(shape)
            continue
        rule := NUCLEUS_NORMALIZE[shape]
        if (rule.lazy && i + 1 == chars.Length)
            continue                      ; vần đang ở cuối buffer, còn có thể là dạng hợp lệ
        fixed := rule.fixed
        SetVowelBaseAt(chars, i,     SubStr(fixed, 1, 1))
        SetVowelBaseAt(chars, i + 1, SubStr(fixed, 2, 1))
        changed := true
    }
    if !changed
        return buffer
    out := JoinChars(chars)
    ; Vần đổi thì chỗ đặt dấu thanh cũng đổi theo: "ngừo" + i phải ra "người"
    ; chứ không phải "ngừơi". Đặt lại dấu bằng đúng quy tắc chuẩn.
    cur := GetCurrentTone(out)
    if IsObject(cur) {
        retoned := ApplyToneToBuffer(StripTone(out), cur.toneIdx)
        if retoned !== ""
            out := retoned
    }
    return out
}

; Cụm nguyên âm này có phải chính là DẠNG ĐÍCH của một mục NUCLEUS_NORMALIZE
; không — tức dấu móc đã nằm đúng chỗ, không còn gì để chuẩn hoá nữa?
;
; NGUYÊN TẮC hàm này phục vụ, phát biểu một lần ở đây:
;     Escape (gõ lại phím trigger) chỉ được phép HUỶ DẤU MÓC DO CHÍNH NGƯỜI DÙNG
;     GÕ RA, không bao giờ được huỷ dấu móc do ENGINE tự chèn khi chuẩn hoá vần.
; Người dùng gõ "dduwo" thì engine đã tự sửa "ưo" thành "ươ" (buffer "đươ"): dấu
; móc trên "ơ" là của engine. Phím "w" kế tiếp vì thế là THỪA — yêu cầu "thêm
; móc" đã được đáp ứng — chứ không phải lệnh huỷ, nên phải nuốt nó, không cho
; rơi xuống nhánh escape ("dduwowngf" -> đường).
; Ngược lại trong "oww"/"uww", cụm cuối "ơ"/"ư" do CHÍNH người dùng gõ ra bằng
; cặp ow/uw của DIACRITIC_PAIRS, nên escape vẫn phải chạy như thường (-> "ow"/"uw").
;
; Đọc thẳng từ bảng chứ không viết lại "ươ" ở đây — thêm vần mới vào
; NUCLEUS_NORMALIZE là chỗ này tự đúng theo.
IsNormalizedNucleus(shape) {
    global NUCLEUS_NORMALIZE
    for k, rule in NUCLEUS_NORMALIZE {
        if (rule.fixed == shape)
            return true
    }
    return false
}

; Cụm nguyên âm ở CUỐI buffer: {start, shape} với shape là chuỗi chữ cái gốc
; (bỏ dấu thanh, viết thường). shape == "" nếu ký tự cuối không phải nguyên âm.
TrailingNucleus(chars) {
    start := chars.Length + 1
    Loop chars.Length {
        i := chars.Length - A_Index + 1
        if VowelBase(chars[i]) == ""
            break
        start := i
    }
    shape := ""
    Loop chars.Length - start + 1
        shape .= VowelBase(chars[start + A_Index - 1])
    return {start: start, shape: shape}
}

; Hậu tố DÀI NHẤT của cụm nguyên âm có trong HORN_NUCLEUS ("" nếu không có).
MatchHornNucleus(shape) {
    global HORN_NUCLEUS
    n := StrLen(shape)
    Loop n {
        len := n - A_Index + 1
        sfx := SubStr(shape, n - len + 1)
        if HORN_NUCLEUS.Has(sfx)
            return sfx
    }
    return ""
}

; Helper: apply 'w' modifier to chars[pos] in-place, returning true if applied.
; Preserves any existing tone: "ọ" -> "ợ", "ụ" -> "ự", "à" -> "ằ", etc.
; This matters when the user types tone before the modifier ("đuojcw" -> "được").
ApplyWModifierAt(chars, pos) {
    global W_MODIFIER
    info := FindVowelInfo(chars[pos])
    if !IsObject(info)
        return false
    baseLower := StrLower(info.baseLetter)
    if !W_MODIFIER.Has(baseLower)
        return false
    return SetVowelBaseAt(chars, pos, W_MODIFIER[baseLower])
}

JoinChars(chars) {
    out := ""
    for c in chars
        out .= c
    return out
}

Transform(buffer, key) {
    global TONE_KEYS, VOWELS, UPPER_VOWELS, PAIR_ESCAPE, DIACRITIC_PAIRS, HORN_NUCLEUS, g_lastWWasShortcut

    ; Snapshot the flag (was the IMMEDIATELY PREVIOUS keystroke a Unikey "w"
    ; shortcut?), then reset. Re-armed below only by the Unikey paths.
    prevWasShortcut := g_lastWWasShortcut
    g_lastWWasShortcut := false

    ; --- Late 'w' modifier paths ---
    ; - "aw"/"ow"/"uw" -> ă/ơ/ư (via DIACRITIC_PAIRS, falls through to pair logic)
    ; - cụm nguyên âm trong HORN_NUCLEUS ("uu" -> "ưu", "uou" -> "ươu")
    ; - "uoiw" -> "uơi" -> "ươi" (via NormalizeNucleus)
    ; - Unikey shortcut: w on consonant-only or empty buffer -> "ư" ("chw" -> "chư")
    if StrLower(key) == "w" {
        if StrLen(buffer) == 0 {
            g_lastWWasShortcut := true
            return (key !== StrLower(key)) ? "Ư" : "ư"
        }
        chars := []
        Loop Parse, buffer
            chars.Push(A_LoopField)
        lastLo := StrLower(chars[chars.Length])

        ; Dấu móc áp lên CẢ CỤM nguyên âm cuối, không phải lên từng ký tự: với
        ; "uu"/"uou" dấu móc không rơi vào nguyên âm cuối ("ưu" chứ không "uư",
        ; "ươu" chứ không "uoư"). Tra bảng HORN_NUCLEUS thay vì đặc-cách từng ca
        ; — nhờ vậy "huouw" -> hươu và "ruouwj" -> rượu chạy đúng như "luuwj".
        ; SetVowelBaseAt giữ nguyên dấu thanh sẵn có, nên thứ tự "dấu trước, móc
        ; sau" ("luujw" -> lựu, "ruoujw" -> rượu) cho kết quả y hệt chiều ngược lại.
        nuc := TrailingNucleus(chars)
        hornShape := MatchHornNucleus(nuc.shape)
        if hornShape !== "" {
            horned := HORN_NUCLEUS[hornShape]
            offset := nuc.start + StrLen(nuc.shape) - StrLen(hornShape)
            Loop StrLen(horned)
                SetVowelBaseAt(chars, offset + A_Index - 1, SubStr(horned, A_Index, 1))
            return NormalizeNucleus(JoinChars(chars))
        }

        ; Vần đã ở ĐÚNG dạng có móc mà NUCLEUS_NORMALIZE nhắm tới ("đưo" vừa được
        ; chuẩn hoá thành "đươ") thì phím "w" tiếp theo không còn gì để làm. Nuốt
        ; nó, KHÔNG để rơi xuống nhánh escape bên dưới — nguyên tắc đầy đủ nằm ở
        ; chú thích của IsNormalizedNucleus, không chép lại ở đây.
        ;   "dduwowngf" -> đường   (w thứ hai là thừa, không phải lệnh huỷ)
        if IsNormalizedNucleus(nuc.shape)
            return buffer

        ; If immediate pair (aw/ow/uw) would form, fall through to normal pair logic.
        ; Otherwise, scan backwards for an unmodified modifiable vowel and apply there.
        immediatePair := lastLo . "w"
        if !DIACRITIC_PAIRS.Has(immediatePair) {
            i := chars.Length
            Loop chars.Length {
                if ApplyWModifierAt(chars, i)
                    return NormalizeNucleus(JoinChars(chars))
                i--
            }
            ; No modifiable vowel. Pair-escape first: "ư" + w -> "uw" (literal escape).
            ; Exception: if the trailing "ư"/"Ư" came from the Unikey "w"-only
            ; shortcut on the previous keystroke (not from a "uw" pair), there's
            ; no implicit "u" to restore — escape to just the literal "w".
            ;   "ww"   -> "w"   (was "uw")
            ;   "chww" -> "chw" (was "chuw")
            lastChar := chars[chars.Length]
            if PAIR_ESCAPE.Has(lastChar) {
                esc := PAIR_ESCAPE[lastChar]
                if StrLower(esc.secondKey) == StrLower(key) {
                    if prevWasShortcut
                        return SubStr(buffer, 1, -1) . key
                    return SubStr(buffer, 1, -1) . esc.firstChar . key
                }
            }
            ; Unikey shortcut: when buffer has no vowels (consonants only) -> append "ư".
            ; Leading "gi" is an initial consonant cluster (like "qu"), so the "i" is
            ; skipped when scanning for vowels — "giw" -> "giư" (giữ, giường, ...).
            ; Skipped if any other vowel is present, so cases like "câw" still produce
            ; literal "câw" (current behavior).
            scanStart := 1
            if (chars.Length >= 2
                && StrLower(chars[1]) == "g"
                && StrLower(chars[2]) == "i") {
                scanStart := 3
            }
            hasVowel := false
            Loop chars.Length - scanStart + 1 {
                if IsObject(FindVowelInfo(chars[scanStart + A_Index - 1])) {
                    hasVowel := true
                    break
                }
            }
            if !hasVowel {
                g_lastWWasShortcut := true
                return buffer . ((key !== StrLower(key)) ? "Ư" : "ư")
            }
            ; Has non-modifiable vowel — fall through to literal-w append
        }
    }

    ; --- Tone key path ---
    if IsToneKey(key) {
        toneIdx := TONE_KEYS[StrLower(key)]
        targetPos := FindToneTarget(buffer)
        if targetPos > 0 {
            targetChar := SubStr(buffer, targetPos, 1)
            info := FindVowelInfo(targetChar)
            if (IsObject(info) && info.tone == toneIdx) {
                ; ESCAPE: same tone key twice -> strip tone + append key literally
                table := info.isUpper ? UPPER_VOWELS : VOWELS
                baseChar := table[info.baseLetter][1]
                stripped := SubStr(buffer, 1, targetPos - 1) . baseChar . SubStr(buffer, targetPos + 1)
                return ApplyDiacriticPair(stripped . key)
            }
        }
        toned := ApplyToneToBuffer(buffer, toneIdx)
        if toned !== ""
            return toned
    }

    ; --- Diacritic-pair escape path ---
    ; If last char is a no-tone pair-result and the new key is the pair's
    ; second trigger, undo the diacritic and append the key as a literal.
    ;   "oww" -> "ow",  "aaa" -> "aa",  "ddd" -> "dd"
    if StrLen(buffer) > 0 {
        lastChar := SubStr(buffer, -1)
        if PAIR_ESCAPE.Has(lastChar) {
            esc := PAIR_ESCAPE[lastChar]
            if StrLower(esc.secondKey) == StrLower(key) {
                ; Replace last char without re-running pair detection
                return SubStr(buffer, 1, -1) . esc.firstChar . key
            }
        }
    }

    ; --- Normal append (with smart tone migration) ---
    existing := GetCurrentTone(buffer)
    if IsObject(existing) {
        ; Strip tone first, then check pair on (stripped + key), then re-apply tone
        stripped := StripTone(buffer)
        result := ApplyDiacriticPair(stripped . key)
        reapplied := ApplyToneToBuffer(result, existing.toneIdx)
        if reapplied !== ""
            result := reapplied
    } else {
        result := ApplyDiacriticPair(buffer . key)
    }
    return NormalizeNucleus(result)
}

SyncToScreen(oldBuf, newBuf) {
    oldArr := []
    newArr := []
    Loop Parse, oldBuf
        oldArr.Push(A_LoopField)
    Loop Parse, newBuf
        newArr.Push(A_LoopField)

    commonLen := 0
    minLen := Min(oldArr.Length, newArr.Length)
    Loop minLen {
        if oldArr[A_Index] !== newArr[A_Index]
            break
        commonLen := A_Index
    }

    backspaces := oldArr.Length - commonLen
    if backspaces > 0
        Send("{Backspace " . backspaces . "}")

    newSuffix := ""
    Loop newArr.Length - commonLen {
        newSuffix .= newArr[commonLen + A_Index]
    }
    if newSuffix !== ""
        SendText(newSuffix)
}

HandleLetter(ch) {
    global g_buffer

    ; Chỉ chạy ở chế độ VN — hotkey chữ được đăng ký dưới HotIf(IsVnMode), ở
    ; chế độ EN chúng không tồn tại nên phím đi thẳng tới ứng dụng.
    ; Caps Lock: hotkey chặn phím gốc rồi phát lại bằng SendText, mà SendText bỏ
    ; qua Caps Lock — nên phải tự lật hoa/thường ở đây (Shift đảo ngược lại).
    ; Ở chế độ EN không cần khối này: Windows tự xử lý vì phím không bị chặn.
    if GetKeyState("CapsLock", "T") {
        cp := Ord(ch)
        if (cp >= 97 && cp <= 122)       ; a-z
            ch := Chr(cp - 32)
        else if (cp >= 65 && cp <= 90)   ; A-Z
            ch := Chr(cp + 32)
    }

    newBuf := Transform(g_buffer, ch)
    SyncToScreen(g_buffer, newBuf)
    g_buffer := newBuf
}

ToggleMode(*) {
    global g_vnMode, g_buffer
    g_vnMode := !g_vnMode
    g_buffer := ""
    UpdateTrayIcon()
    ToolTip(g_vnMode ? "VN: Tieng Viet" : "EN: English")
    SetTimer(() => ToolTip(), -1500)
}

; Điều kiện của HotIf cho các hotkey chữ cái: ở chế độ EN chúng KHÔNG tồn tại,
; phím tới ứng dụng như bàn phím thật (xem lời gọi HotIf(IsVnMode)).
IsVnMode(*) {
    global g_vnMode
    return g_vnMode
}

ResetBuffer(*) {
    global g_buffer
    g_buffer := ""
}

HandleBackspace(*) {
    global g_buffer
    if StrLen(g_buffer) > 0
        g_buffer := SubStr(g_buffer, 1, -1)
}

; Backspace tới từ bàn phím: Ctrl+Backspace xoá CẢ MỘT TỪ (và Alt/Win+Backspace
; là hoàn tác ở nhiều ứng dụng) nên buffer không còn mô hình hoá màn hình ->
; reset. Backspace trần và Shift+Backspace chỉ xoá 1 ký tự -> cắt 1 ký tự.
HandleBackspaceKey(*) {
    if (GetKeyState("Control", "P") || GetKeyState("Alt", "P")
        || GetKeyState("LWin", "P") || GetKeyState("RWin", "P"))
        ResetBuffer()
    else
        HandleBackspace()
}

; Mọi thao tác do CHÍNH SCRIPT gửi đi (dán, copy, ...) đều có thể đổi nội dung
; hoặc vị trí con trỏ, nên g_buffer hết đúng. Gửi qua đây để bất biến
; "g_buffer == N ký tự cuối ngay trước con trỏ" được giữ ở đúng MỘT chỗ.
; Ngoại lệ duy nhất là Send trong SyncToScreen — chính nó tạo ra sự đồng bộ đó.
SendAndResetBuffer(keys) {
    ResetBuffer()
    Send(keys)
}

QuitApp(*) {
    ExitApp()
}

MakeHandler(ch) {
    return (*) => HandleLetter(ch)
}

; ============================================================================
; TRAY ICON — drawn in-memory each toggle so the script ships as a single file
; ============================================================================
; Builds a 32x32 orange square with one centered white letter via GDI, then
; hands the HBITMAP to CreateIconIndirect. Done in-process (rather than two
; pre-baked .ico files) so there's nothing extra to distribute alongside the
; script. Caller is responsible for DestroyIcon on the returned handle.
CreateLetterIcon(letter) {
    size := 32

    screenDC := DllCall("GetDC", "Ptr", 0, "Ptr")
    memDC    := DllCall("CreateCompatibleDC", "Ptr", screenDC, "Ptr")

    ; 32-bit top-down DIB (negative height) so pBits[0] is the top-left pixel.
    bmi := Buffer(40, 0)
    NumPut("UInt",   40,    bmi, 0)
    NumPut("Int",    size,  bmi, 4)
    NumPut("Int",    -size, bmi, 8)
    NumPut("UShort", 1,     bmi, 12)
    NumPut("UShort", 32,    bmi, 14)
    NumPut("UInt",   0,     bmi, 16)   ; BI_RGB

    pBits := 0
    hbm := DllCall("CreateDIBSection", "Ptr", memDC, "Ptr", bmi, "UInt", 0,
                   "Ptr*", &pBits, "Ptr", 0, "UInt", 0, "Ptr")
    oldBM := DllCall("SelectObject", "Ptr", memDC, "Ptr", hbm, "Ptr")

    ; Opaque orange = RGB(255,140,0). DIB memory order is B,G,R,A, so the
    ; little-endian DWORD is 0xFFFF8C00 (A=FF, R=FF, G=8C, B=00).
    Loop size * size
        NumPut("UInt", 0xFFFF8C00, pBits + (A_Index - 1) * 4)

    DllCall("SetBkMode",    "Ptr", memDC, "Int", 1)            ; TRANSPARENT
    DllCall("SetTextColor", "Ptr", memDC, "UInt", 0xFFFFFF)    ; COLORREF white

    ; ANTIALIASED_QUALITY (4), not CLEARTYPE (5) — ClearType leaves colored
    ; subpixel fringes that look wrong against a flat-color background.
    ; Negative cHeight specifies glyph height directly (excludes line leading).
    hFont := DllCall("CreateFontW",
        "Int", -22, "Int", 0, "Int", 0, "Int", 0,
        "Int", 800,             ; FW_EXTRABOLD
        "UInt", 0, "UInt", 0, "UInt", 0,
        "UInt", 0, "UInt", 0, "UInt", 0,
        "UInt", 4,              ; ANTIALIASED_QUALITY
        "UInt", 0,
        "Str", "Arial",
        "Ptr")
    oldFont := DllCall("SelectObject", "Ptr", memDC, "Ptr", hFont, "Ptr")

    rect := Buffer(16, 0)
    NumPut("Int", 0,    rect, 0)
    NumPut("Int", 0,    rect, 4)
    NumPut("Int", size, rect, 8)
    NumPut("Int", size, rect, 12)
    ; DT_CENTER (0x1) | DT_VCENTER (0x4) | DT_SINGLELINE (0x20) = 0x25
    DllCall("DrawTextW", "Ptr", memDC, "Str", letter, "Int", -1, "Ptr", rect, "UInt", 0x25)

    DllCall("SelectObject", "Ptr", memDC, "Ptr", oldFont)
    DllCall("DeleteObject", "Ptr", hFont)

    ; GDI text rendering ignores the alpha channel — text pixels end up with
    ; alpha=0 (fully transparent). Force every pixel back to opaque so the
    ; letter shows up instead of punching a hole through the icon.
    Loop size * size {
        offset := (A_Index - 1) * 4
        NumPut("UInt", NumGet(pBits + offset, "UInt") | 0xFF000000, pBits + offset)
    }

    ; Mask bitmap: phải là toàn 0 (mọi pixel "lấy màu từ bitmap màu", đúng ý vì
    ; bitmap màu tự mang alpha). MSDN nói rõ: CreateBitmap với lpvBits = NULL để
    ; lại nội dung KHÔNG XÁC ĐỊNH — một bit lạc là một lỗ thủng trong suốt trên
    ; icon. Nên cấp phát và zero tường minh thay vì tin vào "chắc là 0".
    ; Mặt nạ 1bpp: mỗi dòng làm tròn lên bội số 2 byte (32 px -> 4 byte/dòng).
    maskStride := ((size + 15) // 16) * 2
    maskBits   := Buffer(maskStride * size, 0)
    hbmMask := DllCall("CreateBitmap", "Int", size, "Int", size,
                       "UInt", 1, "UInt", 1, "Ptr", maskBits, "Ptr")

    ; ICONINFO: { BOOL fIcon; DWORD x,y; HBITMAP mask, color }
    ; On x64 the HBITMAPs are 8-byte aligned, so 4 bytes of padding sit between
    ; yHotspot (ends at offset 12) and hbmMask (starts at offset 16).
    maskOff  := (A_PtrSize == 8) ? 16 : 12
    colorOff := maskOff + A_PtrSize
    iconInfo := Buffer(colorOff + A_PtrSize, 0)
    NumPut("Int",  1,       iconInfo, 0)
    NumPut("UInt", 0,       iconInfo, 4)
    NumPut("UInt", 0,       iconInfo, 8)
    NumPut("Ptr",  hbmMask, iconInfo, maskOff)
    NumPut("Ptr",  hbm,     iconInfo, colorOff)

    hIcon := DllCall("CreateIconIndirect", "Ptr", iconInfo, "Ptr")

    DllCall("SelectObject", "Ptr", memDC, "Ptr", oldBM)
    DllCall("DeleteObject", "Ptr", hbm)
    DllCall("DeleteObject", "Ptr", hbmMask)
    DllCall("DeleteDC",     "Ptr", memDC)
    DllCall("ReleaseDC",    "Ptr", 0, "Ptr", screenDC)

    return hIcon
}

; QUYỀN SỞ HỮU HANDLE (theo tài liệu AHK v2, mục "Image Handles" —
; docs/misc/ImageHandles.htm):
;   "HICON:" không dấu sao -> AutoHotkey NHẬN quyền sở hữu và sẽ tự DestroyIcon.
;                             Script gọi DestroyIcon nữa là DOUBLE-FREE.
;   "HICON:*" có dấu sao   -> AutoHotkey CHỤP MỘT BẢN SAO; handle gốc vẫn thuộc
;                             về script, script phải tự huỷ.
; Chọn dạng có dấu sao: quyền sở hữu rõ ràng, huỷ ngay tại chỗ, không cần biến
; toàn cục nhớ handle cũ nữa (biến g_currentIcon trước đây — đã bỏ — chỉ tồn tại
; để phục vụ đúng cái cách huỷ sai đó).
UpdateTrayIcon() {
    global g_vnMode, g_iconErrorShown
    newIcon := 0
    try {
        newIcon := CreateLetterIcon(g_vnMode ? "V" : "E")
        if !newIcon
            throw Error("CreateIconIndirect trả về handle rỗng")
        TraySetIcon("HICON:*" newIcon)
    } catch as err {
        ; KHÔNG nuốt lỗi: giữ icon mặc định (script vẫn gõ được), và báo cho
        ; người dùng. Hàm này chạy mỗi lần đổi chế độ nên chỉ báo MỘT lần —
        ; lỗi GDI thường là lỗi dai dẳng, báo mỗi lần toggle là spam.
        if !g_iconErrorShown {
            g_iconErrorShown := true
            TrayTip("Không vẽ được icon khay, dùng icon mặc định.`n" . err.Message,
                    "VietDvorak")
        }
    }
    ; Huỷ handle của mình trong MỌI trường hợp — kể cả khi TraySetIcon ném lỗi
    ; sau khi CreateLetterIcon đã cấp phát (nếu không thì rò rỉ GDI handle).
    if newIcon
        DllCall("DestroyIcon", "Ptr", newIcon)
}

; Đệm khoảng trắng cho thẳng cột trong font monospace của hộp thoại.
PadRight(s, width) {
    out := s
    while StrLen(out) < width
        out .= " "
    return out
}

; Dựng phần "Phím dấu thanh" THẲNG từ TONE_KEYS — hộp thoại không giữ bản sao
; của bảng phím. Duyệt theo chỉ số thanh 1..5 (chứ không theo thứ tự khoá của
; Map, vốn được sắp xếp) để giữ đúng thứ tự quen thuộc sắc-huyền-hỏi-ngã-nặng.
BuildToneHelp() {
    global TONE_KEYS, TONE_NAMES, VOWELS
    out := ""
    for idx, name in TONE_NAMES {
        for key, toneIdx in TONE_KEYS {
            if (toneIdx !== idx)
                continue
            out .= key . "  =  " . PadRight(name, 8) . "(" . VOWELS["a"][idx + 1] . ")`n"
        }
    }
    return out
}

; Dựng phần "Chữ có dấu" THẲNG từ DIACRITIC_PAIRS. Ba nhóm dòng cũng được SUY
; RA, không liệt kê tay: (1) nhân đôi nguyên âm, (2) nguyên âm + w, (3) phụ âm.
BuildPairHelp() {
    global DIACRITIC_PAIRS
    rows := ["", "", ""]
    for pair, result in DIACRITIC_PAIRS {
        first  := SubStr(pair, 1, 1)
        second := SubStr(pair, 2, 1)
        if !IsVowelChar(result)
            grp := 3                      ; dd = đ
        else if (second == first)
            grp := 1                      ; aa / ee / oo
        else
            grp := 2                      ; aw / ow / uw
        rows[grp] .= (rows[grp] == "" ? "" : "      ") . pair . " = " . result
    }
    out := ""
    for r in rows {
        if r !== ""
            out .= r . "`n"
    }
    return out
}

; Help dialog wired to the tray icon (default action on double-click, plus the
; first entry in the right-click menu). Section headers in orange to match
; the icon's brand color; body in a monospace font so the example mappings
; line up vertically.
ShowHelp(*) {
    global g_helpGui
    if g_helpGui {
        try {
            g_helpGui.Show()
            return
        }
        catch {
            ; Cửa sổ đã bị huỷ (hiếm) — dựng lại bên dưới. Bắt tường minh để
            ; ý định "rơi xuống dựng lại" là cố ý, không phải nuốt lỗi.
            g_helpGui := 0
        }
    }
    g_helpGui := Gui("+AlwaysOnTop +ToolWindow -MinimizeBox",
                     "VietDvorak for Windows — Phím tắt && cách gõ")
    g_helpGui.MarginX := 20
    g_helpGui.MarginY := 16
    g_helpGui.BackColor := "FFFFFF"

    sections := [
        ["Chuyển chế độ  EN ↔ VN",
            "Double-tap CapsLock`n"
          . "Ctrl + Shift + V    (dự phòng)"],
        ["Phím tắt thao tác",
            "Double-tap Left Ctrl     →  Copy   (Ctrl + C)`n"
          . "Double-tap Left Shift    →  Paste  (Ctrl + V)`n"
          . "Ctrl + Shift + Q         →  Thoát script"],
        ["Phím dấu thanh (Telex)",
            BuildToneHelp()
          . "Gõ phím dấu 2 lần để hủy — vd:  ass → as"],
        ["Chữ có dấu",
            BuildPairHelp()
          . "w  = ư   (đầu từ hoặc sau phụ âm — kiểu Unikey)`n"
          . "Gõ phím lần 3 để hủy — vd:  aaa → aa,  oww → ow"],
        ["Đặt dấu thông minh",
            "tieengs   →  tiếng`n"
          . "ngoaif    →  ngoài`n"
          . "soajn     →  soạn`n"
          . "Quynhf    →  Quỳnh`n"
          . "hoaf      →  hoà    (oa, oe, uy — kiểu mới)"]
    ]

    for s in sections {
        g_helpGui.SetFont("s10 Bold cFF8C00", "Segoe UI")
        g_helpGui.AddText("y+12", s[1])
        g_helpGui.SetFont("s10 Norm c333333", "Consolas")
        g_helpGui.AddText("y+4 x40", s[2])
    }

    ; Hide (not Destroy) on close so the next open is instant and keeps position.
    g_helpGui.OnEvent("Close",  (*) => g_helpGui.Hide())
    g_helpGui.OnEvent("Escape", (*) => g_helpGui.Hide())
    g_helpGui.Show()
}

; ============================================================================
; HOTKEY REGISTRATION
; ============================================================================

; Các hotkey ĐIỀU KHIỂN dưới đây đăng ký ngoài mọi HotIf nên chạy ở CẢ HAI chế
; độ — kể cả khi đang ở EN, nếu không thì bật lại chế độ VN sẽ không còn cách.
Hotkey("^+v", ToggleMode)   ; backup: Ctrl+Shift+V toggles
Hotkey("^+q", QuitApp)

; ----------------------------------------------------------------------------
; DOUBLE-TAP gestures: tap a key twice within DOUBLE_TAP_MS to fire its action.
; A tap only counts when A_PriorKey is the key itself — i.e. nothing was
; pressed between its down and up. So a modifier held while typing a capital,
; or used in a Ctrl+letter shortcut, does NOT count (A_PriorKey is that letter).
; ----------------------------------------------------------------------------

RegisterDoubleTap(keyName, action) {
    global DOUBLE_TAP_MS
    ; Chép vào biến cục bộ để handler bắt được như closure (giống lastTap).
    windowMs := DOUBLE_TAP_MS
    lastTap := 0
    handler(*) {
        if (A_PriorKey !== keyName) {
            lastTap := 0
            return
        }
        now := A_TickCount
        if (lastTap > 0 && (now - lastTap) < windowMs) {
            lastTap := 0
            action()
        } else {
            lastTap := now
        }
    }
    Hotkey("~" keyName " up", handler)
}

RegisterDoubleTap("CapsLock", ToggleMode)
RegisterDoubleTap("LControl", () => SendAndResetBuffer("^c"))
RegisterDoubleTap("LShift",   () => SendAndResetBuffer("^v"))

; ----------------------------------------------------------------------------
; KẾT THÚC ÂM TIẾT — một QUY TẮC, không phải một danh sách phím
; ----------------------------------------------------------------------------
; g_buffer mô hình hoá N ký tự cuối ngay trước con trỏ. QUY TẮC DUY NHẤT:
;
;     Âm tiết chỉ SỐNG SÓT qua ba thứ — một phím chữ cái gõ trần, một phím chữ
;     cái kèm Shift (chính là chữ hoa), và bản thân phím Shift. MỌI sự kiện
;     phím khác đều KẾT THÚC âm tiết.
;
; Vì mọi phím khác hoặc chèn ký tự mà script không biết, hoặc dời con trỏ đi
; chỗ khác — cả hai đều làm buffer sai lệch, và lần SyncToScreen kế tiếp sẽ gửi
; backspace ăn vào chữ chung quanh. Ví dụ kinh điển: gõ "tri.f" — nếu "." không
; reset thì buffer vẫn là "tri", "f" thành dấu huyền -> buffer "trì" nhưng màn
; hình "tri." nhận 1 backspace + "ì" -> ra "triì".
;
; Bản cũ LIỆT KÊ TAY các phím này. Danh sách kiểu đó luôn thiếu một mục, và
; thực tế đã thiếu: dấu huyền `, toàn bộ Numpad0-9 / NumpadDot / NumpadAdd...,
; mọi biến thể khi tắt NumLock (NumpadLeft dời con trỏ!), F13-F24, phím media.
; Nên ở đây quét CẢ KHÔNG GIAN virtual-key, chỉ chừa lại đúng những gì quy tắc
; trên cho phép. ~* để phím vẫn tới ứng dụng y như cũ.
;
; Lưu ý Ctrl/Alt/Win KHÔNG được chừa: chúng không bao giờ tham gia tạo ra một
; chữ cái tiếng Việt, nên chỉ cần chúng được NHẤN là âm tiết kết thúc. Nhờ vậy
; Ctrl+Z / Ctrl+A / Ctrl+X / Ctrl+V (và cả Ctrl+click, Alt+Tab, Win+bất kỳ...)
; đều tự động reset mà không phải đăng ký 26 hotkey "Ctrl+chữ" — vốn sẽ tranh
; chấp với chính "^+v"/"^+q" của script, mà tài liệu AHK không quy định hotkey
; nào thắng khi "^+v" và "*^v" cùng tồn tại.
;
; (Đã cân nhắc InputHook + KeyOpt("{All}","N") làm một chỗ quan sát duy nhất và
;  BÁC BỎ: theo tài liệu AHK v2, MinSendLevel mặc định 0 nên InputHook thu cả
;  input do CHÍNH SCRIPT sinh ra, mà mỗi phím chữ script đều gửi {Backspace} +
;  SendText — InputHook sẽ tự reset buffer ngay sau khi vừa đặt. MinSendLevel
;  lọc theo chiều ngược lại nên không tách được. Hotkey thì mặc định KHÔNG bị
;  kích hoạt bởi input do script tự gửi, đúng thứ ta cần.)

VK_FIRST := 0x08          ; Backspace — dưới mức này là các mã nút chuột
VK_LAST  := 0xFE          ; 0xFF không phải phím thật (AHK dùng nội bộ)
VK_LETTER_FIRST := 0x41   ; A
VK_LETTER_LAST  := 0x5A   ; Z
VK_KEEPS_SYLLABLE := Map(
    0x08, true,       ; Backspace — lùi 1 ký tự, xử lý riêng bên dưới
    0x10, true,       ; Shift (mã ảo) — Shift+chữ vẫn là một phím chữ cái
    0xA0, true,       ; LShift        —   (chữ HOA), nên buffer phải sống tiếp
    0xA1, true,       ; RShift
    0xE7, true        ; VK_PACKET — không phải phím vật lý; chính AHK dùng mã
)                     ;   này để gửi Unicode

; QUY TẮC ở trên, viết thành MỘT HÀM THUẦN: vào một mã VK, ra true/false, không
; đọc gì ngoài hai bảng dữ liệu ngay trên, không tác dụng phụ. Vòng lặp bên dưới
; chỉ còn là phần CÀI ĐẶT quy tắc (đăng ký hotkey), không còn tự phát biểu lại
; quy tắc nữa.
;
; Tách ra vì vòng lặp đăng ký KHÔNG BAO GIỜ chạy được dưới test: gọi nó là dựng
; một bàn phím hook thật trên máy người đang gõ. Trước đây mọi harness kiểm tra
; đều phải CHÉP TAY lại quy tắc này, nên sửa luồng điều khiển của vòng lặp (thêm
; một `continue`, đảo hai nhánh) vẫn qua được mọi phép kiểm — đúng cái lỗi "hai
; bản, đồng bộ bằng tay" mà cả dự án này đang dẹp. Hàm thuần thì cắt ra và GỌI
; được: check-parity.mjs quét VkEndsSyllable() suốt dải VK và so với quy tắc bên
; JS, nên không còn bản chép tay nào nữa.
VkEndsSyllable(vk) {
    global VK_LETTER_FIRST, VK_LETTER_LAST, VK_KEEPS_SYLLABLE
    if (vk >= VK_LETTER_FIRST && vk <= VK_LETTER_LAST)
        return false            ; chữ cái -> HandleLetter, giữ buffer
    if VK_KEEPS_SYLLABLE.Has(vk)
        return false
    return true
}

; Dải VK là THƯA: Windows để trống / dành riêng khá nhiều mã trong đó. Nếu AHK
; từ chối một mã như vậy thì không có phím nào phía sau để móc — đó là kết quả
; bình thường của việc quét một không gian thưa, không phải lỗi bị nuốt. Vẫn
; ĐẾM lại và treo con số lên tooltip khay (thụ động, không popup): nếu một ngày
; nào đó con số nhảy vọt thì giả định về không gian VK đã sai và có chỗ để lần.
vkSkipped := 0
Loop (VK_LAST - VK_FIRST + 1) {
    vk := VK_FIRST + A_Index - 1
    if !VkEndsSyllable(vk)
        continue
    try {
        Hotkey(Format("~*vk{:02X}", vk), ResetBuffer)
    } catch {
        vkSkipped += 1
    }
}
if vkSkipped > 0
    A_IconTip := TRAY_TIP_BASE . " [" . vkSkipped . " VK bỏ qua]"

; Chuột không nằm trong không gian VK bàn phím nên đăng ký theo tên. Bấm chuột
; là đặt con trỏ sang chỗ khác -> kết thúc âm tiết.
for btn in ["LButton", "RButton", "MButton", "XButton1", "XButton2"]
    Hotkey("~*" btn, ResetBuffer)

; Backspace: một hotkey duy nhất, phân biệt bên trong HandleBackspaceKey
; (Ctrl+Backspace xoá cả từ -> reset; Backspace/Shift+Backspace -> lùi 1 ký tự).
Hotkey("~*Backspace", HandleBackspaceKey)

; ----------------------------------------------------------------------------
; PHÍM CHỮ CÁI — chỉ tồn tại ở chế độ VN
; ----------------------------------------------------------------------------
; HotIf: ở chế độ EN các hotkey này KHÔNG khớp, nên phím không bị chặn và đi
; thẳng tới ứng dụng như bàn phím thật. Trước đây chế độ EN vẫn chặn phím rồi
; phát lại bằng SendText — tức là gửi VK_PACKET, thứ mà game DirectInput,
; RDP/Citrix, console máy ảo và các IME Trung/Nhật/Hàn đều bỏ qua: gõ "chết"
; mà tắt chế độ VN cũng không cứu được. Chế độ EN phải là passthrough THẬT.
HotIf(IsVnMode)
Loop 26 {
    lower := Chr(96 + A_Index)
    upper := Chr(64 + A_Index)
    Hotkey(lower,      MakeHandler(lower))
    Hotkey("+" lower,  MakeHandler(upper))
}
HotIf()   ; trả ngữ cảnh về mặc định cho mọi Hotkey() đăng ký sau này

; Tray menu: put the help dialog at the top and make it the default action,
; so a double-left-click on the icon opens it (right-click still shows the
; standard AHK menu underneath). "&&" displays a literal "&" in the label.
A_TrayMenu.Insert("1&", "Phím tắt && cách gõ", ShowHelp)
A_TrayMenu.Default := "Phím tắt && cách gõ"

UpdateTrayIcon()

; Startup notice
ToolTip("VietDvorak loaded - double-tap CapsLock=toggle EN/VN, LCtrl=copy, LShift=paste; Ctrl+Shift+Q=quit")
SetTimer(() => ToolTip(), -2500)
