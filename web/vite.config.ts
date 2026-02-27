import type { PluginOption } from 'vite'
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import { existsSync } from 'fs'
import { resolve } from 'path'

// Stub out the WASM module if it hasn't been built yet
function wasmFallback(): PluginOption {
  const wasmPath = resolve(__dirname, 'src/wasm/kaido_core')
  const wasmExists = existsSync(wasmPath + '.js') || existsSync(wasmPath + '/kaido_core.js') || existsSync(resolve(__dirname, 'src/wasm/kaido_core.js'))

  if (wasmExists) return null

  return {
    name: 'wasm-fallback',
    resolveId(id: string) {
      if (id.includes('wasm/kaido_core')) {
        return '\0virtual:wasm-stub'
      }
    },
    load(id: string) {
      if (id === '\0virtual:wasm-stub') {
        return 'export default function init() {}; export const list_templates = () => "[]"; export const get_template_info = () => "{}"; export const list_features = () => "[]"; export const generate = () => "[]"; export const generate_sdk = () => "[]"; export const validate_custom = () => "{}";'
      }
    },
  }
}

export default defineConfig(({ command }) => ({
  plugins: [
    react(),
    tailwindcss(),
    ...(command === 'serve' ? [wasmFallback()] : []),
  ],
  build: {
    target: 'esnext',
  },
}))
