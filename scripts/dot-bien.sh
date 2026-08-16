#!/usr/bin/env bash
# dot-bien.sh — áp một đột biến vào source, chạy một lệnh kiểm, rồi hoàn nguyên.
#
# 281 vector conformance chỉ có giá trị nếu chúng ĐỎ khi engine sai. Chạy xanh
# không chứng minh điều đó — nó chỉ chứng minh engine khớp spec ở những chỗ spec
# nói tới. Script này trả lời câu còn lại: bộ vector có RĂNG không.
#
#     bash scripts/dot-bien.sh <file> '<chuỗi cũ>' '<chuỗi mới>' <lệnh kiểm...>
#
# Ví dụ:
#     bash scripts/dot-bien.sh crates/vietdvorak-core/src/tone.rs \
#          'TONE_PLACEMENT_MODERN && is_glide_pair' 'false && is_glide_pair' \
#          node check-parity.mjs
#
# BA LUẬT ĐỌC KẾT QUẢ (mượn từ nightly/scripts/dot-bien.sh, nơi mỗi luật là một
# lần đã trả giá — ở đây chưa trả giá lần nào, nên càng phải đọc trước):
#
#   1. ĐỎ chỉ chứng minh "có thứ gì đó bắt được đột biến NÀY". ĐỌC TÊN thứ đỏ.
#      Đỏ nhầm chỗ là dương tính giả — một đột biến làm crash cả binary thì mọi
#      thứ đều đỏ và nó không nói gì về bộ vector.
#   2. Đột biến phải giữ chương trình COMPILE ĐƯỢC. Lỗi biên dịch không phải
#      "bộ test bắt được"; nó là "chưa có bài test nào chạy".
#   3. "0 bài chạy" KHÔNG phải "không có răng". Script này tự từ chối kết luận ở
#      ca đó thay vì để người đọc suy ra nhầm.
#
# Hoàn nguyên bằng `trap` nên đúng byte kể cả khi Ctrl-C hoặc lệnh kiểm chết.

set -uo pipefail

if [ "$#" -lt 4 ]; then
  echo "dùng: bash scripts/dot-bien.sh <file> '<cũ>' '<mới>' <lệnh kiểm...>" >&2
  exit 2
fi

FILE="$1"; OLD="$2"; NEW="$3"; shift 3

[ -f "$FILE" ] || { echo "không thấy file: $FILE" >&2; exit 2; }

BACKUP="$(mktemp)"
cp -- "$FILE" "$BACKUP"
restore() { cp -- "$BACKUP" "$FILE"; rm -f -- "$BACKUP"; }
trap restore EXIT INT TERM

# LUẬT 4 — chuỗi NHIỀU DÒNG bị từ chối thẳng, không cố xử lý.
#
# Đây là một cái bẫy đã cắn ngay lần chạy đầu tiên của chính script này
# (2026-08-16). `awk` và `grep` đều xử lý theo TỪNG DÒNG, nên một chuỗi nhiều
# dòng không bao giờ thay được — nhưng `grep -F` với pattern nhiều dòng lại coi
# mỗi dòng là một lựa chọn RIÊNG, nên nó vẫn đếm ra > 0. Kết quả: script báo
# "ĐỘT BIẾN SỐNG SÓT" cho một đột biến **chưa bao giờ được áp**, tức một dương
# tính giả nói rằng bộ test không có răng trong khi chưa ai thử răng nó.
#
# Sai lầm đó nguy hiểm hơn mọi thứ script này định phát hiện, nên nó bị chặn ở
# đây thay vì được xử lý cho tiện. Muốn đột biến một khối nhiều dòng → chọn một
# chuỗi MỘT DÒNG bên trong khối đó.
case "$OLD$NEW" in
  *$'\n'*)
    echo "TỪ CHỐI: chuỗi nhiều dòng." >&2
    echo "  awk/grep xử lý theo dòng ⇒ phép thay im lặng không xảy ra, mà bộ đếm" >&2
    echo "  vẫn báo tìm thấy ⇒ kết quả 'sống sót' là dương tính giả." >&2
    echo "  Hãy chọn một chuỗi MỘT DÒNG nằm trong khối cần đột biến." >&2
    exit 2
    ;;
esac

# Đếm trước khi thay: 0 lần khớp nghĩa là đột biến chưa bao giờ được áp, và một
# lần chạy "xanh" sau đó sẽ bị đọc nhầm thành "bộ test không có răng".
BEFORE="$(grep -c -F -- "$OLD" "$FILE" || true)"
if [ "$BEFORE" -eq 0 ]; then
  echo "ĐỘT BIẾN KHÔNG ÁP ĐƯỢC: không tìm thấy chuỗi trong $FILE" >&2
  echo "  cần khớp: $OLD" >&2
  exit 2
fi

# Chứng minh phép thay ĐÃ xảy ra, không tin vào bộ đếm trước.
verify_applied() {
  if ! cmp -s -- "$BACKUP" "$FILE"; then return 0; fi
  echo "ĐỘT BIẾN KHÔNG ÁP ĐƯỢC: file không đổi sau khi thay." >&2
  exit 2
}

python_free_replace() {
  # Thay bằng awk để không phụ thuộc regex escaping của sed — chuỗi Rust hay có
  # ký tự sed coi là siêu ký tự.
  awk -v old="$OLD" -v new="$NEW" '
    { line = $0; out = ""
      while ((i = index(line, old)) > 0) {
        out = out substr(line, 1, i-1) new
        line = substr(line, i + length(old))
      }
      print out line
    }' "$BACKUP" > "$FILE"
}
python_free_replace
verify_applied

echo "=== ĐỘT BIẾN ==="
echo "  file : $FILE  ($BEFORE chỗ khớp)"
echo "  cũ   : $OLD"
echo "  mới  : $NEW"
echo "  kiểm : $*"
echo

"$@"
STATUS=$?

echo
if [ "$STATUS" -eq 0 ]; then
  echo "=== XANH (exit 0) — ĐỘT BIẾN SỐNG SÓT ==="
  echo "  Bộ kiểm KHÔNG bắt được thay đổi này. Hoặc thiếu vector, hoặc chỗ vừa"
  echo "  đột biến là code chết. Cả hai đều đáng biết."
else
  echo "=== ĐỎ (exit $STATUS) — ĐỘT BIẾN BỊ BẮT ==="
  echo "  ĐỌC TÊN thứ vừa đỏ trước khi kết luận (luật 1 ở đầu file). Đỏ vì lỗi"
  echo "  biên dịch (luật 2) hoặc vì 0 bài chạy (luật 3) thì KHÔNG tính."
fi
echo "Đã hoàn nguyên $FILE."
exit "$STATUS"
