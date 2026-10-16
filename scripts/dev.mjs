import { spawn } from 'node:child_process'
const windows = process.platform === 'win32'
const children = [
  spawn('cargo', ['run', '--manifest-path', 'backend/Cargo.toml'], { stdio: 'inherit' }),
  spawn(windows ? 'npm.cmd' : 'npm', ['--prefix', 'frontend', 'run', 'dev'], {
    stdio: 'inherit',
    shell: windows,
  }),
]
let stopping = false
function stop(code = 0) {
  if (stopping) return
  stopping = true
  for (const child of children) child.kill()
  process.exitCode = code
}
for (const child of children) {
  child.on('error', (error) => {
    console.error(error.message)
    stop(1)
  })
  child.on('exit', (code) => stop(code || 0))
}
process.on('SIGINT', () => stop())
process.on('SIGTERM', () => stop())
