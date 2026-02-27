import { useState, useEffect, useCallback, useRef } from 'react'
import { useSearchParams } from 'react-router-dom'
import TemplateSelector from '../components/TemplateSelector'
import ConfigPanel from '../components/ConfigPanel'
import CodePreview from '../components/CodePreview'
import FileTree from '../components/FileTree'
import DownloadButton from '../components/DownloadButton'
import CollapsibleSection from '../components/CollapsibleSection'
import { getTemplateDisplayName, toLongTemplateSlug, toShortTemplateSlug } from '../lib/template-icons'
import {
  listTemplates,
  listTemplatesFallback,
  generate,
  generateSdk,
  validateCustom,
  type GeneratedFile,
  type TemplateInfo,
} from '../lib/wasm'

export interface WizardConfig {
  template: string
  namespace: string
  project_name: string
  purpose?: 'spend' | 'mint'
  time_lock?: boolean
  cancellable?: boolean
  partial_claim?: boolean
  features?: string[]
  datum_fields?: Array<{ name: string; field_type: string }>
  redeemer_actions?: Array<{ name: string; fields: Array<{ name: string; field_type: string }> }>
  generate_sdk?: boolean
}

const DEFAULT_CONFIG: WizardConfig = {
  template: 'mint',
  namespace: 'myorg',
  project_name: 'my_token',
  purpose: 'spend',
}

function serializeDatumFields(fields: Array<{ name: string; field_type: string }>): string {
  return fields
    .filter((f) => f.name.trim().length > 0)
    .map((f) => `${f.name}:${f.field_type}`)
    .join(',')
}

function serializeRedeemerActions(
  actions: Array<{ name: string; fields: Array<{ name: string; field_type: string }> }>
): string {
  return actions
    .filter((a) => a.name.trim().length > 0)
    .map((a) => {
      if (!a.fields.length) return a.name
      const fieldStr = a.fields
        .filter((f) => f.name.trim().length > 0)
        .map((f) => `${f.name}:${f.field_type}`)
        .join(',')
      return fieldStr.length > 0 ? `${a.name}(${fieldStr})` : a.name
    })
    .join(',')
}

