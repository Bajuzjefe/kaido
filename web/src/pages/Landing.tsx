import { Link } from 'react-router-dom'
import { useRef, useEffect } from 'react'
import { listTemplatesFallback, type TemplateInfo } from '../lib/wasm'
import { TEMPLATE_ICONS } from '../lib/template-icons'
import ParticleRing from '../components/ParticleRing'

// Scroll-triggered reveal using Intersection Observer
function useScrollReveal() {
  const ref = useRef<HTMLDivElement>(null)
  useEffect(() => {
    const el = ref.current
    if (!el) return
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          el.classList.add('revealed')
          observer.unobserve(el)
        }
      },
      { threshold: 0.15 }
    )
    observer.observe(el)
    return () => observer.disconnect()
  }, [])
  return ref
}

function RevealSection({ children, className = '' }: { children: React.ReactNode; className?: string }) {
  const ref = useScrollReveal()
  return (
    <div ref={ref} className={`scroll-reveal ${className}`}>
      {children}
    </div>
  )
}

const TEMPLATE_CATEGORIES = [
  {
    title: 'DeFi Primitives',
    desc: 'Build minting policies, vesting schedules, escrow contracts, and multisig treasuries with security-focused baseline checks.',
    templates: ['mint', 'vesting', 'escrow', 'treasury'],
  },
  {
    title: 'Protocol Infrastructure',
    desc: 'Launch DEX pools, lending markets, staking systems, and streaming payments with robust starter templates.',
    templates: ['dex', 'lending', 'staking', 'streaming'],
  },
  {
    title: 'Application Layer',
    desc: 'Deploy NFT marketplaces, oracle settlements, governance systems, and referral networks, all aikido-verified.',
    templates: ['marketplace', 'oracle', 'governance', 'referral'],
  },
]

const FEATURES = [
  {
    title: 'Security-Focused by Construction',
    desc: 'Templates include baseline checks for reference script injection, datum continuity, signature verification, and value preservation patterns.',
  },
  {
    title: 'Composable Security Features',
    desc: 'Mix and match features like signature-auth, timelocks, datum-continuity, and value-preservation. Dependencies auto-resolve. Conflicts are caught at generation time.',
  },
  {
    title: 'Instant Browser Preview',
    desc: 'The web wizard runs kaido-core compiled to WebAssembly. Every configuration change regenerates Aiken code instantly. No backend, no waiting, no account needed.',
  },
  {
    title: 'TypeScript SDK Generation',
    desc: 'Generate typed transaction builders alongside your Aiken contracts. Built for the Anvil API with CIP-30 wallet integration.',
  },
  {
    title: 'CLI + MCP Server',
    desc: 'Use the Rust CLI for local generation with full aiken build + check + aikido scan verification. Or integrate via MCP for AI-assisted contract development.',
  },
]

const FEATURE_ICONS = [
  <svg key="shield" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/><path d="M9 12l2 2 4-4"/></svg>,
  <svg key="puzzle" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="M20 7h-3a2 2 0 01-2-2V2"/><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V9l-6-7z"/><circle cx="12" cy="13" r="2"/></svg>,
  <svg key="zap" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z"/></svg>,
  <svg key="code" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>,
  <svg key="terminal" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5"><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></svg>,
]

// Icons for the floating circles section
const FLOATING_ICON_SLUGS = [
  'mint', 'vesting', 'escrow', 'treasury', 'dex', 'lending',
  'staking', 'streaming', 'marketplace', 'oracle', 'governance',
  'referral', 'custom',
]

function TemplateCard({ template }: { template: TemplateInfo }) {
  const icon = TEMPLATE_ICONS[template.slug]
  return (
    <Link
      to={`/wizard?template=${template.slug}`}
      className="group flex items-start gap-3 p-4 rounded-2xl bg-white/[0.05] hover:bg-white/[0.12] border border-white/[0.1] hover:border-white/[0.2] hover:scale-[1.02] hover:shadow-lg hover:shadow-white/[0.03] transition-all duration-200"
    >
      <div className="w-10 h-10 rounded-xl bg-white flex items-center justify-center text-[#121317] shrink-0 group-hover:bg-white/90 group-hover:scale-110 transition-all duration-200">
        {icon || <span className="font-mono font-bold text-sm">{template.slug.charAt(0).toUpperCase()}</span>}
      </div>
      <div>
        <h3 className="text-on-surface font-medium capitalize">{template.slug}</h3>
        <p className="text-sm text-on-surface-variant leading-relaxed mt-0.5">{template.description}</p>
      </div>
    </Link>
  )
}

