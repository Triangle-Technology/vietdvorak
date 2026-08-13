// Bigram cost for LATE-TONE typing style — the model the shipped layout rests on.
//
// User types the whole syllable, then the tone key at the end.
// So the bigram that matters is: (last char of syllable) -> (tone key)
//
// For closed syllables (with coda consonant), last char = coda
// For open syllables, last char = last vowel
//
// Vietnamese codas: c, ch, m, n, ng, nh, p, t
// On Dvorak:
//   c, m, n, t, g, h  -> RIGHT hand
//   p                  -> LEFT hand
// All vowels                  -> LEFT hand
//
// So most closed syllables end with a RIGHT-hand char in Dvorak.
// → tone on RIGHT (s, f, r, z, b) becomes same-hand
// → tone on LEFT  (x, j)          becomes alternation
//
// This is the premise VietDvorak.ahk's header cites, and the reason the shipped
// layout keeps ngã=x / nặng=j on the left hand. bigram-analysis.mjs models the
// EARLY-tone style instead and therefore reaches the opposite verdict — that is
// expected; see the scope warning at the top of that file.
//
// Corpus and helpers: viet-corpus.mjs.

import {
  VIET_SYLLABLES, detectTone, stripSyllable, isLeft, isRight, NO_TONE,
  TONE_KEYS_SHIPPED, LABEL_SHIPPED,
  TONE_KEYS_ALL_RIGHT_VARIANT, LABEL_ALL_RIGHT_VARIANT,
} from './viet-corpus.mjs';

// ---------------------------------------------------------------------------
// THE sample. Defined once, used by every line this script prints.
// ---------------------------------------------------------------------------
//
// A syllable needs a tone key iff it carries a tone. The sample is drawn from
// VIET_SYLLABLES — the vowel-bearing tokens, filtered ONCE in viet-corpus.mjs —
// so this script's denominator is the same list tone-stats.mjs and
// bigram-analysis.mjs count from. (Filtering here is a no-op for the toned
// subset, since detectTone() cannot return a tone without a toned vowel; it
// matters for the "of N tokens" figure printed below, which used to be the raw
// token count while the siblings used the filtered one.)
//
// There used to be one here, written over the UNTONED vowel letters only
// (/[aăâeêioôơuưy]/). Since the toned vowel never matches that class, it did not
// test "has a vowel" at all — it tested "has a SECOND, untoned vowel", and threw
// away 304 of 510 toned syllables (59.6%), every one of them a single-vowel word
// (là, đủ, một, ở, xã, ...). That biased this file's headline metric: it read the
// left-hand share of the last-char-before-tone as 63.6% when the true figure over
// all toned syllables is 54.1%. The ranking of the layouts survived the
// correction — see the note printed at the end — but the percentages did not.
const TONED = VIET_SYLLABLES.filter(syl => detectTone(syl) !== NO_TONE);

// ---------------------------------------------------------------------------
// Layouts to compare
// ---------------------------------------------------------------------------
// The two real candidates come from viet-corpus.mjs so they cannot drift away
// from what the .ahk actually maps. The all-left row is a pure thought
// experiment and stays local to this script.
const LAYOUTS = [
  [LABEL_SHIPPED,             TONE_KEYS_SHIPPED],
  [LABEL_ALL_RIGHT_VARIANT,   TONE_KEYS_ALL_RIGHT_VARIANT],
  ['All-left tone keys       [hypothetical — not a real candidate]',
   { 'sắc': 'k', 'huyền': 'q', 'hỏi': ';', 'ngã': 'x', 'nặng': 'j' }],
];

// For LATE-TONE typing the typed sequence is (syllable letters)(tone key), so the
// char before the tone is the last letter the typist actually pressed — i.e. the
// syllable stripped back to its base letters.
function lastTypedCharBeforeTone(syl) {
  const base = stripSyllable(syl);
  return base[base.length - 1];
}