export default function Wizard() {
  const [searchParams, setSearchParams] = useSearchParams()
  const [templates, setTemplates] = useState<TemplateInfo[]>(listTemplatesFallback())
  const [config, setConfig] = useState<WizardConfig>(() => {
    const t = searchParams.get('template')
    return { ...DEFAULT_CONFIG, ...(t ? { template: t } : {}) }
  })
  const [files, setFiles] = useState<GeneratedFile[]>([])
  const [sdkFiles, setSdkFiles] = useState<GeneratedFile[]>([])
  const [activeFile, setActiveFile] = useState<string>('')
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  const templateSupportsSdk = useCallback(
    (slug: string) => {
      const shortSlug = toShortTemplateSlug(slug)
      return (
        templates.find((t) => toShortTemplateSlug(t.slug) === shortSlug)?.supports_sdk ?? false
      )
    },
    [templates]
  )

  const supportedSdkTemplates = templates
    .filter((t) => t.supports_sdk)
    .map((t) => getTemplateDisplayName(t.slug))

  // Sidebar collapse
  const [sidebarOpen, setSidebarOpen] = useState(true)

  // Resizable file tree
  const [fileTreeHeight, setFileTreeHeight] = useState(192)
  const dragging = useRef(false)
  const dragStartY = useRef(0)
  const dragStartHeight = useRef(0)

  // Keyboard shortcut: Cmd+B / Ctrl+B toggles sidebar
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'b') {
        e.preventDefault()
        setSidebarOpen((prev) => !prev)
      }
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [])

  // Resize drag handlers
  useEffect(() => {
    const onMove = (e: PointerEvent) => {
      if (!dragging.current) return
      const delta = e.clientY - dragStartY.current
      const next = Math.min(400, Math.max(80, dragStartHeight.current + delta))
      setFileTreeHeight(next)
    }
    const onUp = () => {
      dragging.current = false
      document.body.style.cursor = ''
      document.body.style.userSelect = ''
    }
    window.addEventListener('pointermove', onMove)
    window.addEventListener('pointerup', onUp)
    return () => {
      window.removeEventListener('pointermove', onMove)
      window.removeEventListener('pointerup', onUp)
    }
  }, [])

  const startDrag = (e: React.PointerEvent) => {
    e.preventDefault()
    ;(e.target as HTMLElement).setPointerCapture(e.pointerId)
    dragging.current = true
    dragStartY.current = e.clientY
    dragStartHeight.current = fileTreeHeight
    document.body.style.cursor = 'row-resize'
    document.body.style.userSelect = 'none'
  }

  // Load real templates from WASM
  useEffect(() => {
    listTemplates()
      .then(setTemplates)
      .catch(() => {}) // fallback already set
  }, [])

  // Track activeFile via ref to avoid regenerate depending on it
  const activeFileRef = useRef(activeFile)
  activeFileRef.current = activeFile

  // Regenerate on config change
  const regenerate = useCallback(async (cfg: WizardConfig) => {
    setLoading(true)
    setError(null)
    try {
      const options: Record<string, unknown> = {
        template: toLongTemplateSlug(cfg.template),
        namespace: cfg.namespace,
        project_name: cfg.project_name,
      }
      if (toShortTemplateSlug(cfg.template) === 'mint' && cfg.time_lock) options.time_lock = true
      if (toShortTemplateSlug(cfg.template) === 'vesting') {
        if (cfg.cancellable) options.cancellable = true
        if (cfg.partial_claim) options.partial_claim = true
      }
      if (toShortTemplateSlug(cfg.template) === 'custom') {
        options.purpose = cfg.purpose ?? 'spend'
        options.features = (cfg.features ?? []).join(',')
        options.datum = serializeDatumFields(cfg.datum_fields ?? [])
        options.redeemer = serializeRedeemerActions(cfg.redeemer_actions ?? [])

        const validation = await validateCustom(options)
        if (!validation.valid) {
          throw new Error(validation.errors.join('\n'))
        }
      }

      const generated = await generate(options)
      setFiles(generated)
      if (generated.length > 0 && !activeFileRef.current) {
        // Default to the validator file
        const validator = generated.find((f) => f.path.startsWith('validators/'))
        setActiveFile(validator?.path ?? generated[0].path)
      }

      if (cfg.generate_sdk && templateSupportsSdk(cfg.template)) {
        const sdk = await generateSdk(options)
        setSdkFiles(sdk)
      } else {
        setSdkFiles([])
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setLoading(false)
    }
  }, [templateSupportsSdk])

  useEffect(() => {
    regenerate(config)
  }, [config, regenerate])

  useEffect(() => {
    if (config.generate_sdk && !templateSupportsSdk(config.template)) {
      setConfig((prev) => ({ ...prev, generate_sdk: false }))
    }
  }, [config.generate_sdk, config.template, templateSupportsSdk])

  const updateConfig = (updates: Partial<WizardConfig>) => {
    const next = { ...config, ...updates }
    if (updates.template) {
      const shortSlug = toShortTemplateSlug(updates.template)
      next.template = shortSlug
      if (next.generate_sdk && !templateSupportsSdk(shortSlug)) {
        next.generate_sdk = false
      }
      setSearchParams({ template: shortSlug })
      setActiveFile('')
    }
    setConfig(next)
  }

  const allFiles = [...files, ...sdkFiles]
  const supportsSdk = templateSupportsSdk(config.template)

  return (
    <div className="h-[calc(100vh-3.5rem)] flex">
      {/* Left panel */}
      <div
        className={`flex-shrink-0 border-r border-outline-variant overflow-y-auto bg-surface transition-all duration-200 ${
          sidebarOpen ? 'w-[420px]' : 'w-12'
        }`}
      >
        {sidebarOpen ? (
          <div className="p-6 space-y-6">
            <div className="flex items-start justify-between">
              <div>
                <h2 className="text-lg font-medium text-on-surface mb-0.5">Contract Wizard</h2>
                <p className="text-sm text-on-surface-variant font-light">Configure and generate Aiken smart contracts</p>
              </div>
              <button
                onClick={() => setSidebarOpen(false)}
                className="p-1.5 rounded-lg text-on-surface-variant hover:text-on-surface hover:bg-white/[0.08] transition-colors"
                title="Collapse sidebar (Cmd+B)"
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <polyline points="15 18 9 12 15 6" />
                </svg>
              </button>
            </div>

            <CollapsibleSection title="Template" defaultOpen={true}>
              <TemplateSelector
                templates={templates}
                selected={config.template}
                onSelect={(slug) => updateConfig({ template: toShortTemplateSlug(slug) })}
              />
            </CollapsibleSection>

            <CollapsibleSection title="Configuration" defaultOpen={true}>
              <ConfigPanel config={config} onChange={updateConfig} />
            </CollapsibleSection>

            <CollapsibleSection title="Options" defaultOpen={true}>
              <label className="flex items-center gap-2.5 text-sm text-on-surface-variant cursor-pointer hover:text-on-surface transition-colors">
                <input
                  type="checkbox"
                  checked={config.generate_sdk ?? false}
                  onChange={(e) => updateConfig({ generate_sdk: e.target.checked })}
                  disabled={!supportsSdk}
                  className="rounded accent-[var(--color-on-surface)]"
                />
                Generate TypeScript SDK
              </label>
              {!supportsSdk && (
                <p className="mt-2 text-xs text-on-surface-variant/70">
                  SDK generation is not available for this template yet.
                </p>
              )}
              {!supportsSdk && supportedSdkTemplates.length > 0 && (
                <p className="mt-1 text-xs text-on-surface-variant/60">
                  Currently supported: {supportedSdkTemplates.join(', ')}.
                </p>
              )}
            </CollapsibleSection>

            <DownloadButton files={allFiles} projectName={config.project_name} />

            <p className="text-[11px] text-on-surface-variant/40 leading-relaxed">
              Generated contracts are starting points for development and testing. Not intended for direct mainnet deployment without independent security review.
            </p>

            {error && (
              <div className="rounded-2xl bg-red-950/30 border border-red-900/50 px-4 py-3 text-sm text-red-400">
                {error}
              </div>
            )}
          </div>
        ) : (
          <div className="flex flex-col items-center pt-3 gap-2">
            <button
              onClick={() => setSidebarOpen(true)}
              className="p-1.5 rounded-lg text-on-surface-variant hover:text-on-surface hover:bg-white/[0.08] transition-colors"
              title="Expand sidebar (Cmd+B)"
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <polyline points="9 18 15 12 9 6" />
              </svg>
            </button>
          </div>
        )}
      </div>

      {/* Right panel */}
      <div className="flex-1 flex flex-col min-w-0 bg-code-surface">
        {/* File tree bar */}
        <div className="border-b border-code-border bg-code-surface">
          <FileTree
            files={allFiles}
            activeFile={activeFile}
            onSelect={setActiveFile}
            maxHeight={fileTreeHeight}
          />
        </div>

        {/* Drag handle */}
        <div
          onPointerDown={startDrag}
          className="h-2 bg-code-border hover:bg-white/15 cursor-row-resize transition-colors flex items-center justify-center group select-none"
          style={{ touchAction: 'none' }}
        >
          <div className="w-10 h-0.5 rounded-full bg-white/15 group-hover:bg-white/40 transition-colors" />
        </div>

        {/* Code preview */}
        <div className="flex-1 overflow-auto">
          {loading ? (
            <div className="flex items-center justify-center h-full text-white/40">
              <div className="animate-pulse">Generating...</div>
            </div>
          ) : (
            <CodePreview
              files={allFiles}
              activeFile={activeFile}
            />
          )}
        </div>
      </div>
    </div>
  )
}
