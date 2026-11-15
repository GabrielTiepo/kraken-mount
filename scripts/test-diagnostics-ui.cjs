#!/usr/bin/env node
// Smoke tests against the actual diagnostics renderer in src/ui.html.
// No console, game or AMPR process is simulated. Only the UI data contract is tested.
'use strict';

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const html = fs.readFileSync(path.join(__dirname, '..', 'src', 'ui.html'), 'utf8');
assert.ok(html.includes('id="diag-scan"') && html.includes('id="diag-build"'));
const start = html.indexOf('function renderSettingsInfo()');
const end = html.indexOf('async function loadSettings()', start);
assert.ok(start > 0 && end > start, 'diagnostics renderer must be present');
const source = html.slice(start, end);
const dictionary = {
  diagNone: 'Not measured yet.',
  diagScan: 'Scan: {ms} ms, {games} games, {duplicates} duplicates, {errors} errors.',
  diagBuild: 'Cache: {title} ({kind}, {drive}) — {ms} ms — {result}.',
  stReady: 'Ready',
  stError: 'Error',
  badgeFolder: 'Folder',
  cacheInfo: 'Cache: {size} in {path}, {free} free.',
  driveInfo: '{drive}: free {free} of {total}',
};
function translate(key, args) {
  return (dictionary[key] || key).replace(/\{(\w+)\}/g, (_, name) => String((args || {})[name] ?? ''));
}
function render(state) {
  const fields = {};
  const get = (id) => (fields[id] ||= { textContent: '' });
  const renderer = new Function('$', 't', 'state', 'setText', 'driveLabel', 'fmtSize', 'gb',
    source + '; return renderSettingsInfo;')(
    get, translate, state,
    (node, text) => { node.textContent = String(text); },
    (mount) => mount === '/mnt/usb0' ? 'USB 1' : mount,
    (size) => size + ' B',
    (size) => size + ' B',
  );
  renderer();
  return fields;
}

const empty = render(null);
assert.equal(empty['diag-scan'].textContent, 'Not measured yet.');
assert.equal(empty['diag-build'].textContent, 'Not measured yet.');

const metrics = render({
  diagnostics: {
    scope: 'management',
    scan: { elapsed_ms: 125, games: 3, duplicates: 1, errors: 0 },
    builds: [
      { title_id: 'PPSA35889', kind: 'pkg', drive: '/mnt/usb0', elapsed_ms: 6000, success: false },
      { title_id: 'PPSA12345', kind: 'folder', drive: '/data', elapsed_ms: 50, success: true },
    ],
  },
  cache: { size: 100, path: '/data/krakenmount/caches', free: 200 },
  drives: [],
});
assert.match(metrics['diag-scan'].textContent, /125 ms, 3 games/);
assert.match(metrics['diag-build'].textContent, /PPSA12345/);
assert.match(metrics['diag-build'].textContent, /50 ms/);
assert.doesNotMatch(metrics['diag-build'].textContent, /PPSA35889/);
assert.match(metrics['diag-build'].textContent, /Ready/);

const failed = render({
  diagnostics: {
    scan: null,
    builds: [{ title_id: 'PPSA35889', kind: 'pkg', drive: '/mnt/usb0', elapsed_ms: 1234, success: false }],
  },
  cache: {},
  drives: [],
});
assert.equal(failed['diag-scan'].textContent, 'Not measured yet.');
assert.match(failed['diag-build'].textContent, /USB 1/);
assert.match(failed['diag-build'].textContent, /Error/);

assert.ok(html.includes("diagTitle: 'Diagnóstico de varredura e cache'"), 'Portuguese strings must be present');
assert.ok(html.includes("diagHint: 'RAM-only timings"), 'scope warning must be present in the English UI');
console.log('diagnostics UI smoke tests: passed');
