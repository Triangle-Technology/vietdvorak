// Hand-alternation cost of the tone keys — EARLY-TONE MODEL ONLY.
//
// ###########################################################################
// ## SCOPE WARNING — read before quoting any number out of this file.      ##
// ##                                                                       ##
// ## score() below knows exactly two things about a syllable: which tone   ##
// ## it carries, and which hand the tone key sits on. It never looks at    ##
// ## the syllable's coda. That hardcodes one assumption:                   ##
// ##                                                                       ##
// ##     the keystroke immediately before the tone key is always a         ##
// ##     left-hand VOWEL                                                   ##
// ##                                                                       ##
// ## which is true only for EARLY-tone typing (tone struck right after the ##
// ## vowel). Under that assumption a right-hand tone key always alternates ##
// ## and a left-hand one never does, so the model can only ever conclude   ##
// ## "put every tone key on the right hand". It is structurally incapable  ##
// ## of testing the opposite premise.                                      ##
// ##                                                                       ##
// ## The shipped layout was NOT chosen on this model. The user types       ##
// ## LATE-tone (whole syllable, then tone), where the preceding keystroke  ##
// ## is usually a right-hand CODA consonant, which flips the conclusion.   ##
// ## That is what VietDvorak.ahk's header cites, and it is measured in     ##
// ## bigram-late-tone.mjs — the authoritative script for the layout        ##
// ## decision. Keep this one as the early-tone counterfactual.             ##
// ###########################################################################
//
// Corpus and helpers: viet-corpus.mjs.

import {
  VIET_SYLLABLES, detectTone, isRight, TONE_NAMES, NO_TONE,
  TONE_KEYS_SHIPPED, LABEL_SHIPPED,
  TONE_KEYS_ALL_RIGHT_VARIANT, LABEL_ALL_RIGHT_VARIANT,
} from './viet-corpus.mjs';

// --- Tone frequency (same tally as tone-stats.mjs, from the same helpers) ---
// Same sample too: VIET_SYLLABLES is filtered once in viet-corpus.mjs, so this
// script's denominator cannot drift away from the other two.

const toneCounts = {};
for (const t of TONE_NAMES) toneCounts[t] = 0;

const total = VIET_SYLLABLES.length;
let tonedCount = 0;
for (const syl of VIET_SYLLABLES) {
  const tone = detectTone(syl);
  if (tone !== NO_TONE) { toneCounts[tone]++; tonedCount++; }
}

// --- Scoring ------------------------------------------------------------
// Under the early-tone assumption documented above, "does this bigram
// alternate?" collapses to "is the tone key on the right hand?".

// Every percentage this script prints carries its denominator, and the two totals
// print BOTH framings. Without that, this file's "52.4% alternation" and
// bigram-late-tone.mjs's early-tone "70.4%" look like a contradiction: they are
// the same count of the same syllables, over 685 and over 510 respectively.
const pctAll   = n => ((n / total)      * 100).toFixed(1);
const pctToned = n => ((n / tonedCount) * 100).toFixed(1);
const ofAll   = n => `${pctAll(n)}% of all ${total} syllables`;
const ofToned = n => `${pctToned(n)}% of the ${tonedCount} toned`;

function score(layout, label) {
  let altCount = 0, sameCount = 0;
  const breakdown = {};
  for (const [tone, key] of Object.entries(layout)) {
    const n = toneCounts[tone];
    const alt = isRight(key);   // right-hand key + assumed left-hand vowel = alternation
    breakdown[tone] = { key, alt, n };
    if (alt) altCount += n; else sameCount += n;
  }
  console.log(`\n=== ${label} ===`);
  for (const [tone, info] of Object.entries(breakdown)) {
    const pct = ((info.n / total) * 100).toFixed(1);
    console.log(`  ${tone.padEnd(8)} '${info.key}'  ${info.alt ? 'alternates' : 'SAME-HAND  '}  ${pct.padStart(5)}% of all ${total} syllables`);
  }
  console.log(`  Total alternation: ${ofAll(altCount)}  (${ofToned(altCount)})`);
  console.log(`  Total same-hand:   ${ofAll(sameCount)}  (${ofToned(sameCount)})`);
  return { altCount, sameCount };
}

console.log('!!! EARLY-TONE MODEL ONLY — assumes the key before the tone is always a');
console.log('!!! left-hand vowel, i.e. ignores codas entirely. This model cannot test');
console.log('!!! the late-tone premise the shipped layout rests on; by construction it');
console.log('!!! always prefers right-hand tone keys. See bigram-late-tone.mjs.');
console.log();
console.log(`Total syllables analyzed: ${total}`);
console.log(`Toned syllables: ${tonedCount} of ${total} (${((tonedCount / total) * 100).toFixed(1)}% of all syllables)`);
console.log(`Every percentage below states what it is a percentage OF; the totals`);
console.log(`state it twice, because bigram-late-tone.mjs reports the toned-only form.`);

const shipped = score(TONE_KEYS_SHIPPED, LABEL_SHIPPED);
const variant = score(TONE_KEYS_ALL_RIGHT_VARIANT, LABEL_ALL_RIGHT_VARIANT);

const diff = (shipped.sameCount - variant.sameCount) / total * 100;
console.log(`\n→ UNDER THE EARLY-TONE ASSUMPTION ONLY: the shipped x/j layout scores`);
console.log(`  ${diff.toFixed(1)}% more same-hand syllables than the unshipped z/b variant`);
console.log(`  (same-hand ${pctAll(shipped.sameCount)}% vs ${pctAll(variant.sameCount)}% of all ${total} syllables;`);
console.log(`   ${pctToned(shipped.sameCount)}% vs ${pctToned(variant.sameCount)}% of the ${tonedCount} toned).`);
console.log(`  That is the expected artefact of a coda-blind model, not a defect in the`);
console.log(`  shipped layout: with codas taken into account (bigram-late-tone.mjs) the`);
console.log(`  ordering reverses and x/j wins. Do not quote this line on its own.`);
