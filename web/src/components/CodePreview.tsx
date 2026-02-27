import { useEffect, useState, useRef } from 'react'
import type { GeneratedFile } from '../lib/wasm'

interface Props {
  files: GeneratedFile[]
  activeFile: string
}

async function getHighlighter() {
  const { createHighlighter } = await import('shiki')
  return createHighlighter({
    themes: ['github-dark-default'],
    langs: ['rust', 'toml', 'typescript', 'json'],
  })
}

let highlighterPromise: ReturnType<typeof getHighlighter> | null = null

function getLang(path: string): string {
  if (path.endsWith('.ak')) return 'rust' // Aiken is close enough to Rust for highlighting
  if (path.endsWith('.toml')) return 'toml'
  if (path.endsWith('.ts')) return 'typescript'
  if (path.endsWith('.json')) return 'json'
  return 'rust'
}

export default function CodePreview({ files, activeFile }: Props) {
  const [html, setHtml] = useState('')
  const [copied, setCopied] = useState(false)
  const timeoutRef = useRef<ReturnType<typeof setTimeout>>(null)

  const file = files.find((f) => f.path === activeFile)

  useEffect(() => {
    if (!file) {
      return
    }

    let cancelled = false

    ;(async () => {
      if (!highlighterPromise) {
        highlighterPromise = getHighlighter()
      }
      const hl = await highlighterPromise
      if (cancelled) return

      const result = hl.codeToHtml(file.content, {
        lang: getLang(file.path),
        theme: 'github-dark-default',
      })
      if (!cancelled) setHtml(result)
    })()

    return () => { cancelled = true }
  }, [file])

  const handleCopy = async () => {
    if (!file) return
    await navigator.clipboard.writeText(file.content)
    setCopied(true)
    if (timeoutRef.current) clearTimeout(timeoutRef.current)
    timeoutRef.current = setTimeout(() => setCopied(false), 2000)
  }

  if (!file) {
    return (
      <div className="flex items-center justify-center h-full text-white/30 text-sm">
        Select a file to preview
      </div>
    )
  }

  return (
    <div className="relative h-full">
      {/* Header */}
      <div className="sticky top-0 z-10 flex items-center justify-between px-4 py-2 bg-code-surface/90 backdrop-blur border-b border-white/5">
        <span className="text-sm text-white/50 font-mono">{file.path}</span>
        <button
          onClick={handleCopy}
          className="px-3 py-1 text-xs rounded-full border border-white/10 text-white/50 hover:text-white hover:border-white/20 transition-colors"
        >
          {copied ? 'Copied!' : 'Copy'}
        </button>
      </div>

      {/* Code */}
      <div className="p-4 overflow-auto">
        {html ? (
          <div
            className="shiki"
            dangerouslySetInnerHTML={{ __html: html }}
          />
        ) : (
          <pre className="font-mono text-sm text-white/70 whitespace-pre-wrap leading-relaxed">
            {file.content}
          </pre>
        )}
      </div>
    </div>
  )
}
