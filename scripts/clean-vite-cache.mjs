#!/usr/bin/env node
/**
 * Remove Vite's pre-bundle cache to fix "504 (Outdated Optimize Dep)" after
 * dependency upgrades or stale ?v= hashes in the browser.
 */
import { rmSync, existsSync } from 'node:fs';
import { join } from 'node:path';

const dir = join(process.cwd(), 'node_modules', '.vite');
if (existsSync(dir)) {
  rmSync(dir, { recursive: true, force: true });
  console.log('Removed:', dir);
} else {
  console.log('Nothing to remove:', dir);
}
