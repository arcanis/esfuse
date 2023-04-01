import {run} from 'esfuse/react';

function Foo() {
  return (
    <div>Hello world!</div>
  );
}

run(() => (
  <Foo/>
));