// For EARLY-TONE typing the tone lands right after the vowel that carries it.
function vowelBeforeTone(syl) {
  for (const c of syl) {
    const stripped = stripSyllable(c);
    if ('aeiouy'.includes(stripped)) return stripped;
  }
  // Unreachable: every syllable here is toned, so it has a vowel. Fail loudly
  // rather than scoring an invented 'a'.
  throw new Error(`toned syllable with no vowel: ${syl}`);
}

function analyze(layout, label, lateTone) {
  let altCount = 0, sameCount = 0;
  for (const syl of TONED) {
    const toneKey = layout[detectTone(syl)];
    const prev = lateTone ? lastTypedCharBeforeTone(syl) : vowelBeforeTone(syl);
    if ((isLeft(prev) && isRight(toneKey)) || (isRight(prev) && isLeft(toneKey))) altCount++;
    else sameCount++;
  }
  // Print the denominator with the figure, always. This script divides by the
  // TONED syllables while bigram-analysis.mjs divides by ALL of them, so a bare
  // percentage from either file reads as contradicting the other.
  const altPct  = (altCount  / TONED.length * 100).toFixed(1);
  const samePct = (sameCount / TONED.length * 100).toFixed(1);
  console.log(`  alt=${altPct.padStart(5)}%  same=${samePct.padStart(5)}%  of the ${TONED.length} toned   ${label}`);
  return altCount / TONED.length * 100;
}

console.log(`Toned syllables: ${TONED.length} of ${VIET_SYLLABLES.length} syllables ` +
            `(${(TONED.length / VIET_SYLLABLES.length * 100).toFixed(1)}%)`);
console.log('(this is the sample every percentage below is computed from)');

console.log();
console.log('--- LATE TONE STYLE (gõ hết âm tiết rồi mới tone) ---');
const lateAlt = {};
for (const [label, layout] of LAYOUTS) lateAlt[label] = analyze(layout, label, true);

console.log();
console.log('--- EARLY TONE STYLE (gõ tone ngay sau vowel) ---');
// The [SHIPPED] row below loses badly here, and the line carries the full
// [SHIPPED] label — so the caveat has to be ON SCREEN, not just in this comment.
// Same banner and same wording as bigram-analysis.mjs, which models this style
// throughout.
console.log('!!! EARLY-TONE MODEL ONLY — assumes the key before the tone is always a');
console.log('!!! left-hand vowel, i.e. ignores codas entirely. This model cannot test');
console.log('!!! the late-tone premise the shipped layout rests on; by construction it');
console.log('!!! always prefers right-hand tone keys. These rows are the counterfactual,');
console.log('!!! not the layout decision — do not quote them on their own.');
for (const [label, layout] of LAYOUTS) analyze(layout, label, false);

// ---------------------------------------------------------------------------
// Per-tone breakdown for late-tone style
// ---------------------------------------------------------------------------
console.log();
console.log('--- LATE TONE: bigram analysis per tone ---');
console.log('(showing what "last char before tone" looks like for each tone;');
console.log(' left=/right= are shares of that row\'s own n, printed on the row)');
const perTone = {};
for (const syl of TONED) {
  const tone = detectTone(syl);
  const last = lastTypedCharBeforeTone(syl);
  if (!perTone[tone]) perTone[tone] = { left: 0, right: 0, total: 0, lastChars: {} };
  perTone[tone].total++;
  if (isLeft(last))  perTone[tone].left++;
  if (isRight(last)) perTone[tone].right++;
  perTone[tone].lastChars[last] = (perTone[tone].lastChars[last] || 0) + 1;
}
let allLeft = 0, allRight = 0;
for (const [tone, info] of Object.entries(perTone)) {
  allLeft += info.left; allRight += info.right;
  const leftPct  = (info.left  / info.total * 100).toFixed(0);
  const rightPct = (info.right / info.total * 100).toFixed(0);
  const topChars = Object.entries(info.lastChars).sort((a, b) => b[1] - a[1]).slice(0, 5)
                    .map(([c, n]) => `${c}=${n}`).join(' ');
  console.log(`  ${tone.padEnd(8)} n=${String(info.total).padStart(3)}  left=${leftPct.padStart(2)}% right=${rightPct.padStart(2)}%  top: ${topChars}`);
}
console.log(`  ${'ALL'.padEnd(8)} n=${String(TONED.length).padStart(3)}  ` +
            `left=${(allLeft / TONED.length * 100).toFixed(0)}% right=${(allRight / TONED.length * 100).toFixed(0)}%`);

