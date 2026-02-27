/* eslint-disable react-refresh/only-export-components */
import type { JSX } from 'react'

const s = { fill: 'none', stroke: 'currentColor', strokeWidth: 1.5, strokeLinecap: 'round' as const, strokeLinejoin: 'round' as const }

export const TEMPLATE_ICONS: Record<string, JSX.Element> = {
  // Coin
  mint: (
    <svg width="20" height="20" viewBox="0 0 24 24" {...s}>
      <circle cx="12" cy="12" r="9" />
      <path d="M14.5 9.5a3 3 0 10-5 2.24V15h2v2h2v-2h1v-2h-3v-1.26a3 3 0 003-2.24" />
    </svg>
  ),
  // Hourglass
  vesting: (
    <svg width="20" height="20" viewBox="0 0 24 24" {...s}>
      <path d="M6 2h12M6 22h12M7 2v4l5 5-5 5v4M17 2v4l-5 5 5 5v4" />
    </svg>
  ),
  // Lock
  escrow: (
    <svg width="20" height="20" viewBox="0 0 24 24" {...s}>
      <rect x="5" y="11" width="14" height="10" rx="2" />
      <path d="M8 11V7a4 4 0 018 0v4" />
    </svg>
  ),
  // Vault
  treasury: (
    <svg width="20" height="20" viewBox="0 0 24 24" {...s}>
      <rect x="3" y="4" width="18" height="16" rx="2" />
      <circle cx="12" cy="12" r="3" />
      <path d="M12 9V4M12 20v-5M15 12h6M3 12h6" />
    </svg>
  ),
  // Exchange arrows
  dex: (
    <svg width="20" height="20" viewBox="0 0 24 24" {...s}>
      <path d="M7 16l-4-4 4-4" />
      <path d="M3 12h18" />
      <path d="M17 8l4 4-4 4" />
    </svg>
  ),
  // Percentage
  lending: (
    <svg width="20" height="20" viewBox="0 0 24 24" {...s}>
      <circle cx="8" cy="8" r="2" />
      <circle cx="16" cy="16" r="2" />
      <path d="M6 18L18 6" />
    </svg>
  ),
  // Layers
  staking: (
    <svg width="20" height="20" viewBox="0 0 24 24" {...s}>
      <path d="M12 2L2 7l10 5 10-5-10-5z" />
      <path d="M2 17l10 5 10-5" />
      <path d="M2 12l10 5 10-5" />
    </svg>
  ),
  // Wave/flow
  streaming: (
    <svg width="20" height="20" viewBox="0 0 24 24" {...s}>
      <path d="M2 12c2-3 4-3 6 0s4 3 6 0 4-3 6 0" />
      <path d="M2 7c2-3 4-3 6 0s4 3 6 0 4-3 6 0" />
      <path d="M2 17c2-3 4-3 6 0s4 3 6 0 4-3 6 0" />
    </svg>
  ),
  // Store tag
  marketplace: (
    <svg width="20" height="20" viewBox="0 0 24 24" {...s}>
      <path d="M20.59 13.41l-7.17 7.17a2 2 0 01-2.83 0L2 12V2h10l8.59 8.59a2 2 0 010 2.82z" />
      <circle cx="7" cy="7" r="1.5" fill="currentColor" stroke="none" />
    </svg>
  ),
  // Eye
  oracle: (
    <svg width="20" height="20" viewBox="0 0 24 24" {...s}>
      <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8S1 12 1 12z" />
      <circle cx="12" cy="12" r="3" />
    </svg>
  ),
  // Ballot/checkmark
  governance: (
    <svg width="20" height="20" viewBox="0 0 24 24" {...s}>
      <rect x="4" y="3" width="16" height="18" rx="2" />
      <path d="M9 12l2 2 4-4" />
    </svg>
  ),
  // Share/users
  referral: (
    <svg width="20" height="20" viewBox="0 0 24 24" {...s}>
      <circle cx="18" cy="5" r="3" />
      <circle cx="6" cy="12" r="3" />
      <circle cx="18" cy="19" r="3" />
      <path d="M8.59 13.51l6.83 3.98M15.41 6.51l-6.82 3.98" />
    </svg>
  ),
  // Gear
  custom: (
    <svg width="20" height="20" viewBox="0 0 24 24" {...s}>
      <circle cx="12" cy="12" r="3" />
      <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z" />
    </svg>
  ),
}

// Map WASM long slugs to short icon keys
const SLUG_ALIASES: Record<string, string> = {
  simple_mint: 'mint',
  multisig_treasury: 'treasury',
  nft_marketplace: 'marketplace',
  staking_pool: 'staking',
  oracle_settlement: 'oracle',
  referral_system: 'referral',
  dex_pool: 'dex',
  lending_pool: 'lending',
  dao_governance: 'governance',
  streaming_payments: 'streaming',
}

const REVERSE_SLUG_ALIASES: Record<string, string> = Object.fromEntries(
  Object.entries(SLUG_ALIASES).map(([longSlug, shortSlug]) => [shortSlug, longSlug])
)

function normalizeSlug(slug: string): string {
  return slug.toLowerCase().replace(/-/g, '_')
}

export function toShortTemplateSlug(slug: string): string {
  const normalized = normalizeSlug(slug)
  return SLUG_ALIASES[normalized] ?? normalized
}

export function toLongTemplateSlug(slug: string): string {
  const short = toShortTemplateSlug(slug)
  return REVERSE_SLUG_ALIASES[short] ?? short
}

export function getTemplateIcon(slug: string) {
  const shortSlug = toShortTemplateSlug(slug)
  return TEMPLATE_ICONS[shortSlug] ?? TEMPLATE_ICONS['']
}

// Short display name for wizard
export function getTemplateDisplayName(slug: string): string {
  return toShortTemplateSlug(slug).replace(/_/g, ' ')
}

export function TemplateIcon({ slug, size = 20 }: { slug: string; size?: number }) {
  const icon = getTemplateIcon(slug)
  if (!icon) return <span className="font-mono font-bold">{slug.charAt(0).toUpperCase()}</span>
  if (size === 20) return icon
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.5} strokeLinecap="round" strokeLinejoin="round">
      {icon.props.children}
    </svg>
  )
}
