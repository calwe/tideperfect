import { Result } from "@/bindings";

/**
 * Unwraps a Result, throwing an error if status is "error"
 * Use this when you want to handle errors via try/catch or error boundaries
 */
export function unwrap<T, E>(result: Result<T, E>): T {
  if (result.status === "ok") {
    return result.data;
  }
  throw result.error;
}

/**
 * Checks if a Result is ok (successful)
 */
export function isOk<T, E>(result: Result<T, E>): result is { status: "ok"; data: T } {
  return result.status === "ok";
}

/**
 * Checks if a Result is an error
 */
export function isError<T, E>(result: Result<T, E>): result is { status: "error"; error: E } {
  return result.status === "error";
}

/**
 * Maps the data value of an ok Result, leaves error Results unchanged
 */
export function mapOk<T, E, U>(result: Result<T, E>, fn: (data: T) => U): Result<U, E> {
  if (result.status === "ok") {
    return { status: "ok", data: fn(result.data) };
  }
  return result;
}

/**
 * Returns the data if ok, otherwise returns the fallback value
 */
export function unwrapOr<T, E>(result: Result<T, E>, fallback: T): T {
  if (result.status === "ok") {
    return result.data;
  }
  return fallback;
}
