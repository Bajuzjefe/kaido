import { Routes, Route, Link, useLocation } from 'react-router-dom'

const SIDEBAR_LINKS = [
  { to: '/docs', label: 'Getting Started' },
  { to: '/docs/cli', label: 'CLI Reference' },
  { to: '/docs/features', label: 'Composable Features' },
  { to: '/docs/templates', label: 'Templates' },
  { to: '/docs/sdk', label: 'TypeScript SDK' },
]

function Sidebar() {
  const { pathname } = useLocation()
  return (
    <nav className="w-56 flex-shrink-0 border-r border-outline-variant py-6 px-4 space-y-1">
      {SIDEBAR_LINKS.map(({ to, label }) => {
        const active = pathname === to || (to !== '/docs' && pathname.startsWith(to))
        return (
          <Link
            key={to}
            to={to}
            className={`block px-3 py-2 rounded-2xl text-sm transition-colors ${
              active
                ? 'text-on-surface bg-surface-container-high font-medium'
                : 'text-on-surface-variant hover:text-on-surface hover:bg-surface-container'
            }`}
          >
            {label}
          </Link>
        )
      })}
    </nav>
  )
}

function DocPage({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="max-w-3xl py-8 px-8">
      <h1 className="text-3xl font-medium text-on-surface mb-6">{title}</h1>
      <div className="max-w-none text-on-surface-variant leading-relaxed space-y-4">
        {children}
      </div>
    </div>
  )
}

function GettingStarted() {
  return (
    <DocPage title="Getting Started">
      <p>
        kaido generates security-focused Aiken starter templates.
        Generated contracts are intended for development and testing, with independent security review required before production deployment.
      </p>

      <h2 className="text-xl font-medium text-on-surface mt-8 mb-3">Prerequisites</h2>
      <ul className="list-disc list-inside space-y-1 text-on-surface-variant">
        <li>Aiken v1.1.21+</li>
        <li>Rust toolchain (for CLI installation)</li>
      </ul>

      <h2 className="text-xl font-medium text-on-surface mt-8 mb-3">Quick Start</h2>
      <p>Install the CLI and generate your first contract:</p>
      <pre className="bg-code-surface rounded-2xl p-4 text-sm font-mono text-white/80 overflow-x-auto">
{`git clone https://github.com/Bajuzjefe/kaido.git
cd kaido
cargo install --path crates/kaido-cli

# Generate a minting policy
kaido generate --template mint --namespace myorg --project-name my_token

# Generate with time lock
kaido generate --template mint --namespace myorg --project-name my_token --time-lock

# List all templates
kaido list`}
      </pre>

      <h2 className="text-xl font-medium text-on-surface mt-8 mb-3">Or Use the Web Wizard</h2>
      <p>
        No installation needed. Use the <Link to="/wizard" className="text-on-surface font-medium underline underline-offset-2">web wizard</Link> to
        configure and download contracts directly in your browser. Everything runs locally via WebAssembly.
      </p>
    </DocPage>
  )
}

function CLIReference() {
  return (
    <DocPage title="CLI Reference">
      <h2 className="text-xl font-medium text-on-surface mt-4 mb-3">Commands</h2>

      <h3 className="text-lg font-medium text-on-surface mt-6 mb-2">kaido generate</h3>
      <p className="text-on-surface-variant">Generate a new Aiken smart contract project.</p>
      <pre className="bg-code-surface rounded-2xl p-4 text-sm font-mono text-white/70 overflow-x-auto">
{`kaido generate --template <TEMPLATE> --namespace <NS> --project-name <NAME> [OPTIONS]

Options:
  --template <T>       Template to use (mint, vesting, escrow, treasury, ...)
  --namespace <NS>     Aiken namespace (e.g., myorg)
  --project-name <N>   Project name (snake_case)
  --output <DIR>       Output directory (default: ./<project_name>)
  --sdk                Generate TypeScript SDK alongside Aiken code
  --skip-verify        Skip aiken build + aiken check + aikido scan

Template-specific:
  --time-lock          [mint] Add time-lock to minting policy
  --cancellable        [vesting] Allow owner to cancel
  --partial-claim      [vesting] Allow partial withdrawals
  --purpose <P>        [custom] Validator purpose: spend or mint
  --features <F>       [custom] Comma-separated features
  --datum <D>          [custom] Datum fields (e.g. admin:ByteArray,amount:Int)
  --redeemer <R>       [custom] Redeemer actions (e.g. Claim,Cancel(reason:ByteArray))`}
      </pre>

      <h3 className="text-lg font-medium text-on-surface mt-6 mb-2">kaido list</h3>
      <p className="text-on-surface-variant">List all available templates.</p>

      <h3 className="text-lg font-medium text-on-surface mt-6 mb-2">kaido verify</h3>
      <p className="text-on-surface-variant">Run verification on an existing Aiken project (aiken build + check + aikido scan).</p>
      <pre className="bg-code-surface rounded-2xl p-4 text-sm font-mono text-white/70 overflow-x-auto">
{`kaido verify [PATH]

Options:
  PATH    Path to Aiken project (default: current directory)`}
      </pre>
    </DocPage>
  )
}

