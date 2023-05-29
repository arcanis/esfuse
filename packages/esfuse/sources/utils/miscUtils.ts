export type DeepPartial<T> = {
  [P in keyof T]?: T[P] extends Array<infer U>
    ? Array<DeepPartial<U>>
    : T[P] extends ReadonlyArray<infer U>
      ? ReadonlyArray<DeepPartial<U>>
      : DeepPartial<T[P]>
};

// This function transforms an iterable into an array and sorts it according to
// the mapper functions provided as parameter. The mappers are expected to take
// each element from the iterable and generate a string from it, that will then
// be used to compare the entries.
//
// Using sortMap is more efficient than kinda reimplementing the logic in a sort
// predicate because sortMap caches the result of the mappers in such a way that
// they are guaranteed to be executed exactly once for each element.

export type SortMapper<T> =
  | ((value: T) => string)
  | ((value: T) => number);

export function sortMap<T>(values: Iterable<T>, mappers: SortMapper<T> | Array<SortMapper<T>>) {
  const asArray = Array.from(values);

  if (!Array.isArray(mappers))
    mappers = [mappers];

  const stringified: Array<Array<string | number>> = [];

  for (const mapper of mappers)
    stringified.push(asArray.map(value => mapper(value)));

  const indices = asArray.map((_, index) => index);

  indices.sort((a, b) => {
    for (const layer of stringified) {
      const comparison = layer[a] < layer[b] ? -1 : layer[a] > layer[b] ? +1 : 0;

      if (comparison !== 0) {
        return comparison;
      }
    }

    return 0;
  });

  return indices.map(index => {
    return asArray[index];
  });
}

export function getFactoryWithDefault<K, T>(map: Map<K, T>, key: K, factory: () => T) {
  let value = map.get(key);

  if (typeof value === `undefined`)
    map.set(key, value = factory());

  return value;
}

export function getArrayWithDefault<K, T>(map: Map<K, Array<T>>, key: K) {
  let value = map.get(key);

  if (typeof value === `undefined`)
    map.set(key, value = []);

  return value;
}

export function getSetWithDefault<K, T>(map: Map<K, Set<T>>, key: K) {
  let value = map.get(key);

  if (typeof value === `undefined`)
    map.set(key, value = new Set<T>());

  return value;
}

export function getMapWithDefault<K, MK, MV>(map: Map<K, Map<MK, MV>>, key: K) {
  let value = map.get(key);

  if (typeof value === `undefined`)
    map.set(key, value = new Map<MK, MV>());

  return value;
}

export type Deferred<T = void> = {
  promise: Promise<T>;
  resolve: (val: T) => void;
  reject: (err: Error) => void;
};

export function makeDeferred<T = void>(): Deferred<T> {
  let resolve: (val: T) => void;
  let reject: (err: Error) => void;

  const promise = new Promise<T>((resolveFn, rejectFn) => {
    resolve = resolveFn;
    reject = rejectFn;
  });

  return {promise, resolve: resolve!, reject: reject!};
}

export function getUniqueValue<T>(values: Array<T>): T {
  if (values.length !== 1)
    throw new Error(`Assertion failed: Expected the provided array to have exactly one value`);

  return values[0];
}

export function getUniqueMember<T>(dict: Record<string, T>): T {
  const values = Object.values(dict);
  if (values.length !== 1)
    throw new Error(`Assertion failed: Expected the provided object to have exactly one property`);

  return values[0];
}

export function filterOutNull<T>(value: T | null): value is T {
  return value !== null;
}

export function route<T>(val: string, variants: Record<string, () => T>): T {
  if (!Object.prototype.hasOwnProperty.call(variants, val))
    throw new Error(`Invalid value "${val}"; expected one of ${Object.keys(variants).join(`, `)}`);

  return variants[val]();
}

export async function replaceStack<T>(val: Promise<T>): Promise<T> {
  try {
    return await val;
  } catch (err: any) {
    throw Object.assign(new Error(err.message), err);
  }
}

export function withErrorLogging<T>(fn: (...args: Array<any>) => Promise<T>) {
  return async (...args: Array<any>) => {
    try {
      return await fn(...args);
    } catch (err) {
      console.log(err);
      throw err;
    }
  };
}

export function cooked(strings: Array<string>, ...subs: Array<string>) {
  return String.raw({raw: strings}, ...subs);
}

export function rethrowAllSettled(results: Array<PromiseSettledResult<any>>) {
  const errors = results.filter((result): result is PromiseRejectedResult => {
    return result.status === `rejected`;
  });

  if (errors.length > 0) {
    throw errors[0].reason;
  }
}
