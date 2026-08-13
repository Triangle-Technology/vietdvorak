// ============================================================================
// Shared data + helpers for the VietDvorak layout analysis notebooks.
// ============================================================================
//
// This module is the SINGLE SOURCE OF TRUTH for everything the three analysis
// scripts (tone-stats.mjs, bigram-analysis.mjs, bigram-late-tone.mjs) have in
// common. Before this existed, each script carried its own pasted copy of the
// corpus, its own syllable-split regex, and its own tone table — and the tone
// tables had already drifted apart from each other. If you need to change how a
// syllable is split, which tokens count as syllables (VIET_SYLLABLES — the one
// sample every script counts from), how a tone is detected, or which keys the
// product ships, change it HERE, once.
//
// Corpus text lives in corpus.txt (plain UTF-8, one paragraph per line) so it is
// data, not source. Do not paste it back into a script.

import { readFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const HERE = dirname(fileURLToPath(import.meta.url));

// ---------------------------------------------------------------------------
// Corpus
// ---------------------------------------------------------------------------

export const CORPUS_TEXT = readFileSync(join(HERE, 'corpus.txt'), 'utf8');

// Whitespace + punctuation + digits all act as syllable separators. Vietnamese
// is written with one syllable per whitespace-delimited token, so this is a good
// enough tokenizer for frequency work.
export const SYLLABLE_SEPARATORS = /[\s.,;:?!()'"\-—\d\n]+/;

// Tokens are lowercased: every downstream analysis is case-insensitive, and the
// physical key you press is the same either way.
export function splitSyllables(text) {
  return text.toLowerCase().split(SYLLABLE_SEPARATORS).filter(s => s.length > 0);
}

/** The corpus as a flat list of lowercased syllable tokens. */
export const SYLLABLES = splitSyllables(CORPUS_TEXT);

// ---------------------------------------------------------------------------
// Tones
// ---------------------------------------------------------------------------

// The ONE tone table. Lowercase glyphs only — detectTone() folds case itself, so
// listing the uppercase forms as well (as two of the old copies did) would be
// redundant, not more correct.
export const TONE_MARKS = {
  'sắc':   new Set('áấắéếíóốớúứý'),
  'huyền': new Set('àầằèềìòồờùừỳ'),
  'hỏi':   new Set('ảẩẳẻểỉỏổởủửỷ'),
  'ngã':   new Set('ãẫẵẽễĩõỗỡũữỹ'),
  'nặng':  new Set('ạậặẹệịọộợụựỵ'),
};

/** Tone names in the order results are usually reported. */
export const TONE_NAMES = Object.keys(TONE_MARKS);

/** Toneless syllables are reported under this name. */
export const NO_TONE = 'ngang';

/**
 * Tone of a syllable, or 'ngang' if it carries none.
 * Case-insensitive: callers do not have to lowercase first.
 */
export function detectTone(syl) {
  for (const c of syl) {
    const lo = c.toLowerCase();
    for (const [tone, marks] of Object.entries(TONE_MARKS)) {
      if (marks.has(lo)) return tone;
    }
  }
  return NO_TONE;
}

// ---------------------------------------------------------------------------
// Vowels
// ---------------------------------------------------------------------------

// Base vowels: the seven plain letters plus the four modified bases (ă â ê ô ơ ư)
// that Vietnamese treats as distinct vowels.
export const BASE_VOWELS = new Set('aăâeêioôơuưy');

// Every vowel that can appear in text, toned or not. Note this INCLUDES the toned
// forms — a guard written over the untoned letters alone silently degrades into a
// "must have at least two vowels" filter, which is exactly the bug that used to
// skew bigram-late-tone.mjs.
export const VIET_VOWELS = new Set(BASE_VOWELS);
for (const marks of Object.values(TONE_MARKS)) {
  for (const c of marks) VIET_VOWELS.add(c);
}

/** True if the token contains any Vietnamese vowel, toned or untoned. */
export function hasVietVowel(syl) {
  return [...syl].some(c => VIET_VOWELS.has(c.toLowerCase()));
}

/**
 * THE sample: corpus tokens that are actually Vietnamese syllables.
 *
 * Every script counts from THIS list, so "how many syllables are we talking
 * about?" has exactly one answer across the whole notebook set. Each script used
 * to re-derive it with its own inline `if (!hasVietVowel(syl)) continue` loop
 * (and bigram-late-tone.mjs skipped the filter altogether); they agree today only
 * because the current corpus happens to contain no vowel-less token. One such
 * token — a bare "!" run, a stray "km", a Roman numeral — and the denominators
 * would silently split again, which is the exact defect this module exists to
 * end. Filter once, here.
 */
export const VIET_SYLLABLES = SYLLABLES.filter(hasVietVowel);

// Map every vowel glyph (and đ) back to the plain ASCII letter the typist
// actually presses, so we can reason about physical key positions. Grouped by
// target letter so each base is stated once.
export const VOWEL_BASE = {};
for (const [base, glyphs] of Object.entries({
  a: 'aàáảãạ' + 'ăằắẳẵặ' + 'âầấẩẫậ',
  e: 'eèéẻẽẹ' + 'êềếểễệ',
  i: 'iìíỉĩị',
  o: 'oòóỏõọ' + 'ôồốổỗộ' + 'ơờớởỡợ',
  u: 'uùúủũụ' + 'ưừứửữự',
  y: 'yỳýỷỹỵ',
  d: 'dđ',
})) {
  for (const g of glyphs) VOWEL_BASE[g] = base;
}

/** Strip tone + diacritic from one character, giving the key that was typed. */
export function stripChar(c) {
  const lo = c.toLowerCase();
  return VOWEL_BASE[lo] || lo;
}

/** Strip a whole syllable down to the plain letters the typist pressed. */
export function stripSyllable(syl) {
  return [...syl].map(stripChar).join('');
}

// ---------------------------------------------------------------------------
// Dvorak hand assignment
// ---------------------------------------------------------------------------
//
// Dvorak (simplified US layout), letters only:
//   top    ' , . p y | f g c r l
//   home   a o e u i | d h t n s
//   bottom ; q j k x | b m w v z
// Left of the bar is the left hand. All vowels are on the left hand, which is the
// whole reason tone-key placement is interesting.

export const LEFT_HAND = new Set([
  "'", ',', '.', 'p', 'y',
  'a', 'o', 'e', 'u', 'i',
  ';', 'q', 'j', 'k', 'x',
]);
export const RIGHT_HAND = new Set([
  'f', 'g', 'c', 'r', 'l', '/', '=', '\\',
  'd', 'h', 't', 'n', 's', '-',
  'b', 'm', 'w', 'v', 'z',
]);

export function isLeft(ch)  { return LEFT_HAND.has(ch); }
export function isRight(ch) { return RIGHT_HAND.has(ch); }

// ---------------------------------------------------------------------------
// Tone-key layouts (product configuration — keep in step with the .ahk)
// ---------------------------------------------------------------------------

// Cite the SYMBOL, never a line range. This used to read "VietDvorak.ahk:98-104"
// in two places (a comment and the label printed on every result line); those
// lines are the tail of the VOWELS map, and were never TONE_KEYS. A line number
// into a file under active edit cannot be kept true, and a wrong one printed on
// every run is worse than none. Stated ONCE here; the doc comment and the printed
// label both reference this constant.
const SHIPPED_SOURCE = 'VietDvorak.ahk :: TONE_KEYS';

/**
 * What the product actually ships: s / f / r / x / j — standard Telex.
 *
 * This is a hand-kept copy of the map named by SHIPPED_SOURCE (no import is
 * possible across the AHK/JS runtime boundary). The startup check at the bottom
 * of this file re-reads that map out of the .ahk and refuses to run if the two
 * disagree, so the copy cannot drift unnoticed.
 */
export const TONE_KEYS_SHIPPED = {
  'sắc': 's', 'huyền': 'f', 'hỏi': 'r', 'ngã': 'x', 'nặng': 'j',
};

export const LABEL_SHIPPED = `Standard Telex, ngã=x nặng=j  [SHIPPED — ${SHIPPED_SOURCE}]`;

/**
 * Tone -> tone INDEX, the number the .ahk actually identifies a tone by.
 *
 * The .ahk writes its TONE_KEYS as key -> index and every consumer there uses
 * that number: Transform() compares it against a vowel's current tone, and
 * BuildToneHelp() reads VOWELS["a"][idx + 1] (position 1 being the toneless
 * base). 1..5 in the order sắc-huyền-hỏi-ngã-nặng is therefore the real
 * cross-runtime contract, and it is what the startup check below joins on.
 *
 * Spelled out rather than derived from the order of TONE_MARKS: the numbering is
 * a fact about the product, not about which order this file happens to list its
 * tone glyphs in, and reordering TONE_MARKS (whose order is only a reporting
 * convenience) must not silently renumber the tones.
 */
const TONE_INDEX = {
  'sắc': 1, 'huyền': 2, 'hỏi': 3, 'ngã': 4, 'nặng': 5,
};

/**
 * Historical alternative that was considered and NOT shipped: it moves ngã and
 * nặng onto the right hand (z, b), putting every tone key on the right. Kept
 * only so the comparison in the notebooks stays runnable. This is NOT VietDvorak.
 */
export const TONE_KEYS_ALL_RIGHT_VARIANT = {
  'sắc': 's', 'huyền': 'f', 'hỏi': 'r', 'ngã': 'z', 'nặng': 'b',
};
export const LABEL_ALL_RIGHT_VARIANT = 'All-right variant, ngã=z nặng=b  [NOT shipped — historical]';

// ---------------------------------------------------------------------------
// Startup drift check: TONE_KEYS_SHIPPED vs the map it claims to mirror
// ---------------------------------------------------------------------------
//
// The key -> tone mapping is hand-written in three runtimes that cannot import
// from each other: VietDvorak.ahk (TONE_KEYS, the product), practice.html
// (TONE_KEYS, the trainer) and TONE_KEYS_SHIPPED above (these notebooks). The
// trainer's copy is covered indirectly — check-parity.mjs runs the Telex
// conformance vectors through it, and a wrong tone key fails them. Nothing
// covered THIS copy, so every "[SHIPPED]" figure the notebooks print rested on a
// promise in a comment.
//
// We cannot delete the copy, but we can make it self-checking: the .ahk is right
// there on disk, so read the real map back and compare. Disagreement is a real
// defect (the notebooks would be measuring a layout the product does not ship)
// and throws. Failure to PARSE is a defect in the reader below, not in the
// product — it warns and lets the analysis run, so that reformatting the .ahk can
// never take these scripts down.

const AHK_PATH = join(HERE, 'VietDvorak.ahk');

/** The reader could not read — distinct from "the two maps disagree". */
class AhkParseError extends Error {}

/**
 * Entries of an AHK `global NAME := Map(...)` literal, in source order, as
 * [key, value] pairs of strings and numbers.
 *
 * Scans the literal instead of matching lines, so line breaks, indentation,
 * pair grouping and trailing `;` comments are all irrelevant; the only
 * formatting this depends on is the `global NAME := Map(` header itself. Nested
 * calls and non-literal arguments are out of scope and raise AhkParseError
 * rather than being silently misread.
 */
function ahkMapEntries(src, mapName) {
  const header = new RegExp(String.raw`global\s+${mapName}\s*:=\s*Map\s*\(`);
  const m = header.exec(src);
  if (!m) throw new AhkParseError(`no "global ${mapName} := Map(" found`);

  const tokens = [];
  let i = m.index + m[0].length;   // first char inside the '('
  let depth = 1;
  while (i < src.length && depth > 0) {
    const c = src[i];
    if (c === '"') {                       // string literal ("" escapes a quote)
      let j = i + 1, s = '';
      for (;;) {
        if (j >= src.length) throw new AhkParseError(`unterminated string in ${mapName}`);
        if (src[j] === '"') {
          if (src[j + 1] !== '"') break;
          s += '"'; j += 2; continue;
        }
        s += src[j++];
      }
      if (depth === 1) tokens.push(s.normalize('NFC'));
      i = j + 1;
    } else if (c === ';') {                // AHK line comment
      while (i < src.length && src[i] !== '\n') i++;
    } else if (c === '(') { depth++; i++; }
    else if (c === ')') { depth--; i++; }
    else if (c >= '0' && c <= '9' && !/[A-Za-z_]/.test(src[i - 1])) {
      let j = i;
      while (j < src.length && src[j] >= '0' && src[j] <= '9') j++;
      if (depth === 1) tokens.push(Number(src.slice(i, j)));
      i = j;
    } else i++;
  }
  if (depth !== 0) throw new AhkParseError(`unbalanced Map( ... ) for ${mapName}`);
  if (tokens.length === 0 || tokens.length % 2 !== 0) {
    throw new AhkParseError(`${mapName} did not read as key/value pairs (${tokens.length} items)`);
  }
  const pairs = [];
  for (let k = 0; k < tokens.length; k += 2) pairs.push([tokens[k], tokens[k + 1]]);
  return pairs;
}

/**
 * The shipped tone INDEX -> key map, read out of the .ahk's TONE_KEYS.
 *
 * Reads TONE_KEYS and nothing else. The tone's Vietnamese NAME is not part of
 * the cross-runtime contract — the .ahk keeps names in TONE_NAMES, which it
 * declares there as display data for the help dialog — so recasing or rewording
 * one of those labels is a product-only edit and must not be able to reach this
 * check at all. Joining through TONE_NAMES (as this function used to) made every
 * such edit fail all three scripts, and blame TONE_KEYS_SHIPPED for it.
 */
function shippedToneKeysFromAhk(src) {
  const byIndex = new Map();
  for (const [key, idx] of ahkMapEntries(src, 'TONE_KEYS')) {
    if (typeof key !== 'string' || typeof idx !== 'number') {
      throw new AhkParseError(`TONE_KEYS entry is not "key"/index: ${key}, ${idx}`);
    }
    if (byIndex.has(idx)) {
      // Legal in the product (two keys, one tone), but TONE_KEYS_SHIPPED is
      // one-key-per-tone and cannot represent it — so the check cannot conclude
      // anything. That is a limit of this reader, not proven drift: warn.
      throw new AhkParseError(
        `TONE_KEYS gives tone ${idx} two keys ('${byIndex.get(idx)}' and '${key}'); ` +
        `the notebooks' one-key-per-tone model cannot represent that`);
    }
    byIndex.set(idx, key);
  }
  return byIndex;
}

/**
 * Tone index -> the .ahk's own Vietnamese label for it, best effort.
 *
 * Used ONLY to make the messages below readable. TONE_NAMES is display data, so
 * nothing here may throw or otherwise change the outcome of the check: an
 * unreadable TONE_NAMES just means the messages fall back to the tone number.
 */
function ahkToneLabels(src) {
  const labels = new Map();
  try {
    for (const [idx, name] of ahkMapEntries(src, 'TONE_NAMES')) {
      if (typeof idx === 'number' && typeof name === 'string') labels.set(idx, name);
    }
  } catch (err) {
    if (!(err instanceof AhkParseError)) throw err;
  }
  return labels;
}

let ahkShipped = null;
let ahkLabels = new Map();
try {
  const ahkSrc = readFileSync(AHK_PATH, 'utf8');
  ahkShipped = shippedToneKeysFromAhk(ahkSrc);
  ahkLabels = ahkToneLabels(ahkSrc);
} catch (err) {
  // Cannot read the .ahk (moved away, reformatted past what the scanner handles)
  // -> the CHECK is broken, not the layout. Say so loudly and carry on.
  if (!(err instanceof AhkParseError) && err.code !== 'ENOENT') throw err;
  console.warn(
    `viet-corpus: could NOT verify TONE_KEYS_SHIPPED against ${SHIPPED_SOURCE} — ${err.message}.\n` +
    `             Analysis still runs, but every [SHIPPED] figure below is unverified.\n` +
    `             Fix the reader in viet-corpus.mjs (ahkMapEntries) or the .ahk path.`);
}

if (ahkShipped) {
  // Join on the tone INDEX (see TONE_INDEX and shippedToneKeysFromAhk): it is the
  // only thing both runtimes agree to mean. Nothing below compares a tone's name
  // across the boundary — which is also why no NFC normalisation is needed here
  // any more; no string crosses it.
  const ours = new Map();
  const problems = [];
  for (const [name, key] of Object.entries(TONE_KEYS_SHIPPED)) {
    const idx = TONE_INDEX[name];
    if (idx === undefined) {
      problems.push(`  '${name}' has a key ('${key}') but no entry in TONE_INDEX, so it cannot be checked`);
    } else {
      ours.set(idx, key);
    }
  }

  // Tones are identified by number; a name is only ever printed as a label. Show
  // ours, plus the .ahk's when it has one and it reads differently, so a message
  // stays legible after the help dialog is reworded.
  const describe = idx => {
    const names = [...new Set(
      [Object.keys(TONE_INDEX).find(n => TONE_INDEX[n] === idx), ahkLabels.get(idx)].filter(Boolean))];
    return names.length ? `tone ${idx} (${names.join(' / ')})` : `tone ${idx}`;
  };

  for (const idx of [...new Set([...ahkShipped.keys(), ...ours.keys()])].sort((a, b) => a - b)) {
    const theirKey = ahkShipped.get(idx), ourKey = ours.get(idx);
    if (ourKey === undefined)        problems.push(`  ${describe(idx)}: '${theirKey}' in the .ahk, missing here`);
    else if (theirKey === undefined) problems.push(`  ${describe(idx)}: '${ourKey}' here, missing from the .ahk`);
    else if (theirKey !== ourKey)    problems.push(`  ${describe(idx)}: '${theirKey}' in the .ahk, '${ourKey}' here`);
  }
  if (problems.length) {
    throw new Error(
      `TONE_KEYS_SHIPPED has drifted from ${SHIPPED_SOURCE}:\n${problems.join('\n')}\n` +
      `Everything these notebooks label [SHIPPED] is computed from the copy in\n` +
      `viet-corpus.mjs. Until it matches the product, those figures describe a\n` +
      `layout nobody ships. Fix the copy (or the .ahk) before reading any output.`);
  }
}
