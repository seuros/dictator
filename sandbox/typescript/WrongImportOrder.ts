// This file has wrong import order to test enforced: false
// Expected order: System → External → Internal
// Actual order: Internal → External → System (WRONG!)

import { helper } from './utils';
import { format } from 'date-fns';
import * as path from 'path';

export function processPath(input: string): string {
  return path.join(format(new Date(), 'yyyy-MM-dd'), helper(input));
}
