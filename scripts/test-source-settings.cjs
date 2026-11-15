#!/usr/bin/env node
// Regression test for source-mode settings in the real browser script.
// Uses only Node's standard library and a minimal DOM stub. It does NOT emulate the PS5.
'use strict';

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const html = fs.readFileSync(path.join(__dirname, '..', 'src', 'ui.html'), 'utf8');
const start = html.indexOf("var BOOLS = ['inject_ampr'");
const end = html.indexOf('function renderSettingsInfo()', start);
assert.ok(start >= 0 && end > start, 'could not locate the settings functions in UI');
const snippet = html.slice(start, end);

const fields = Object.create(null);
const get = (id) => (fields[id] ||= { value: '', checked: false, disabled: false, title: '' });
const translate = (key) => key;
// Evaluate just the original settings functions. No network or browser APIs are called.
const { fillSettings, readSettings, guardPurge } =
  new Function('$', 't', snippet + '; return {fillSettings, readSettings, guardPurge};')(get, translate);

const original = {
  scanpath: ['/mnt/usb0/homebrew'],
  source_mode: 'pkg',
  inject_ampr: true,
  ampr_auto_update: true,
  backports: true,
  notify: true,
  log: true,
  debug: false,
  scan_interval_seconds: 10,
  low_space_gb: 20,
};

fillSettings(original);
assert.equal(get('f_source_mode').value, 'pkg');
assert.equal(get('spurge').disabled, true);
const payload = readSettings();
assert.deepEqual(payload, original, 'settings POST payload must round-trip');

get('f_source_mode').value = 'folder';
guardPurge();
assert.equal(get('spurge').disabled, true);

get('f_source_mode').value = 'all';
guardPurge();
assert.equal(get('spurge').disabled, false);

fillSettings({ ...original, source_mode: undefined });
assert.equal(get('f_source_mode').value, 'all', 'old config defaults to all');
assert.equal(get('spurge').disabled, false);

console.log('source settings UI: 6 checks passed');
