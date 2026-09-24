import { execSync } from 'node:child_process'
import { existsSync, cpSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from 'node:fs'
import { resolve } from 'node:path'

const version = process.argv[2] || process.env.SEMANTIC_RELEASE_NEXT_RELEASE_VERSION
if (!version) {
  throw new Error('No release version given (pass it as argv[2])')
}

const pkgJson = (path) => JSON.parse(readFileSync(path, 'utf8'))
const setPkgVersion = (path) => {
  const pkg = pkgJson(path)
  pkg.version = version
  writeFileSync(path, `${JSON.stringify(pkg, null, 2)}\n`)
}

setPkgVersion('package.json')
setPkgVersion('apps/menubar/package.json')

const tauriConf = JSON.parse(readFileSync('apps/menubar/src-tauri/tauri.conf.json', 'utf8'))
tauriConf.version = version
writeFileSync('apps/menubar/src-tauri/tauri.conf.json', `${JSON.stringify(tauriConf, null, 2)}\n`)

const cargoToml = readFileSync('apps/menubar/src-tauri/Cargo.toml', 'utf8')
writeFileSync(
  'apps/menubar/src-tauri/Cargo.toml',
  cargoToml.replace(/^(version\s*=\s*)"[^"]+"/m, `$1"${version}"`),
)

const cargoLock = readFileSync('apps/menubar/src-tauri/Cargo.lock', 'utf8')
const cargoLockNext = cargoLock.replace(
  /(\[\[package\]\]\nname = "mapper"\nversion = ")[^"]+(")/m,
  `$1${version}$2`,
)
if (cargoLockNext === cargoLock) {
  throw new Error('Could not find "mapper" package in Cargo.lock')
}
writeFileSync('apps/menubar/src-tauri/Cargo.lock', cargoLockNext)

if (process.env.RELEASE_PREPARE_SKIP_BUILD === '1') {
  console.log(`[release-prepare] version synced to ${version} (build skipped)`)
  process.exit(0)
}

execSync('pnpm build', { stdio: 'inherit' })

const macos = 'apps/menubar/src-tauri/target/release/bundle/macos'
const dmgDir = 'apps/menubar/src-tauri/target/release/bundle/dmg'
const releaseDir = '.release'
const releaseDirAbs = resolve(releaseDir)

mkdirSync(releaseDir, { recursive: true })
rmSync(releaseDir, { recursive: true, force: true })
mkdirSync(releaseDir, { recursive: true })

if (existsSync(macos)) {
  execSync(`ditto -c -k --sequesterRsrc --keepParent Mapper.app ${releaseDirAbs}/Mapper-${version}-macos.app.zip`, {
    cwd: macos,
    stdio: 'inherit',
  })
}
if (existsSync(dmgDir)) {
  for (const file of readdirSync(dmgDir)) {
    cpSync(`${dmgDir}/${file}`, `${releaseDir}/${file}`)
  }
}