// Count tone frequency in a Vietnamese text sample.
// One syllable = one whitespace/punctuation-delimited token.
//
// These are the numbers quoted in VietDvorak.ahk's header comment next to each
// tone key (~21% sắc, ~22% huyền, ~9% hỏi, ~4% ngã, ~18% nặng), so if this
// output moves, that header is stale.
//
// Corpus and helpers come from viet-corpus.mjs — see that file before changing
// how a syllable is split or a tone detected.

import {
  CORPUS_TEXT, SYLLABLES, VIET_SYLLABLES, detectTone, TONE_NAMES, NO_TONE,
} from './viet-corpus.mjs';

const stats = { [NO_TONE]: 0 };
for (const t of TONE_NAMES) stats[t] = 0;

// VIET_SYLLABLES is the corpus with non-Vietnamese tokens (no vowel at all)
// already dropped, filtered once in viet-corpus.mjs so all three scripts share
// one denominator. Do not re-filter here.
for (const syl of VIET_SYLLABLES) stats[detectTone(syl)]++;

const total = VIET_SYLLABLES.length;

console.log(`Corpus: ${CORPUS_TEXT.length} chars, ${SYLLABLES.length} tokens`);
console.log(`Total syllables analyzed: ${total}`);
console.log(`(every percentage below is out of these ${total})\n`);

// Frequency order, most common first.
const order = [NO_TONE, 'sắc', 'huyền', 'nặng', 'hỏi', 'ngã'];
for (const t of order) {
  const n = stats[t];
  const pct = ((n / total) * 100).toFixed(1);
  const bar = '█'.repeat(Math.round(pct));
  console.log(`${t.padEnd(8)} ${String(n).padStart(4)}  ${pct.padStart(5)}% of ${total}  ${bar}`);
}

const toned = total - stats[NO_TONE];
console.log(`\nToned syllables: ${toned} of ${total} (${((toned / total) * 100).toFixed(1)}% of all syllables)`);
console.log('→ these are the syllables that cost an extra tone keystroke.');