function FeaturesGuide() {
  return (
    <DocPage title="Composable Features">
      <p>
        The custom template builder lets you compose validators from individual security features.
        Each feature adds specific checks to your validator logic.
      </p>

      <div className="space-y-6 mt-6">
        {[
          { name: 'signature-auth', desc: 'Require a specific signer in extra_signatories. The most basic authentication check.', deps: [] },
          { name: 'timelock', desc: 'Enforce validity_range before/after a deadline field in the datum. Commonly used for vesting and escrow deadlines.', deps: [] },
          { name: 'datum-continuity', desc: 'Find the continuing output at the same script address and validate datum preservation across transactions.', deps: [] },
          { name: 'value-preservation', desc: 'Verify lovelace conservation (input >= output). Prevents value extraction attacks.', deps: ['datum-continuity'] },
          { name: 'reference-safety', desc: 'Reject reference script injection on continuing output. Prevents script replacement attacks.', deps: ['datum-continuity'] },
          { name: 'burn-verification', desc: 'Check that all minted quantities are negative (burn-only). For mint purpose validators.', deps: [] },
          { name: 'bounded-operations', desc: 'Enforce a minimum lovelace floor on the continuing output. Prevents dust attacks.', deps: ['datum-continuity'] },
        ].map((f) => (
          <div key={f.name} className="border-l-2 border-on-surface/20 pl-4">
            <h3 className="text-on-surface font-medium font-mono">{f.name}</h3>
            <p className="text-sm text-on-surface-variant mt-1">{f.desc}</p>
            {f.deps.length > 0 && (
              <p className="text-xs text-on-surface-variant/60 mt-1">
                Depends on: <span className="font-mono">{f.deps.join(', ')}</span>
              </p>
            )}
          </div>
        ))}
      </div>
    </DocPage>
  )
}

function TemplatesList() {
  return (
    <DocPage title="Templates">
      <p>kaido ships with 13 security-focused templates covering common Cardano DeFi patterns.</p>

      <div className="space-y-4 mt-6">
        {[
          { slug: 'mint', desc: 'CIP-25 minting policy with admin signature and optional time-lock' },
          { slug: 'vesting', desc: 'Time-locked fund release with beneficiary claim and optional cancel' },
          { slug: 'escrow', desc: 'Two-party escrow with deadline, completion, and mutual cancellation' },
          { slug: 'treasury', desc: 'N-of-M multisig treasury with deposit, withdraw, datum continuity, and 2 ADA floor' },
          { slug: 'marketplace', desc: 'NFT marketplace with list, buy, and delist actions' },
          { slug: 'staking', desc: 'Staking pool with deposit, withdraw, and admin rewards' },
          { slug: 'oracle', desc: 'Oracle-gated settlement with deadline and buyer reclaim' },
          { slug: 'referral', desc: 'On-chain referral system with mint, treasury, and anti-sybil protection' },
          { slug: 'dex', desc: 'DEX/AMM pool with constant-product swaps, liquidity, and fee management' },
          { slug: 'lending', desc: 'Lending pool with supply, borrow, repay, and collateral ratio enforcement' },
          { slug: 'governance', desc: 'DAO governance with token-gated treasury and proposal execution' },
          { slug: 'streaming', desc: 'Streaming payments with time-based tranches and cancel/top-up' },
          { slug: 'custom', desc: 'Custom validator with composable features (sig, timelock, datum-continuity, ...)' },
        ].map((t) => (
          <div key={t.slug} className="flex items-start gap-3 p-3 rounded-2xl bg-surface-container">
            <span className="font-mono text-on-surface font-bold w-24 flex-shrink-0 capitalize">{t.slug}</span>
            <span className="text-sm text-on-surface-variant">{t.desc}</span>
          </div>
        ))}
      </div>

      <p className="mt-6 text-sm text-on-surface-variant/60">
        Try any template in the <Link to="/wizard" className="text-on-surface font-medium underline underline-offset-2">wizard</Link>.
      </p>
    </DocPage>
  )
}

function SDKGuide() {
  return (
    <DocPage title="TypeScript SDK">
      <p>
        kaido can generate a TypeScript SDK alongside your Aiken contract. The SDK provides typed
        transaction builders that work with the Anvil API for Cardano transaction construction.
      </p>

      <h2 className="text-xl font-medium text-on-surface mt-8 mb-3">Generation</h2>
      <p>
        Enable SDK generation in the wizard by checking "Generate TypeScript SDK" on supported
        templates, or via CLI:
      </p>
      <p>
        Currently supported: <span className="font-mono">mint, vesting, escrow, treasury, marketplace, staking, oracle, referral</span>.
        Not yet supported: <span className="font-mono">dex, lending, governance, streaming, custom</span>.
      </p>
      <pre className="bg-code-surface rounded-2xl p-4 text-sm font-mono text-white/80 overflow-x-auto">
{`kaido generate --template mint --namespace myorg --project-name my_token --sdk`}
      </pre>

      <h2 className="text-xl font-medium text-on-surface mt-8 mb-3">Usage</h2>
      <pre className="bg-code-surface rounded-2xl p-4 text-sm font-mono text-white/70 overflow-x-auto">
{`import { MintClient } from './sdk/client'

const client = new MintClient({
  address: 'addr1...',
  // Anvil API integration
})

// Build a mint transaction
const tx = await client.buildMint({
  assetName: 'MyToken',
  quantity: 1,
  metadata: { name: 'My Token', image: 'ipfs://...' },
})

// Sign with CIP-30 wallet
const signed = await wallet.signTx(tx.complete, true)
await client.submit(tx.complete, [signed])`}
      </pre>
    </DocPage>
  )
}

export default function Docs() {
  return (
    <div className="flex min-h-[calc(100vh-3.5rem)]">
      <Sidebar />
      <div className="flex-1 overflow-auto">
        <Routes>
          <Route index element={<GettingStarted />} />
          <Route path="cli" element={<CLIReference />} />
          <Route path="features" element={<FeaturesGuide />} />
          <Route path="templates" element={<TemplatesList />} />
          <Route path="sdk" element={<SDKGuide />} />
        </Routes>
      </div>
    </div>
  )
}
