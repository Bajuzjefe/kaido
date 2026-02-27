import { useState } from 'react'
import type { TemplateInfo } from '../lib/wasm'
import { getTemplateIcon, getTemplateDisplayName, toShortTemplateSlug } from '../lib/template-icons'

interface Props {
  templates: TemplateInfo[]
  selected: string
  onSelect: (slug: string) => void
}

export default function TemplateSelector({ templates, selected, onSelect }: Props) {
  const [expanded, setExpanded] = useState(true)
  const selectedShort = toShortTemplateSlug(selected)

  const selectedTemplate = templates.find((t) => toShortTemplateSlug(t.slug) === selectedShort)
  const selectedIcon = getTemplateIcon(selected)
  const selectedName = getTemplateDisplayName(selected)

  if (!expanded && selectedTemplate) {
    return (
      <div>
        <label className="block text-sm font-medium text-on-surface mb-2">Template</label>
        <button
          onClick={() => setExpanded(true)}
          className="w-full flex items-center gap-3 p-3 rounded-xl border border-on-surface bg-on-surface text-on-primary transition-all duration-150 cursor-pointer"
        >
          <span className="flex items-center justify-center w-6 h-6">
            {selectedIcon || <span className="font-mono font-bold text-lg">{selected.charAt(0).toUpperCase()}</span>}
          </span>
          <span className="text-sm font-medium capitalize flex-1 text-left">{selectedName}</span>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="opacity-50">
            <polyline points="6 9 12 15 18 9" />
          </svg>
        </button>
      </div>
    )
  }

  return (
    <div>
      <label className="block text-sm font-medium text-on-surface mb-2">Template</label>
      <div className="grid grid-cols-3 gap-2">
        {templates.map((t) => {
          const active = toShortTemplateSlug(t.slug) === selectedShort
          const icon = getTemplateIcon(t.slug)
          const displayName = getTemplateDisplayName(t.slug)
          return (
            <button
              key={t.slug}
              onClick={() => {
                onSelect(t.slug)
                setExpanded(false)
              }}
              title={t.description}
              className={`flex flex-col items-center gap-1 p-3 rounded-xl border text-center transition-all duration-150 cursor-pointer ${
                active
                  ? 'border-on-surface bg-on-surface text-on-primary'
                  : 'border-outline-variant bg-surface-container text-on-surface-variant hover:bg-surface-container-high hover:text-on-surface'
              }`}
            >
              <span className="flex items-center justify-center w-5 h-5">
                {icon || <span className="font-mono font-bold text-lg">{t.slug.charAt(0).toUpperCase()}</span>}
              </span>
              <span className="text-xs capitalize">{displayName}</span>
            </button>
          )
        })}
      </div>
    </div>
  )
}
