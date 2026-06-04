/* tslint:disable */
/* eslint-disable */

/**
 * Compile a .llm source string to the requested target.
 *
 * target: "plain" | "json-ir" | "shadow" |
 *         "openai-chat" | "anthropic-messages"
 *
 * Returns a JSON object:
 * { ok: true, output: "..." }
 * or
 * { ok: false, errors: ["..."] }
 */
export function compile(source: string, target: string): string;

/**
 * Set up better panic messages in the browser console.
 */
export function init(): void;

/**
 * Lint a .llm source string.
 *
 * Returns:
 * { warnings: [{ code: "L001", message: "..." }, ...] }
 */
export function lint(source: string): string;

/**
 * Validate a .llm source string.
 *
 * Returns:
 * { ok: true }
 * or
 * { ok: false, errors: ["[E101] missing required key: `agent`", ...] }
 */
export function validate(source: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly compile: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly lint: (a: number, b: number, c: number) => void;
    readonly validate: (a: number, b: number, c: number) => void;
    readonly init: () => void;
    readonly __wbindgen_export: (a: number, b: number, c: number) => void;
    readonly __wbindgen_export2: (a: number, b: number) => number;
    readonly __wbindgen_export3: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
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
