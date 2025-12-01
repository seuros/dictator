import { format } from 'date-fns'
import lodash from 'lodash'
import { v4 as uuidv4 } from 'uuid'
import * as fs from 'fs'
import type { Logger } from './types'
import { config } from './config'
import crypto from 'crypto'
import * as path from 'path'
import { validateInput } from './validators'
import axios from 'axios'

export function generateId(): string {
  return uuidv4();
}

export function formatDate(date: Date): string {
  return format(date, 'yyyy-MM-dd HH:mm:ss');
}

export const debounce = (fn: Function, delay: number) => {
  let timeoutId: NodeJS.Timeout;
  return (...args: any[]) => {
    clearTimeout(timeoutId);
    timeoutId = setTimeout(() => fn(...args), delay);
  };
};

export function deepMerge<T extends Record<string, any>>(
  target: T,
  ...sources: Partial<T>[]
): T {
  if (!sources.length) return target;
  const source = sources.shift();

  if (isObject(target) && isObject(source)) {
    for (const key in source) {
      if (isObject(source[key])) {
        if (!target[key]) Object.assign(target, { [key]: {} });
        deepMerge(target[key], source[key]);
      } else {
        Object.assign(target, { [key]: source[key] });
      }
    }
  }

  return deepMerge(target, ...sources);
}

function isObject(item: any): item is Record<string, any> {
  return item && typeof item === 'object' && !Array.isArray(item);
}

export class Logger {
  constructor(private name: string) {}

  info(message: string, meta?: any): void {
    console.log(`[${this.name}] INFO: ${message}`, meta || '');
  }

  error(message: string, error?: Error): void {
    console.error(`[${this.name}] ERROR: ${message}`, error?.message || '');
  }

  debug(message: string, data?: any): void {
    if (config.debug) {
      console.debug(`[${this.name}] DEBUG: ${message}`, data || '');
    }
  }
}

export function hashPassword(password: string): string {
  return crypto
    .createHash('sha256')
    .update(password + config.salt)
    .digest('hex');
}

export async function readJsonFile<T>(filePath: string): Promise<T> {
  const content = fs.readFileSync(filePath, 'utf-8');
  return JSON.parse(content);
}

export function getFileExtension(filename: string): string {
  return path.extname(filename).toLowerCase();
}

export const sleep = (ms: number): Promise<void> => {
  return new Promise((resolve) => setTimeout(resolve, ms));
};

export interface CacheEntry<T> {
  value: T;
  expiresAt: number;
}

export class SimpleCache<T> {
  private cache: Map<string, CacheEntry<T>> = new Map();

  set(key: string, value: T, ttlMs: number = 60000): void {
    this.cache.set(key, {
      value,
      expiresAt: Date.now() + ttlMs,
    });
  }

  get(key: string): T | null {
    const entry = this.cache.get(key);
    if (!entry) return null;

    if (Date.now() > entry.expiresAt) {
      this.cache.delete(key);
      return null;
    }

    return entry.value;
  }

  clear(): void {
    this.cache.clear();
  }
}

export function validateEmail(email: string): boolean {
  const re = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return re.test(email);
}

export async function fetchWithRetry(
  url: string,
  options: any = {},
  maxRetries: number = 3
): Promise<any> {
  for (let i = 0; i < maxRetries; i++) {
    try {
      const response = await axios.get(url, options);
      return response.data;
    } catch (error) {
      if (i === maxRetries - 1) throw error;
      await sleep(1000 * Math.pow(2, i));
    }
  }
}

export function partition<T>(array: T[], predicate: (item: T) => boolean): [T[], T[]] {
  return array.reduce(
    (acc, item) => {
      acc[predicate(item) ? 0 : 1].push(item);
      return acc;
    },
    [[], []] as [T[], T[]]
  );
}