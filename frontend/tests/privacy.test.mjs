import assert from 'node:assert/strict'
import { test } from 'node:test'
import { privacyConfig, validPrivacyChoice } from '../app/utils/privacy.ts'

test('privacy notice accepts only a current, unexpired choice within its retention window', () => {
  const now = Date.UTC(2026, 9, 16)
  const choice = (version, expires) => JSON.stringify({ version, expires })
  const deadline = now + privacyConfig.retentionDays * 86400000
  assert.equal(validPrivacyChoice(choice(privacyConfig.version, deadline), now), true)
  for (const raw of [null, '', 'broken', 'null', '[]', '{}',
    choice(privacyConfig.version + 1, deadline),
    choice(privacyConfig.version, now),
    choice(privacyConfig.version, now - 1),
    choice(privacyConfig.version, deadline + 1),
    choice(privacyConfig.version, String(deadline)),
  ]) assert.equal(validPrivacyChoice(raw, now), false)
})
