import { useState, type ReactNode } from 'react'

interface Props {
  title: string
  defaultOpen?: boolean
  children: ReactNode
}

export default function CollapsibleSection({ title, defaultOpen = true, children }: Props) {
  const [open, setOpen] = useState(defaultOpen)

  return (
    <div>
      <button
        onClick={() => setOpen(!open)}
        className="flex items-center justify-between w-full text-left group"
      >
        <span className="text-xs font-medium uppercase tracking-wider text-on-surface-variant group-hover:text-on-surface transition-colors">
          {title}
        </span>
        <svg
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          className={`text-on-surface-variant transition-transform duration-200 ${open ? 'rotate-0' : '-rotate-90'}`}
        >
          <polyline points="6 9 12 15 18 9" />
        </svg>
      </button>
      <div
        className={`grid transition-[grid-template-rows] duration-200 ${open ? 'grid-rows-[1fr]' : 'grid-rows-[0fr]'}`}
      >
        <div className="overflow-hidden">
          <div className="pt-3">
            {children}
          </div>
        </div>
      </div>
    </div>
  )
}
