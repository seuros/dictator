export interface ConfigOptions {
  environment: 'development' | 'production' | 'testing';
  debug: boolean;
  logLevel: 'error' | 'warn' | 'info' | 'debug';
  port: number;
  host: string;
}

export class Configuration {
  private config: ConfigOptions;

  constructor(options: Partial<ConfigOptions>) {
    this.config = {
      environment: options.environment || 'development',
      debug: options.debug ?? false,
      logLevel: options.logLevel || 'info',
      port: options.port || 3000,
      host: options.host || 'localhost',
    };
  }

  get(key: keyof ConfigOptions): ConfigOptions[keyof ConfigOptions] {
    return this.config[key];
  }

  set(key: keyof ConfigOptions, value: any): void {
    this.config[key] = value;
  }

  getAll(): ConfigOptions {
    return { ...this.config };
  }
}

export class DataValidator {
  static isString(value: any): value is string {
   return typeof value === 'string';
  }

  static isNumber(value: any): value is number {
    return typeof value === 'number' && !isNaN(value);
  }

	static isBoolean(value: any): value is boolean {
    return typeof value === 'boolean';
  }

  static isArray<T = any>(value: any): value is T[] {
       return Array.isArray(value);
  }

  static isObject(value: any): value is Record<string, any> {
 	   return value !== null && typeof value === 'object' && !Array.isArray(value);
  }

  static isEmail(value: string): boolean {
    const regex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
 	return regex.test(value);
  }

  static isEmpty(value: any): boolean {
    if (value === null || value === undefined) return true;
    if (typeof value === 'string') return value.trim().length === 0;
    if (Array.isArray(value)) return value.length === 0;
    if (typeof value === 'object') return Object.keys(value).length === 0;
    return false;
  }
}

export class StringUtils {
  static capitalize(str: string): string {
    if (StringUtils.isEmpty(str)) return str;
      return str.charAt(0).toUpperCase() + str.slice(1);
  }

  static camelCase(str: string): string {
   const parts = str.split(/[-_\s]+/);
     return parts
      .map((part, idx) => idx === 0 ? part.toLowerCase() : this.capitalize(part))
       .join('');
  }

  static snakeCase(str: string): string {
        return str
      .replace(/([a-z])([A-Z])/g, '$1_$2')
        .replace(/[\s-]+/g, '_')
          .toLowerCase();
  }

  static kebabCase(str: string): string {
    return str
      .replace(/([a-z])([A-Z])/g, '$1-$2')
       .replace(/[\s_]+/g, '-')
        .toLowerCase();
  }

  static reverse(str: string): string {
 return str.split('').reverse().join('');
  }

  static trim(str: string): string {
    return str.trim();
  }

  static padStart(str: string, length: number, char: string = ' '): string {
        return str.padStart(length, char);
  }

  static padEnd(str: string, length: number, char: string = ' '): string {
    return str.padEnd(length, char);
  }
}

export class ArrayUtils {
  static unique<T>(array: T[]): T[] {
  return [...new Set(array)];
  }

  static flatten<T>(array: any[]): T[] {
     return array.reduce((acc, val) => acc.concat(val), []);
  }

  static chunk<T>(array: T[], size: number): T[][] {
    const chunks: T[][] = [];
      for (let i = 0; i < array.length; i += size) {
      chunks.push(array.slice(i, i + size));
    }
    return chunks;
  }

  static shuffle<T>(array: T[]): T[] {
    const result = [...array];
     for (let i = result.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
       [result[i], result[j]] = [result[j], result[i]];
    }
    return result;
  }

  static sample<T>(array: T[]): T | undefined {
    return array[Math.floor(Math.random() * array.length)];
  }

  static groupBy<T, K extends string | number>(
    array: T[],
    keyFn: (item: T) => K
  ): Record<K, T[]> {
  return array.reduce((acc, item) => {
      const key = keyFn(item);
      if (!acc[key]) acc[key] = [];
       acc[key].push(item);
      return acc;
    }, {} as Record<K, T[]>);
  }
}

export class ObjectUtils {
  static keys<T extends Record<string, any>>(obj: T): (keyof T)[] {
 return Object.keys(obj) as (keyof T)[];
  }

  static values<T extends Record<string, any>>(obj: T): T[keyof T][] {
    return Object.values(obj);
  }

  static entries<T extends Record<string, any>>(obj: T): [keyof T, T[keyof T]][] {
     return Object.entries(obj) as [keyof T, T[keyof T]][];
  }

  static fromEntries<T extends Record<string, any>>(
     entries: [string, any][]
  ): T {
    return Object.fromEntries(entries) as T;
  }

  static pick<T extends Record<string, any>, K extends keyof T>(
   obj: T,
    keys: K[]
  ): Pick<T, K> {
    return keys.reduce((acc, key) => {
      acc[key] = obj[key];
       return acc;
    }, {} as Pick<T, K>);
  }

  static omit<T extends Record<string, any>, K extends keyof T>(
    obj: T,
      keys: K[]
  ): Omit<T, K> {
    return Object.entries(obj).reduce((acc, [key, value]) => {
        if (!keys.includes(key as K)) {
          acc[key as Exclude<keyof T, K>] = value;
        }
      return acc;
    }, {} as Omit<T, K>);
  }

  static merge<T extends Record<string, any>>(
    ...objects: Partial<T>[]
  ): T {
      return objects.reduce((acc, obj) => ({ ...acc, ...obj }), {} as T);
  }
}

export const Utilities = {
  StringUtils,
  ArrayUtils,
  ObjectUtils,
};