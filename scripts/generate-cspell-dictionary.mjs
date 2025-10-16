#!/usr/bin/env node
import { importTrie, iteratorTrieWords } from "cspell-trie-lib";
import { readFileSync, writeFileSync } from "node:fs";
import { gunzipSync } from "node:zlib";
import { createRequire } from "node:module";
import path from "node:path";

const require = createRequire(import.meta.url);
const configPath = require.resolve("@cspell/dict-en_us/cspell-ext.json");
const pkgDir = path.dirname(configPath);
const triePath = path.join(pkgDir, "en_US.trie.gz");

const data = gunzipSync(readFileSync(triePath));
const lines = data.toString("utf8").split(/\r?\n/);
const trie = importTrie(lines);

const allowPattern = /^[a-z]+$/i;
const words = iteratorTrieWords(trie);
const set = new Set();

for (const word of words) {
  if (word.toLowerCase() !== word) continue;
  if (!allowPattern.test(word)) continue;
  const upper = word.toUpperCase();
  if (upper.length < 2 || upper.length > 15) continue;
  set.add(upper);
}

const sorted = Array.from(set).sort();
const outPath = path.join("src-tauri", "src", "data", "cspell-words.txt");
writeFileSync(outPath, `${sorted.join("\n")}\n`);

console.log(`Generated ${sorted.length} entries at ${outPath}`);
