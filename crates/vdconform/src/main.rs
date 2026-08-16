//! Chạy vector conformance qua `vietdvorak-core` và in kết quả — không phán xét.
//!
//! Tồn tại để `vietdvorak-core` thành ENGINE THỨ BA trong `check-parity.mjs`,
//! cạnh `Transform()` của AHK và bản mirror JS trong `practice.html`. Nó cố ý
//! KHÔNG đọc `telex-conformance.txt`, KHÔNG biết `expected` là gì và KHÔNG bao
//! giờ quyết định pass/fail.
//!
//! Lý do là single source of truth: spec đã có đúng một parser và đúng một
//! verdict, cả hai nằm trong `check-parity.mjs`. Thêm một `tests/conformance.rs`
//! tự parse lại file `.txt` là tạo parser thứ hai và verdict thứ hai cho cùng
//! một spec — hai thứ sẽ trôi khỏi nhau, và khi chúng bất đồng thì không ai
//! biết bên nào đúng. Một harness, một exit code.
//!
//! # Giao thức
//!
//! stdin: mỗi dòng `section<TAB>input`. stdout: đúng MỘT dòng kết quả cho mỗi
//! dòng vào, cùng thứ tự. Chỉ vậy. Đây đúng là chuỗi mà `check-parity.mjs` đã
//! dựng sẵn cho arm AHK (`runAhkVectors` ghi ra `vectors.tsv` cùng định dạng),
//! nên arm thứ ba tái dùng luôn hình dạng đó thay vì phát minh cái mới.
//!
//! Hai section, đúng như `telex-conformance.txt` định nghĩa:
//!
//! - `transform` — `input` là chuỗi phím CHỮ CÁI vào buffer RỖNG. Từng ký tự
//!   qua `transform()`; kết quả là buffer sau ký tự cuối.
//! - `integration` — `input` có thể chứa ký tự BẤT KỲ. Từng ký tự qua đường
//!   apply (hỏi luật ngắt âm tiết trước, rồi mới `transform()`); kết quả là
//!   committed + buffer sau ký tự cuối.
//!
//! Lỗi giao thức (section lạ, thiếu TAB) là lỗi CỦA HARNESS, không phải bất
//! đồng của engine — nên nó ra stderr và exit khác 0, chứ không lặng lẽ thành
//! một dòng kết quả sai mà bên kia sẽ báo cáo nhầm thành engine drift.

use std::io::{self, BufWriter, Read, Write};
use std::process::ExitCode;

use vietdvorak_core::{Session, transform};

/// Chạy `input` như một chuỗi phím chữ cái vào buffer rỗng, qua `transform` trần.
fn run_transform(input: &str) -> String {
    let mut buf: Vec<char> = Vec::new();
    for key in input.chars() {
        buf = transform(&buf, key);
    }
    buf.into_iter().collect()
}

/// Chạy `input` qua đường apply đầy đủ; kết quả là committed + buffer.
fn run_integration(input: &str) -> String {
    let mut session = Session::new();
    let mut out = String::new();
    for key in input.chars() {
        let view = session.feed(key);
        out.push_str(&view.committed);
    }
    out.push_str(&session.composing());
    out
}

fn main() -> ExitCode {
    let mut src = String::new();
    if let Err(err) = io::stdin().read_to_string(&mut src) {
        let _ = writeln!(io::stderr(), "vdconform: không đọc được stdin: {err}");
        return ExitCode::FAILURE;
    }

    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    // Bỏ dòng rỗng cuối do trailing newline, KHÔNG bỏ dòng rỗng ở giữa: một
    // dòng rỗng giữa chừng nghĩa là harness gửi sai số lượng vector, và bên kia
    // đếm dòng để phát hiện đúng chuyện đó.
    let body = src.strip_suffix('\n').unwrap_or(&src);
    let body = body.strip_suffix('\r').unwrap_or(body);
    if body.is_empty() {
        return ExitCode::SUCCESS;
    }

    for (n, line) in body.split('\n').enumerate() {
        let line = line.strip_suffix('\r').unwrap_or(line);
        let Some((section, input)) = line.split_once('\t') else {
            let _ = writeln!(
                io::stderr(),
                "vdconform: dòng {} thiếu TAB phân tách: {line:?}",
                n.saturating_add(1)
            );
            return ExitCode::FAILURE;
        };

        let result = match section {
            "transform" => run_transform(input),
            "integration" => run_integration(input),
            other => {
                let _ = writeln!(
                    io::stderr(),
                    "vdconform: dòng {} có section lạ {other:?} — \
                     lõi này chỉ biết \"transform\" và \"integration\"",
                    n.saturating_add(1)
                );
                return ExitCode::FAILURE;
            }
        };

        if let Err(err) = writeln!(out, "{result}") {
            let _ = writeln!(io::stderr(), "vdconform: không ghi được stdout: {err}");
            return ExitCode::FAILURE;
        }
    }

    if let Err(err) = out.flush() {
        let _ = writeln!(io::stderr(), "vdconform: không flush được stdout: {err}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test ở đây pin GIAO THỨC, không pin hành vi Telex — hành vi Telex sống
    // trong telex-conformance.txt và được phán xét bởi check-parity.mjs.

    #[test]
    fn transform_feeds_every_char_into_an_empty_buffer() {
        assert_eq!(run_transform(""), "");
        assert_eq!(run_transform("abc"), "abc");
    }

    #[test]
    fn integration_keeps_break_chars_in_the_output() {
        // Dấu cách kết thúc âm tiết và được chốt nguyên văn — nó phải CÓ MẶT
        // trong kết quả, đúng như "xin chaof" -> "xin chào" giữ dấu cách.
        assert_eq!(run_integration("ab cd"), "ab cd");
        assert_eq!(run_integration("a-b"), "a-b");
    }

    #[test]
    fn integration_ends_with_the_uncommitted_syllable() {
        assert_eq!(run_integration("ab "), "ab ");
        assert_eq!(run_integration("ab"), "ab");
    }
}
