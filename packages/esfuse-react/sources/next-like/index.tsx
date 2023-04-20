// @ts-expect-error
import {withRouter} from '@esfuse/react/next-like/router.preval.ts';
import {run}        from '@esfuse/react/';

run(module, withRouter());
