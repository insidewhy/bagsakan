#!/usr/bin/env node

// This script helps debug import resolution issues

const fs = require('fs')
const path = require('path')

console.log('Debugging import resolution for user-messaging-types...\n')

// Check if package exists
const packagePath = path.join(process.cwd(), 'node_modules', 'user-messaging-types')
if (!fs.existsSync(packagePath)) {
  console.error('❌ Package user-messaging-types not found in node_modules')
  process.exit(1)
}

console.log('✅ Found user-messaging-types in node_modules')

// Check package.json
const packageJsonPath = path.join(packagePath, 'package.json')
if (fs.existsSync(packageJsonPath)) {
  const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'))
  console.log('\n📦 Package.json contents:')
  console.log('  Main:', packageJson.main)
  console.log('  Types:', packageJson.types || packageJson.typings)

  if (packageJson.exports) {
    console.log('\n  Exports:')
    console.log(JSON.stringify(packageJson.exports, null, 4))
  }
}

// Check for entities file
console.log('\n📁 Checking for entities files:')
const possiblePaths = [
  'entities.js',
  'entities.ts',
  'entities.d.ts',
  'dist/entities.js',
  'dist/entities.d.ts',
  'lib/entities.js',
  'lib/entities.d.ts',
  'src/entities.ts',
]

possiblePaths.forEach((p) => {
  const fullPath = path.join(packagePath, p)
  if (fs.existsSync(fullPath)) {
    console.log(`  ✅ Found: ${p}`)

    // If it's a .d.ts file, check for exported interfaces
    if (p.endsWith('.d.ts')) {
      const content = fs.readFileSync(fullPath, 'utf8')
      const interfaces = content.match(/export\s+interface\s+(\w+)/g)
      if (interfaces) {
        console.log(
          `     Exported interfaces: ${interfaces.map((i) => i.replace(/export\s+interface\s+/, '')).join(', ')}`,
        )
      }
    }
  }
})

// Try to resolve using Node's require.resolve
console.log('\n🔍 Trying Node.js resolution:')
try {
  const resolved = require.resolve('user-messaging-types/entities')
  console.log('  ✅ Resolved to:', resolved)
} catch (e) {
  console.log('  ❌ Failed to resolve user-messaging-types/entities')
  console.log('     Error:', e.message)
}