export default function Landing() {
  const templates = listTemplatesFallback()

  return (
    <div>
      {/* ===== HERO / WELCOME ===== */}
      <section className="relative min-h-[85vh] flex flex-col items-center justify-center text-center px-6 py-24 overflow-hidden">
        <ParticleRing />
        <div className="relative z-10">
          <h1 className="animate-entrance text-5xl sm:text-6xl lg:text-7xl font-medium text-on-surface leading-[1.05] tracking-tight max-w-4xl">
            Smart Contract Wizard
            <span className="block text-on-surface-variant text-[0.7em] mt-2 font-light">for Cardano</span>
          </h1>
          <p className="animate-entrance-d2 text-lg text-on-surface-variant mt-6 max-w-xl mx-auto leading-relaxed font-light">
            Generate security-focused Aiken starter contracts in seconds. 13 templates, composable features, and built-in static analysis workflow.
          </p>
          <div className="animate-entrance-d3 flex flex-wrap items-center justify-center gap-3 mt-8">
            <Link to="/wizard" className="btn-primary text-base px-7 py-3">
              Start Building
            </Link>
            <Link to="/docs/templates" className="btn-secondary text-base px-7 py-3">
              Explore Templates
            </Link>
          </div>
        </div>
      </section>

      {/* ===== PRODUCT ===== */}
      <section className="max-w-7xl mx-auto px-6 lg:px-12 py-16 lg:py-24">
        <RevealSection>
          <h2 className="text-3xl sm:text-4xl lg:text-5xl font-light text-on-surface leading-tight max-w-xl mb-4">
            Generate secure Aiken starters from 13 templates.
          </h2>
        </RevealSection>

        <RevealSection>
          <div className="flex flex-wrap items-center justify-center gap-3 py-20 lg:py-24">
            {FLOATING_ICON_SLUGS.map((slug, i) => (
              <Link
                key={slug}
                to={`/wizard?template=${slug}`}
                className="floating-icon flex-shrink-0 w-16 h-16 sm:w-20 sm:h-20 lg:w-24 lg:h-24 rounded-full bg-white/95 shadow-lg flex items-center justify-center text-[#121317] hover:scale-110 hover:shadow-xl transition-transform duration-300 cursor-pointer"
                style={{ animationDelay: `${(i % 5) * -1}s` }}
              >
                <span className="scale-[1.2] sm:scale-[1.5] lg:scale-[2]">
                  {TEMPLATE_ICONS[slug]}
                </span>
              </Link>
            ))}
          </div>
        </RevealSection>
      </section>

      {/* ===== FEATURES with Sticky Layout ===== */}
      <section className="max-w-7xl mx-auto px-6 lg:px-12 py-16 lg:py-24">
        <RevealSection>
          <p className="text-on-surface-variant font-light text-lg max-w-lg mb-16">
            Every contract survives a 3-layer verification pipeline: compiler check, inline tests, and 75 static analysis detectors.
          </p>
        </RevealSection>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-8 lg:gap-16">
          <div className="space-y-16 lg:space-y-24">
            {FEATURES.map((f, i) => (
              <div key={f.title} className="max-w-md">
                <div className="text-on-surface-variant mb-4">
                  {FEATURE_ICONS[i]}
                </div>
                <h3 className="text-xl font-normal text-on-surface leading-snug mb-3">{f.title}</h3>
                <p className="text-on-surface-variant font-light leading-relaxed">{f.desc}</p>
              </div>
            ))}
          </div>

          <div className="hidden lg:block">
            <div className="sticky top-28 pb-8">
              <div className="rounded-3xl bg-white p-6 text-sm font-mono text-[#121317] overflow-hidden shadow-2xl">
                <div className="flex items-center gap-2 mb-4">
                  <div className="w-3 h-3 rounded-full bg-red-400" />
                  <div className="w-3 h-3 rounded-full bg-yellow-400" />
                  <div className="w-3 h-3 rounded-full bg-green-400" />
                  <span className="ml-2 text-xs text-[#121317]/40">validators/my_token_mint.ak</span>
                </div>
                <pre className="leading-relaxed text-xs overflow-x-auto text-[#121317]/80"><code>{`use aiken/primitive/bytearray
use cardano/transaction.{Transaction, OutputReference}

validator my_token_mint(admin: ByteArray) {
  mint(
    _redeemer: Void,
    _policy_id: ByteArray,
    self: Transaction,
  ) {
    // Signature authentication
    let must_be_signed =
      list.has(self.extra_signatories, admin)

    must_be_signed
  }

  else(_) {
    fail
  }
}`}</code></pre>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* ===== TEMPLATES GRID ===== */}
      <section className="border-t border-outline-variant">
        <div className="max-w-7xl mx-auto px-6 lg:px-12 py-16 lg:py-24">
          <RevealSection>
            <div className="flex flex-col lg:flex-row lg:items-end lg:justify-between gap-4 mb-12">
              <div>
                <h2 className="text-3xl sm:text-4xl font-light text-on-surface">13 Templates</h2>
                <p className="text-on-surface-variant font-light text-lg mt-2 max-w-lg">
                  From simple minting policies to full DeFi protocols. Every template compiles, passes tests, and survives static analysis.
                </p>
              </div>
              <Link to="/wizard" className="btn-secondary self-start lg:self-auto whitespace-nowrap">
                Open Wizard
              </Link>
            </div>
          </RevealSection>

          <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 mb-12">
            {TEMPLATE_CATEGORIES.map((cat) => (
              <div key={cat.title} className="rounded-3xl bg-white p-8">
                <h3 className="text-xl font-normal text-[#121317] mb-2">{cat.title}</h3>
                <p className="text-sm text-[#45474D] font-light leading-relaxed mb-6">{cat.desc}</p>
                <div className="flex flex-wrap gap-2">
                  {cat.templates.map((slug) => (
                    <Link
                      key={slug}
                      to={`/wizard?template=${slug}`}
                      className="px-3 py-1.5 rounded-full text-xs font-medium bg-[#121317] text-white hover:bg-[#3A3D45] hover:scale-105 transition-all duration-150 capitalize"
                    >
                      {slug}
                    </Link>
                  ))}
                </div>
              </div>
            ))}
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
            {templates.map((t) => (
              <TemplateCard key={t.slug} template={t} />
            ))}
          </div>
        </div>
      </section>

      {/* ===== Download Card ===== */}
      <section className="max-w-7xl mx-auto px-6 lg:px-12 pb-24">
        <RevealSection>
          <div className="card-dark p-10 lg:p-16">
            <p className="text-2xl sm:text-3xl lg:text-4xl font-light text-on-primary/95 max-w-md leading-snug">
              Install kaido and start generating contracts
            </p>
            <div className="flex flex-col sm:flex-row gap-3 mt-8">
              <a
                href="https://github.com/Bajuzjefe/kaido"
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center gap-2 px-6 py-3 rounded-full bg-on-primary text-primary font-medium text-sm hover:bg-on-primary/80 transition-colors"
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></svg>
                install from source
              </a>
              <Link
                to="/wizard"
                className="inline-flex items-center gap-2 px-6 py-3 rounded-full border border-on-primary/20 text-on-primary/90 font-medium text-sm hover:bg-on-primary/10 transition-colors"
              >
                Use the Web Wizard
              </Link>
            </div>
          </div>
        </RevealSection>

        <p className="text-xs text-on-surface-variant/50 text-center mt-6 max-w-2xl mx-auto leading-relaxed">
          Generated contracts are starting points for dApp development, prototyping, and testing. They are not intended for direct deployment to mainnet without independent security review and thorough testing. Use at your own risk.
        </p>
      </section>
    </div>
  )
}
