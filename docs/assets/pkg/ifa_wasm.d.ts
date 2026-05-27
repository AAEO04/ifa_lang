/* tslint:disable */
/* eslint-disable */

/**
 * Structured execution result returned from `run_code`
 */
export class RunResult {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    readonly error: string | undefined;
    readonly events: string;
    readonly output: string;
    readonly success: boolean;
}

/**
 * Cast the Opele and return an Odu name using JS-native Math.random()
 */
export function cast_opele(): string;

/**
 * Format Ifá-Lang source code
 */
export function format_code(source: string): string;

/**
 * Get version information
 */
export function get_version(): string;

/**
 * Run Ifá-Lang code and return the structured RunResult.
 * This runs synchronously as the WASM execution is fast and event-loop yielding
 * belongs at a higher orchestration layer (e.g. Web Workers) rather than fake promises.
 */
export function run_code(source: string): RunResult;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_runresult_free: (a: number, b: number) => void;
    readonly cast_opele: (a: number) => void;
    readonly format_code: (a: number, b: number, c: number) => void;
    readonly get_version: (a: number) => void;
    readonly run_code: (a: number, b: number) => number;
    readonly runresult_error: (a: number, b: number) => void;
    readonly runresult_events: (a: number, b: number) => void;
    readonly runresult_output: (a: number, b: number) => void;
    readonly runresult_success: (a: number) => number;
    readonly __wasm_bindgen_func_elem_5621: (a: number, b: number, c: number, d: number) => void;
    readonly __wasm_bindgen_func_elem_1398: (a: number, b: number, c: number) => void;
    readonly __wasm_bindgen_func_elem_1398_2: (a: number, b: number, c: number) => void;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export3: (a: number) => void;
    readonly __wbindgen_export4: (a: number, b: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_export5: (a: number, b: number, c: number) => void;
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