// ---------------------------------------------------------------------------
console.log();
console.log('--- VERDICT (late-tone style) ---');
const shippedAlt = lateAlt[LABEL_SHIPPED];
const variantAlt = lateAlt[LABEL_ALL_RIGHT_VARIANT];
console.log(`  shipped x/j alternates ${shippedAlt.toFixed(1)}% vs z/b variant ${variantAlt.toFixed(1)}%`);
console.log(`  (both out of the ${TONED.length} toned syllables — the only ones that cost a tone key)`);
console.log(`  → shipped layout wins by ${(shippedAlt - variantAlt).toFixed(1)}pp.`);

// Which of the two moved keys actually earns that win? Score the two hybrids.
console.log();
console.log('  Decomposition — moving one tone key at a time off the right hand:');
const base = { 'sắc': 's', 'huyền': 'f', 'hỏi': 'r' };
for (const [what, keys] of [
  ['ngã=z nặng=b  (neither moved)', { 'ngã': 'z', 'nặng': 'b' }],
  ['ngã=z nặng=j  (nặng only)    ', { 'ngã': 'z', 'nặng': 'j' }],
  ['ngã=x nặng=b  (ngã only)     ', { 'ngã': 'x', 'nặng': 'b' }],
  ['ngã=x nặng=j  (SHIPPED)      ', { 'ngã': 'x', 'nặng': 'j' }],
]) {
  let a = 0;
  for (const syl of TONED) {
    const prev = lastTypedCharBeforeTone(syl);
    const k = { ...base, ...keys }[detectTone(syl)];
    if ((isLeft(prev) && isRight(k)) || (isRight(prev) && isLeft(k))) a++;
  }
  const pct = a / TONED.length * 100;
  console.log(`    ${what}  alt=${pct.toFixed(1)}% of ${TONED.length}  (${(pct - variantAlt >= 0 ? '+' : '') + (pct - variantAlt).toFixed(1)}pp)`);
}
const nNang = perTone['nặng'].total, nNga = perTone['ngã'].total;
console.log('  → nặng=j is the whole win (+7.8pp): nặng syllables end on a right-hand');
console.log(`    coda 66% of the time (of n=${nNang}), so a left-hand tone key alternates.`);
console.log('    ngã=x is a small net NEGATIVE (-2.9pp) — ngã syllables end left-hand');
console.log(`    80% of the time (of n=${nNga}). It is kept for standard-Telex muscle-memory`);
console.log(`    compatibility, not for alternation, and at n=${nNga} ` +
            `(${(nNga / VIET_SYLLABLES.length * 100).toFixed(1)}% of all ${VIET_SYLLABLES.length} syllables)`);
console.log('    it is too rare to outweigh nặng. Be precise about this: the .ahk');
console.log('    header\'s "codas favor left-hand tone keys" holds for nặng and sắc,');
console.log('    not for ngã, hỏi or huyền.');
console.log('  Note: an earlier version of this script silently dropped every');
console.log('  single-vowel syllable and reported 66.5% vs 63.6% (a 2.9pp gap) over');
console.log('  n=206. Those percentages were wrong. The corrected full-sample figures');
console.log('  above are lower, but the gap is WIDER — dropping the filter strengthens');
console.log('  the case for the shipped layout rather than weakening it.');
