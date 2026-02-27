import type { WizardConfig } from '../pages/Wizard'
import CustomConfig from './CustomConfig'
import { toShortTemplateSlug } from '../lib/template-icons'

interface Props {
  config: WizardConfig
  onChange: (updates: Partial<WizardConfig>) => void
}

function TextInput({ label, value, onChange }: { label: string; value: string; onChange: (v: string) => void }) {
  return (
    <div>
      <label className="block text-sm font-medium text-on-surface mb-1.5">{label}</label>
      <input
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="w-full px-3 py-2 rounded-xl bg-surface-container border border-outline-variant text-on-surface text-sm placeholder-on-surface-variant/50 focus:outline-none focus:border-on-surface/30 focus:ring-1 focus:ring-on-surface/10 transition-colors"
      />
    </div>
  )
}

function Toggle({ label, checked, onChange, description }: { label: string; checked: boolean; onChange: (v: boolean) => void; description?: string }) {
  return (
    <label className="flex items-start gap-3 cursor-pointer group">
      <div className="pt-0.5">
        <div
          role="switch"
          aria-checked={checked}
          onClick={() => onChange(!checked)}
          onKeyDown={(e) => e.key === 'Enter' && onChange(!checked)}
          tabIndex={0}
          className={`w-9 h-5 rounded-full transition-colors relative ${
            checked ? 'bg-on-surface' : 'bg-surface-container-highest'
          }`}
        >
          <div
            className={`absolute top-0.5 w-4 h-4 rounded-full transition-transform shadow-sm ${
              checked ? 'translate-x-4 bg-surface' : 'translate-x-0.5 bg-on-surface'
            }`}
          />
        </div>
      </div>
      <div>
        <span className="text-sm text-on-surface-variant group-hover:text-on-surface transition-colors">{label}</span>
        {description && <p className="text-xs text-on-surface-variant/70 mt-0.5">{description}</p>}
      </div>
    </label>
  )
}

export default function ConfigPanel({ config, onChange }: Props) {
  const template = toShortTemplateSlug(config.template)

  return (
    <div className="space-y-4">
      <TextInput
        label="Namespace"
        value={config.namespace}
        onChange={(v) => onChange({ namespace: v.replace(/[^a-z0-9_]/g, '') })}
      />
      <TextInput
        label="Project Name"
        value={config.project_name}
        onChange={(v) => onChange({ project_name: v.replace(/[^a-z0-9_]/g, '') })}
      />

      {/* Template-specific options */}
      {template === 'mint' && (
        <div className="space-y-3 pt-2 border-t border-outline-variant">
          <Toggle
            label="Time Lock"
            description="Add a deadline after which minting is disabled"
            checked={config.time_lock ?? false}
            onChange={(v) => onChange({ time_lock: v })}
          />
        </div>
      )}

      {template === 'vesting' && (
        <div className="space-y-3 pt-2 border-t border-outline-variant">
          <Toggle
            label="Cancellable"
            description="Allow the owner to cancel and reclaim locked funds"
            checked={config.cancellable ?? false}
            onChange={(v) => onChange({ cancellable: v })}
          />
          <Toggle
            label="Partial Claim"
            description="Allow the beneficiary to withdraw in multiple tranches"
            checked={config.partial_claim ?? false}
            onChange={(v) => onChange({ partial_claim: v })}
          />
        </div>
      )}

      {template === 'custom' && (
        <CustomConfig config={config} onChange={onChange} />
      )}
    </div>
  )
}
