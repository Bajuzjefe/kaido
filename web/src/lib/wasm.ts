export interface GeneratedFile {
  path: string
  content: string
}

export interface TemplateInfo {
  slug: string
  description: string
  options?: string[]
  supports_sdk?: boolean
}

export interface FeatureInfo {
  name: string
  description: string
  purpose: string | null
  depends_on: string[]
}

export interface ValidationResult {
  valid: boolean
  errors: string[]
}

const SDK_SUPPORTED_SLUGS = new Set([
  'mint',
  'simple_mint',
  'vesting',
  'escrow',
  'treasury',
  'multisig_treasury',
  'marketplace',
  'nft_marketplace',
  'staking',
  'staking_pool',
  'oracle',
  'oracle_settlement',
  'referral',
  'referral_system',
])

function inferSdkSupport(slug: string): boolean {
  return SDK_SUPPORTED_SLUGS.has(slug.toLowerCase().replace(/-/g, '_'))
}

function normalizeTemplateInfo(template: TemplateInfo): TemplateInfo {
  return {
    ...template,
    supports_sdk: template.supports_sdk ?? inferSdkSupport(template.slug),
  }
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
let wasm: any = null
let loading: Promise<void> | null = null
let wasmFailed = false

async function loadWasm() {
  if (wasm) return
  if (wasmFailed) return
  if (loading) {
    await loading
    return
  }
  loading = (async () => {
    try {
      const mod = await import('../wasm/kaido_core')
      await mod.default()
      wasm = mod
    } catch {
      wasmFailed = true
      console.warn('WASM not available, using fallback mode. Build with wasm-pack to enable full generation.')
    }
  })()
  await loading
}

export async function listTemplates(): Promise<TemplateInfo[]> {
  await loadWasm()
  if (!wasm) {
    if (import.meta.env.DEV) return FALLBACK_TEMPLATES
    throw new Error('WASM module unavailable. Build web with `npm run build:full`.')
  }
  return (JSON.parse(wasm.list_templates()) as TemplateInfo[]).map(normalizeTemplateInfo)
}

export async function getTemplateInfo(slug: string): Promise<TemplateInfo> {
  await loadWasm()
  if (!wasm) {
    if (import.meta.env.DEV) {
      return normalizeTemplateInfo(
        FALLBACK_TEMPLATES.find((t) => t.slug === slug) ?? FALLBACK_TEMPLATES[0]
      )
    }
    throw new Error('WASM module unavailable. Build web with `npm run build:full`.')
  }
  return normalizeTemplateInfo(JSON.parse(wasm.get_template_info(slug)) as TemplateInfo)
}

export async function listFeatures(): Promise<FeatureInfo[]> {
  await loadWasm()
  if (!wasm) {
    if (import.meta.env.DEV) return FALLBACK_FEATURES
    throw new Error('WASM module unavailable. Build web with `npm run build:full`.')
  }
  return JSON.parse(wasm.list_features())
}

export async function generate(options: Record<string, unknown>): Promise<GeneratedFile[]> {
  await loadWasm()
  if (!wasm) {
    if (import.meta.env.DEV) return generateFallback(options)
    throw new Error('WASM generation unavailable. Build web with `npm run build:full`.')
  }
  const result = wasm.generate(JSON.stringify(options))
  return JSON.parse(result)
}

export async function generateSdk(options: Record<string, unknown>): Promise<GeneratedFile[]> {
  await loadWasm()
  if (!wasm) {
    if (import.meta.env.DEV) return []
    throw new Error('WASM SDK generation unavailable. Build web with `npm run build:full`.')
  }
  const result = wasm.generate_sdk(JSON.stringify(options))
  return JSON.parse(result)
}

export async function validateCustom(options: Record<string, unknown>): Promise<ValidationResult> {
  await loadWasm()
  if (!wasm) {
    if (import.meta.env.DEV) {
      return { valid: false, errors: ['WASM validator unavailable in dev fallback mode.'] }
    }
    throw new Error('WASM validation unavailable. Build web with `npm run build:full`.')
  }
  return JSON.parse(wasm.validate_custom(JSON.stringify(options)))
}

// Fallback data for dev mode (no WASM build)
const FALLBACK_TEMPLATES: TemplateInfo[] = [
  { slug: 'mint', description: 'CIP-25 minting policy with admin signature and optional time-lock', supports_sdk: true },
  { slug: 'vesting', description: 'Time-locked fund release with beneficiary claim and optional cancel', supports_sdk: true },
  { slug: 'escrow', description: 'Two-party escrow with deadline, completion, and mutual cancellation', supports_sdk: true },
  { slug: 'treasury', description: 'N-of-M multisig treasury with deposit, withdraw, datum continuity, and 2 ADA floor', supports_sdk: true },
  { slug: 'marketplace', description: 'NFT marketplace with list, buy, and delist actions', supports_sdk: true },
  { slug: 'staking', description: 'Staking pool with deposit, withdraw, and admin rewards', supports_sdk: true },
  { slug: 'oracle', description: 'Oracle-gated settlement with deadline and buyer reclaim', supports_sdk: true },
  { slug: 'referral', description: 'On-chain referral system with mint, treasury, and anti-sybil protection', supports_sdk: true },
  { slug: 'dex', description: 'DEX/AMM pool with constant-product swaps, liquidity, and fee management', supports_sdk: false },
  { slug: 'lending', description: 'Lending pool with supply, borrow, repay, and collateral ratio enforcement', supports_sdk: false },
  { slug: 'governance', description: 'DAO governance with token-gated treasury and proposal execution', supports_sdk: false },
  { slug: 'streaming', description: 'Streaming payments with time-based tranches and cancel/top-up', supports_sdk: false },
  { slug: 'custom', description: 'Custom validator with composable features (sig, timelock, datum-continuity, ...)', supports_sdk: false },
]

const FALLBACK_FEATURES: FeatureInfo[] = [
  { name: 'signature-auth', description: 'Require a specific signer in extra_signatories', purpose: null, depends_on: [] },
  { name: 'timelock', description: 'Enforce validity_range before/after a deadline field', purpose: 'spend', depends_on: [] },
  { name: 'datum-continuity', description: 'Find continuing output and validate datum preservation', purpose: 'spend', depends_on: [] },
  { name: 'value-preservation', description: 'Verify lovelace math (input >= output)', purpose: 'spend', depends_on: ['datum-continuity'] },
  { name: 'reference-safety', description: 'Reject reference script injection on continuing output', purpose: 'spend', depends_on: ['datum-continuity'] },
  { name: 'burn-verification', description: 'Check all minted quantities are negative (mint-only)', purpose: 'mint', depends_on: [] },
  { name: 'bounded-operations', description: 'Enforce minimum lovelace floor on continuing output', purpose: 'spend', depends_on: ['datum-continuity'] },
]

export function listTemplatesFallback(): TemplateInfo[] {
  return FALLBACK_TEMPLATES.map(normalizeTemplateInfo)
}

export function listFeaturesFallback(): FeatureInfo[] {
  return FALLBACK_FEATURES
}

// Fallback code generation (sample output for dev preview)
function generateFallback(options: Record<string, unknown>): GeneratedFile[] {
  const ns = String(options.namespace ?? 'myorg')
  const name = String(options.project_name ?? 'my_contract')
  const template = String(options.template ?? 'mint')

  const aikenToml = `name = "${ns}/${name}"
version = "0.0.1"
compiler = "v1.1.21"
plutus = "v3"
license = "Apache-2.0"

[repository]
user = "${ns}"
project = "${name}"
platform = "github"

[[dependencies]]
name = "aiken-lang/stdlib"
version = "v3.0.0"
source = "github"
`

  const typesAk = generateTypesFile(template, options)
  const validatorAk = generateValidatorFile(template, ns, name)

  return [
    { path: 'aiken.toml', content: aikenToml },
    { path: `lib/${ns}/${name}/types.ak`, content: typesAk },
    { path: `validators/${name}.ak`, content: validatorAk },
  ]
}

function generateTypesFile(template: string, options: Record<string, unknown>): string {
  switch (template) {
    case 'mint':
      return options.time_lock
        ? `pub type MintDatum {\n  admin: ByteArray,\n  deadline: Int,\n}\n\npub type MintRedeemer {\n  Mint\n  Burn\n}\n`
        : `pub type MintRedeemer {\n  Mint\n  Burn\n}\n`
    case 'vesting':
      return `pub type VestingDatum {\n  owner: ByteArray,\n  beneficiary: ByteArray,\n  deadline: Int,\n  amount: Int,\n}\n\npub type VestingRedeemer {\n  Claim\n${options.cancellable ? '  Cancel\n' : ''}}\n`
    case 'escrow':
      return `pub type EscrowDatum {\n  buyer: ByteArray,\n  seller: ByteArray,\n  amount: Int,\n  deadline: Int,\n}\n\npub type EscrowRedeemer {\n  Complete\n  Cancel\n  Reclaim\n}\n`
    default:
      return `pub type Datum {\n  admin: ByteArray,\n}\n\npub type Redeemer {\n  Execute\n}\n`
  }
}

function generateValidatorFile(
  template: string,
  ns: string,
  name: string,
): string {
  const modPath = `${ns}/${name}/types`
  switch (template) {
    case 'mint':
      return `use cardano/transaction.{Transaction}\nuse ${modPath}.{MintRedeemer}\n\nvalidator ${name}(admin: ByteArray) {\n  mint(redeemer: MintRedeemer, _policy_id: ByteArray, self: Transaction) {\n    when redeemer is {\n      MintRedeemer.Mint -> {\n        let must_be_signed = list.has(self.extra_signatories, admin)\n        must_be_signed\n      }\n      MintRedeemer.Burn -> True\n    }\n  }\n\n  else(_) {\n    fail\n  }\n}\n`
    case 'vesting':
      return `use cardano/transaction.{Transaction}\nuse ${modPath}.{VestingDatum, VestingRedeemer}\n\nvalidator ${name} {\n  spend(datum_opt: Option<VestingDatum>, redeemer: VestingRedeemer, _input: Data, self: Transaction) {\n    expect Some(datum) = datum_opt\n    when redeemer is {\n      VestingRedeemer.Claim -> {\n        let signed_by_beneficiary = list.has(self.extra_signatories, datum.beneficiary)\n        let after_deadline = when self.validity_range.lower_bound.bound_type is {\n          Finite(lower) -> lower >= datum.deadline\n          _ -> False\n        }\n        signed_by_beneficiary && after_deadline\n      }\n    }\n  }\n\n  else(_) {\n    fail\n  }\n}\n`
    default:
      return `use cardano/transaction.{Transaction}\nuse ${modPath}.{Datum, Redeemer}\n\nvalidator ${name} {\n  spend(datum_opt: Option<Datum>, redeemer: Redeemer, _input: Data, self: Transaction) {\n    expect Some(datum) = datum_opt\n    when redeemer is {\n      Redeemer.Execute -> {\n        list.has(self.extra_signatories, datum.admin)\n      }\n    }\n  }\n\n  else(_) {\n    fail\n  }\n}\n`
  }
}
