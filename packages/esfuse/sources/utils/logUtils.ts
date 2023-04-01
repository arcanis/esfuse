import {MessageName, MessagePayload} from '../MessageName';

export type MessageLogFn = <T extends MessageName>(
  name: T,
  payload: MessagePayload[T],
) => void;

export type MessageLogger = {
  [T in MessageName]?: (payload: MessagePayload[T]) => string;
};

export function stringifyLogEvent<T extends MessageName>(logger: MessageLogger, name: T, payload: MessagePayload[T]) {
  return Object.prototype.hasOwnProperty.call(logger, name)
    ? logger[name]!(payload)
    : null;
}

export function createLogger(fn: (data: string) => void, logger: MessageLogger): MessageLogFn {
  return (name, payload) => {
    const data = stringifyLogEvent(logger, name, payload);

    if (data !== null) {
      fn(data);
    }
  };
}
