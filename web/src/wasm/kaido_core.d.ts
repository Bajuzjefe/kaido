/* tslint:disable */
/* eslint-disable */

/**
 * Generate an Aiken project from options JSON. Returns [{path, content}].
 */
export function generate(options_json: string): string;

/**
 * Generate TypeScript SDK files. Returns [{path, content}].
 */
export function generate_sdk(options_json: string): string;

/**
 * Get detailed info for a specific template
 */
export function get_template_info(slug: string): string;

/**
 * List all composable features as JSON
 */
export function list_features(): string;

/**
 * List all available templates as JSON
 */
export function list_templates(): string;

export function slugify(s: string): string;

/**
 * Validate custom builder options (live validation for the wizard)
 */
export function validate_custom(json: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly generate: (a: number, b: number) => [number, number, number, number];
    readonly generate_sdk: (a: number, b: number) => [number, number, number, number];
    readonly get_template_info: (a: number, b: number) => [number, number];
    readonly list_features: () => [number, number];
    readonly list_templates: () => [number, number];
    readonly validate_custom: (a: number, b: number) => [number, number];
    readonly slugify: (a: number, b: number) => [number, number];
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
