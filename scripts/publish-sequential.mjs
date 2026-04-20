import fs from 'node:fs'
import path from 'node:path'
import process from 'node:process'
import { execFileSync } from 'node:child_process'

const ROOT_DIR = process.cwd()
const NPM_DIR = path.join(ROOT_DIR, 'npm')
const ROOT_PACKAGE = path.join(ROOT_DIR, 'package.json')
const NPM_COMMAND = process.platform === 'win32' ? 'npm.cmd' : 'npm'

const MAX_ATTEMPTS = Number.parseInt(process.env.NPM_PUBLISH_MAX_ATTEMPTS ?? '6', 10)
const WAIT_BETWEEN_PACKAGES_MS = Number.parseInt(process.env.NPM_PUBLISH_WAIT_MS ?? '45000', 10)
const INITIAL_RETRY_DELAY_MS = Number.parseInt(process.env.NPM_PUBLISH_RETRY_DELAY_MS ?? '120000', 10)
const DRY_RUN = process.env.NPM_PUBLISH_DRY_RUN === '1'

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

function runJson(command, args, options = {}) {
  const output = execFileSync(command, args, {
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
    ...options,
  })

  return JSON.parse(output)
}

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, 'utf8'))
}

function packageExists(name, version) {
  try {
    const output = execFileSync(NPM_COMMAND, ['view', `${name}@${version}`, 'version', '--json'], {
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'pipe'],
      shell: process.platform === 'win32',
    }).trim()

    if (!output) {
      return false
    }

    const parsed = JSON.parse(output)
    if (Array.isArray(parsed)) {
      return parsed.includes(version)
    }

    return parsed === version
  } catch {
    return false
  }
}

function shouldRetry(error) {
  const stderr = `${error.stderr ?? ''}\n${error.stdout ?? ''}\n${error.message ?? ''}`
  return /E429|Too Many Requests|rate limited|ECONNRESET|ETIMEDOUT|EAI_AGAIN|503|504/i.test(stderr)
}

function publishPackage(dir, tag, access) {
  const args = ['publish', '--ignore-scripts']

  if (tag) {
    args.push('--tag', tag)
  }

  if (access) {
    args.push('--access', access)
  }

  if (DRY_RUN) {
    console.log(`[dry-run] ${NPM_COMMAND} ${args.join(' ')} (cwd=${dir})`)
    return
  }

  try {
    const output = execFileSync(NPM_COMMAND, args, {
      cwd: dir,
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'pipe'],
      env: process.env,
      shell: process.platform === 'win32',
    })

    if (output) {
      process.stdout.write(output)
    }
  } catch (error) {
    if (error.stdout) {
      process.stdout.write(error.stdout)
    }

    if (error.stderr) {
      process.stderr.write(error.stderr)
    }

    throw error
  }
}

function ensurePublishable(dir) {
  const pkg = readJson(path.join(dir, 'package.json'))
  for (const entry of pkg.files ?? []) {
    const target = path.join(dir, entry)
    if (!fs.existsSync(target)) {
      throw new Error(`Missing publish file for ${pkg.name}: ${target}`)
    }
  }
  return pkg
}

async function publishWithRetry(dir, tag, access) {
  const pkg = ensurePublishable(dir)
  const label = `${pkg.name}@${pkg.version}`

  if (packageExists(pkg.name, pkg.version)) {
    console.log(`Skipping existing package ${label}`)
    return false
  }

  for (let attempt = 1; attempt <= MAX_ATTEMPTS; attempt += 1) {
    try {
      console.log(`Publishing ${label} from ${dir} (attempt ${attempt}/${MAX_ATTEMPTS})`)
      publishPackage(dir, tag, access)
      console.log(`Published ${label}`)
      return true
    } catch (error) {
      if (packageExists(pkg.name, pkg.version)) {
        console.log(`Package ${label} is now available after publish attempt, continuing`)
        return true
      }

      if (attempt === MAX_ATTEMPTS || !shouldRetry(error)) {
        throw error
      }

      const delay = INITIAL_RETRY_DELAY_MS * 2 ** (attempt - 1)
      console.log(`Retrying ${label} in ${delay}ms due to transient publish failure`)
      await sleep(delay)
    }
  }
}

function getReleaseTag() {
  const releaseTag = process.env.NPM_RELEASE_TAG?.trim()
  if (releaseTag) {
    if (!/^[a-z0-9][a-z0-9._-]*$/i.test(releaseTag)) {
      throw new Error(`Invalid npm release tag: ${releaseTag}`)
    }

    return releaseTag
  }

  const commitMessage = execFileSync('git', ['log', '-1', '--pretty=%B'], {
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  }).trim()

  if (/^\d+\.\d+\.\d+$/.test(commitMessage)) {
    return 'latest'
  }

  if (/^\d+\.\d+\.\d+/.test(commitMessage)) {
    return 'next'
  }

  return null
}

async function main() {
  if (!fs.existsSync(ROOT_PACKAGE)) {
    throw new Error('package.json not found')
  }

  if (!fs.existsSync(NPM_DIR)) {
    throw new Error('npm directory not found')
  }

  const tag = getReleaseTag()
  if (!tag) {
    console.log('Not a release commit, skipping publish')
    return
  }

  const access = 'public'
  const packageDirs = fs
    .readdirSync(NPM_DIR, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => path.join(NPM_DIR, entry.name))
    .sort((left, right) => {
      const a = readJson(path.join(left, 'package.json')).name
      const b = readJson(path.join(right, 'package.json')).name
      return a.localeCompare(b)
    })

  for (const dir of packageDirs) {
    const published = await publishWithRetry(dir, tag, access)
    if (published) {
      await sleep(WAIT_BETWEEN_PACKAGES_MS)
    }
  }

  await publishWithRetry(ROOT_DIR, tag, access)
}

main().catch((error) => {
  console.error(error)
  process.exit(1)
})
