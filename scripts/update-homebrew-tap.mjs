import { execFileSync } from 'node:child_process'
import { createHash } from 'node:crypto'
import { mkdtempSync, readFileSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'

const version = process.argv[2] || process.env.SEMANTIC_RELEASE_NEXT_RELEASE_VERSION
if (!version) {
  throw new Error('No release version given (pass it as argv[2])')
}

const tapToken = process.env.HOMEBREW_TAP_TOKEN
if (!tapToken) {
  console.log('[update-homebrew-tap] HOMEBREW_TAP_TOKEN not set, skipping tap update')
  process.exit(0)
}

const dmgPath = `.release/Mapper_${version}_aarch64.dmg`
const sha256 = createHash('sha256').update(readFileSync(dmgPath)).digest('hex')

const tapDir = mkdtempSync(join(tmpdir(), 'homebrew-mapper-'))
const tapRemote = `https://x-access-token:${tapToken}@github.com/jabedzaman/homebrew-mapper.git`

const run = (cmd, args, cwd) => execFileSync(cmd, args, { cwd, stdio: 'inherit' })

run('git', ['clone', '--depth', '1', tapRemote, tapDir])

const caskPath = join(tapDir, 'Casks/mapper.rb')
const cask = readFileSync(caskPath, 'utf8')
  .replace(/version "[^"]+"/, `version "${version}"`)
  .replace(/sha256 "[^"]+"/, `sha256 "${sha256}"`)
writeFileSync(caskPath, cask)

run('git', ['-c', 'user.name=mapper-release', '-c', 'user.email=noreply@anthropic.com', 'commit', '-am', `mapper ${version}`], tapDir)
run('git', ['push', 'origin', 'HEAD:main'], tapDir)

console.log(`[update-homebrew-tap] pushed mapper ${version} (sha256 ${sha256})`)
