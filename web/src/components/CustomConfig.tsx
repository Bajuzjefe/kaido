import { useState, useEffect } from 'react'
import { listFeatures, listFeaturesFallback, type FeatureInfo } from '../lib/wasm'
import type { WizardConfig } from '../pages/Wizard'

interface Props {
  config: WizardConfig
  onChange: (updates: Partial<WizardConfig>) => void
}

const FIELD_TYPES = ['Int', 'ByteArray', 'Address', 'Bool', 'List<Int>', 'List<ByteArray>']

export default function CustomConfig({ config, onChange }: Props) {
  const [features, setFeatures] = useState<FeatureInfo[]>(listFeaturesFallback())
  const selectedFeatures = config.features ?? []
  const datumFields = config.datum_fields ?? []
  const redeemerActions = config.redeemer_actions ?? []

  useEffect(() => {
    listFeatures()
      .then(setFeatures)
      .catch(() => {})
  }, [])

  const toggleFeature = (name: string) => {
    let next: string[]
    if (selectedFeatures.includes(name)) {
      next = selectedFeatures.filter((f) => f !== name)
    } else {
      // Auto-add dependencies
      const feature = features.find((f) => f.name === name)
      const deps = feature?.depends_on ?? []
      const all = new Set([...selectedFeatures, name, ...deps])
      next = [...all]
    }
    onChange({ features: next })
  }

  const addDatumField = () => {
    onChange({
      datum_fields: [...datumFields, { name: '', field_type: 'Int' }],
    })
  }

  const updateDatumField = (idx: number, key: 'name' | 'field_type', value: string) => {
    const updated = datumFields.map((f, i) =>
      i === idx ? { ...f, [key]: key === 'name' ? value.replace(/[^a-z0-9_]/g, '') : value } : f
    )
    onChange({ datum_fields: updated })
  }

  const removeDatumField = (idx: number) => {
    onChange({ datum_fields: datumFields.filter((_, i) => i !== idx) })
  }

  const addRedeemerAction = () => {
    onChange({
      redeemer_actions: [...redeemerActions, { name: '', fields: [] }],
    })
  }

  const updateActionName = (idx: number, name: string) => {
    const updated = redeemerActions.map((a, i) =>
      i === idx ? { ...a, name: name.replace(/[^A-Za-z0-9]/g, '') } : a
    )
    onChange({ redeemer_actions: updated })
  }

  const removeAction = (idx: number) => {
    onChange({ redeemer_actions: redeemerActions.filter((_, i) => i !== idx) })
  }

  const addActionField = (actionIdx: number) => {
    const updated = redeemerActions.map((a, i) =>
      i === actionIdx ? { ...a, fields: [...a.fields, { name: '', field_type: 'Int' }] } : a
    )
    onChange({ redeemer_actions: updated })
  }

  const updateActionField = (actionIdx: number, fieldIdx: number, key: 'name' | 'field_type', value: string) => {
    const updated = redeemerActions.map((a, i) =>
      i === actionIdx
        ? {
            ...a,
            fields: a.fields.map((f, j) =>
              j === fieldIdx ? { ...f, [key]: key === 'name' ? value.replace(/[^a-z0-9_]/g, '') : value } : f
            ),
          }
        : a
    )
    onChange({ redeemer_actions: updated })
  }

  const removeActionField = (actionIdx: number, fieldIdx: number) => {
    const updated = redeemerActions.map((a, i) =>
      i === actionIdx ? { ...a, fields: a.fields.filter((_, j) => j !== fieldIdx) } : a
    )
    onChange({ redeemer_actions: updated })
  }

  const selectCls = "px-3 py-2 rounded-xl bg-surface-container border border-outline-variant text-on-surface text-sm focus:outline-none focus:border-on-surface/30 transition-colors"
  const smallInputCls = "flex-1 px-2.5 py-1.5 rounded-lg bg-surface-container border border-outline-variant text-on-surface text-sm placeholder-on-surface-variant/50 focus:outline-none focus:border-on-surface/30"
  const smallSelectCls = "px-2.5 py-1.5 rounded-lg bg-surface-container border border-outline-variant text-on-surface text-sm focus:outline-none focus:border-on-surface/30"

  return (
    <div className="space-y-5 pt-2 border-t border-outline-variant">
      <div>
        <label className="block text-sm font-medium text-on-surface mb-1.5">Purpose</label>
        <select
          value={config.purpose ?? 'spend'}
          onChange={(e) => onChange({ purpose: e.target.value as 'spend' | 'mint' })}
          className={selectCls + " w-full"}
        >
          <option value="spend">spend</option>
          <option value="mint">mint</option>
        </select>
      </div>

      {/* Features */}
      <div>
        <label className="block text-sm font-medium text-on-surface mb-2">Features</label>
        <div className="space-y-2">
          {features.map((f) => {
            const checked = selectedFeatures.includes(f.name)
            const isDep = selectedFeatures.some((sel) => {
              const feat = features.find((x) => x.name === sel)
              return feat?.depends_on.includes(f.name)
            })
            return (
              <label
                key={f.name}
                className="flex items-start gap-2.5 cursor-pointer group"
              >
                <input
                  type="checkbox"
                  checked={checked}
                  disabled={isDep && checked}
                  onChange={() => toggleFeature(f.name)}
                  className="mt-0.5 rounded accent-[var(--color-on-surface)]"
                />
                <div>
                  <span className="text-sm text-on-surface-variant group-hover:text-on-surface transition-colors">
                    {f.name}
                  </span>
                  {f.purpose && (
                    <span className="ml-1.5 text-xs text-on-surface-variant/60 bg-surface-container-high px-1.5 py-0.5 rounded">
                      {f.purpose}
                    </span>
                  )}
                  <p className="text-xs text-on-surface-variant/70 mt-0.5">{f.description}</p>
                  {f.depends_on.length > 0 && (
                    <p className="text-xs text-on-surface-variant/50 mt-0.5">
                      Requires: {f.depends_on.join(', ')}
                    </p>
                  )}
                </div>
              </label>
            )
          })}
        </div>
      </div>

      {/* Datum Fields */}
      <div>
        <div className="flex items-center justify-between mb-2">
          <label className="text-sm font-medium text-on-surface">Datum Fields</label>
          <button
            onClick={addDatumField}
            className="text-xs text-on-surface-variant hover:text-on-surface transition-colors"
          >
            + Add Field
          </button>
        </div>
        <div className="space-y-2">
          {datumFields.map((field, idx) => (
            <div key={idx} className="flex items-center gap-2">
              <input
                type="text"
                placeholder="field_name"
                value={field.name}
                onChange={(e) => updateDatumField(idx, 'name', e.target.value)}
                className={smallInputCls}
              />
              <select
                value={field.field_type}
                onChange={(e) => updateDatumField(idx, 'field_type', e.target.value)}
                className={smallSelectCls}
              >
                {FIELD_TYPES.map((t) => (
                  <option key={t} value={t}>{t}</option>
                ))}
              </select>
              <button
                onClick={() => removeDatumField(idx)}
                className="text-on-surface-variant/50 hover:text-red-500 text-sm transition-colors"
              >
                x
              </button>
            </div>
          ))}
        </div>
      </div>

      {/* Redeemer Actions */}
      <div>
        <div className="flex items-center justify-between mb-2">
          <label className="text-sm font-medium text-on-surface">Redeemer Actions</label>
          <button
            onClick={addRedeemerAction}
            className="text-xs text-on-surface-variant hover:text-on-surface transition-colors"
          >
            + Add Action
          </button>
        </div>
        <div className="space-y-3">
          {redeemerActions.map((action, aIdx) => (
            <div key={aIdx} className="rounded-xl border border-outline-variant bg-surface-container p-3">
              <div className="flex items-center gap-2 mb-2">
                <input
                  type="text"
                  placeholder="ActionName"
                  value={action.name}
                  onChange={(e) => updateActionName(aIdx, e.target.value)}
                  className={smallInputCls}
                />
                <button
                  onClick={() => removeAction(aIdx)}
                  className="text-on-surface-variant/50 hover:text-red-500 text-sm transition-colors"
                >
                  x
                </button>
              </div>
              <div className="space-y-1.5 ml-2">
                {action.fields.map((field, fIdx) => (
                  <div key={fIdx} className="flex items-center gap-2">
                    <input
                      type="text"
                      placeholder="field_name"
                      value={field.name}
                      onChange={(e) => updateActionField(aIdx, fIdx, 'name', e.target.value)}
                      className="flex-1 px-2 py-1 rounded-lg bg-surface border border-outline-variant text-on-surface text-xs placeholder-on-surface-variant/50 focus:outline-none focus:border-on-surface/30"
                    />
                    <select
                      value={field.field_type}
                      onChange={(e) => updateActionField(aIdx, fIdx, 'field_type', e.target.value)}
                      className="px-2 py-1 rounded-lg bg-surface border border-outline-variant text-on-surface text-xs focus:outline-none focus:border-on-surface/30"
                    >
                      {FIELD_TYPES.map((t) => (
                        <option key={t} value={t}>{t}</option>
                      ))}
                    </select>
                    <button
                      onClick={() => removeActionField(aIdx, fIdx)}
                      className="text-on-surface-variant/50 hover:text-red-500 text-xs transition-colors"
                    >
                      x
                    </button>
                  </div>
                ))}
                <button
                  onClick={() => addActionField(aIdx)}
                  className="text-xs text-on-surface-variant/60 hover:text-on-surface transition-colors"
                >
                  + field
                </button>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}
