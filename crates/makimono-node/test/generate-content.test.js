const test = require('node:test');
const assert = require('node:assert/strict');

const binding = require('../index.js');
const generateContent = binding.generateContent ?? binding.generate_content;

test('exports generate content function', () => {
  assert.equal(typeof generateContent, 'function');
});

test('inserts message using default settings', () => {
  const content = '### Latest Changes\n\n### 0.1.0\n\n* Old entry\n';
  const result = generateContent(content, '* New entry', []);

  assert.ok(result.includes('* New entry'));
  assert.ok(result.includes('### 0.1.0'));
});

test('supports custom start header via options', () => {
  const content = '## Changes\n\n## 0.1.0\n\n* Old entry\n';
  const options = {
    startHeader: '## Changes',
    endRegex: '(?m)(^## .*)',
  };

  const result = generateContent(content, '* New entry', [], options);

  assert.ok(result.includes('* New entry'));
  assert.ok(result.includes('## 0.1.0'));
});
