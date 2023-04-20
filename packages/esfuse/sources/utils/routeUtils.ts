const IDENTIFIER_REGEXP = /^[0-9a-zA-Z]+$/;

export type StaticSegment =
  | {type: `static`, value: string};

export type ParametrizedSegment =
  | {type: `parameter` | `wildcard` | `optional-wildcard`, name: string};

export type RoutePatternSegment =
  | StaticSegment
  | ParametrizedSegment;

export function parseFilePattern(path: string) {
  const segments: Array<RoutePatternSegment> = [];

  const fsPattern = path.replace(/((^|[\\/])[a-z]+)?\.[^/]+$/, ``);

  let i = 0;

  const isIdentifierCharacter = () => !!IDENTIFIER_REGEXP.test(fsPattern[i]);

  const checkString = (str: string) => {
    if (!fsPattern.startsWith(str, i))
      throw new Error(`Parse error: Expected ${str}`);

    return false;
  };

  const checkIsIdentifierCharacter = () => {
    if (!isIdentifierCharacter())
      throw new Error(`Parse error: Invalid identifier`);

    return true;
  };

  const checkEol = () => {
    if (i >= fsPattern.length)
      throw new Error(`Parse error: Unexpected end of input`);

    return true;
  };

  while (i < fsPattern.length) {
    if (fsPattern[i] === `/` || fsPattern[i] === `\\`) {
      i += 1;
    } else if (fsPattern[i] === `(`) {
      i += 1;
      while (checkEol() && fsPattern[i] !== `)`) {
        checkIsIdentifierCharacter();
        i += 1;
      }
      i += 1;
    } else if (fsPattern[i] === `[`) {
      i += 1;

      let optional = false;
      if (fsPattern[i] === `[`) {
        optional = true;
        i += 1;
        checkString(`...`);
      }

      let parameterType: ParametrizedSegment[`type`] = `parameter`;

      if (fsPattern[i] === `.`) {
        checkString(`...`);
        parameterType = optional
          ? `optional-wildcard`
          : `wildcard`;
        i += 3;
      }

      let name = ``;
      while (checkEol() && fsPattern[i] !== `]`) {
        checkIsIdentifierCharacter();
        name += fsPattern[i];
        i += 1;
      }

      segments.push({
        type: parameterType,
        name,
      });

      if (optional) {
        checkString(`]]`);
        i += 2;
      } else {
        checkString(`]`);
        i += 1;
      }
    } else {
      checkIsIdentifierCharacter();

      let value = ``;
      while (isIdentifierCharacter()) {
        value += fsPattern[i];
        i += 1;
      }

      segments.push({
        type: `static`,
        value,
      });
    }
  }

  return segments;
}

export function serializeToRadix(segments: Array<RoutePatternSegment>) {
  const required: Array<string> = [];

  const pattern = `/${segments.map(segment => {
    switch (segment.type) {
      case `static`:
        return segment.value;
      case `parameter`:
        return `:${segment.name}`;
      case `wildcard`:
        return `**:${segment.name}`;
      case `optional-wildcard`:
        required.push(segment.name);
        return `**:${segment.name}`;
      default:
        throw new Error(`Assertion failed: Unknown segment type`);
    }
  }).join(`/`)}`;

  return {pattern, required};
}

export function serializeToReact(segments: Array<RoutePatternSegment>) {
  const required: Array<string> = [];
  let splat: string = ``;

  const pattern = `/${segments.map(segment => {
    switch (segment.type) {
      case `static`:
        return segment.value;
      case `parameter`:
        return `:${segment.name}`;
      case `wildcard`:
        splat = segment.name;
        return `*`;
      case `optional-wildcard`:
        splat = segment.name;
        required.push(segment.name);
        return `*`;
      default:
        throw new Error(`Assertion failed: Unknown segment type`);
    }
  }).join(`/`)}`;

  return {pattern, required, splat};
}

/**
console.log(convertPattern("foo/bar")); // "/foo/bar"
console.log(convertPattern("abc/[val]")); // "/abc/:val"
console.log(convertPattern("qux/[val]/bar")); // "/qux/:val/bar"
console.log(convertPattern("foo/hello/[...val]")); // "/foo/hello/**:val"
console.log(convertPattern("foo/[[...val]]")); // "/foo/**:val"
console.log(convertPattern("[val1]/[[...val2]]")); // "/:val1/**:val2"
console.log(convertPattern("foo/[val1]/[[...val2]]")); // "/foo/:val1/**:val2"
console.log(convertPattern("(foo)/bar")); // "/bar"
console.log(convertPattern("(foo)/bar/[word]")); // "/bar/:word"
*/
