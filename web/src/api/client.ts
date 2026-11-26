// The one way the SPA talks to the JSON API. Area-specific calls live next to
// their pages (src/pages/<area>/api.ts) and are built on `api`.

import { notify } from "../components/Toasts";
import type { TFunction } from "i18next";

/** An error response: `{"error": {"code": "..."}}`. `code` is stable API. */
export class ApiError extends Error {
  constructor(
    readonly status: number,
    readonly code: string,
  ) {
    super(`API error ${status}: ${code}`);
  }
}

type Options = {
  method?: "GET" | "POST" | "PUT" | "PATCH" | "DELETE";
  /** Sent as JSON. */
  body?: unknown;
  signal?: AbortSignal;
  success?: string;
  /** No automatic success notice after a write; the caller shows its own. Errors still notify. */
  quiet?: boolean;
};

/** Calls `/api<path>` and returns the JSON body. Throws `ApiError`. */
export async function api<T>(path: string, options: Options = {}): Promise<T> {
  const mutation = options.method !== undefined && options.method !== "GET";
  const fail = (error: ApiError) => {
    if (mutation) notify({ error: error.code });
    throw error;
  };
  let response: Response;
  try {
    response = await fetch(`/api${path}`, {
      method: options.method ?? "GET",
      credentials: "same-origin",
      headers: options.body === undefined ? undefined : { "Content-Type": "application/json" },
      body: options.body === undefined ? undefined : JSON.stringify(options.body),
      signal: options.signal,
    });
  } catch (cause) {
    if (cause instanceof DOMException && cause.name === "AbortError") throw cause;
    return fail(new ApiError(0, "network"));
  }

  if (!response.ok) {
    const payload = await response.json().catch(() => null);
    return fail(new ApiError(response.status, payload?.error?.code ?? "internal"));
  }
  const data = response.status === 204 ? undefined : await response.json();
  if (mutation && !options.quiet) notify({ success: options.success ?? "saved" });
  return data as T;
}

/**
 * The localised text for an error. Looks up `errors.<code>` in the given
 * `t`'s namespace, then in `common`, then falls back to `common:errors.unknown`.
 */
export function errorMessage(t: TFunction, error: unknown): string {
  const code = error instanceof ApiError ? error.code : "unknown";
  return t([`errors.${code}`, `common:errors.${code}`, "common:errors.unknown"]);
}
