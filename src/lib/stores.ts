import { writable } from 'svelte/store';
import type { AudioFile, RenameRecord, AppPhase } from './types';

export const phase = writable<AppPhase>('idle');
export const sourceDir = writable<string>('');
export const scannedFiles = writable<AudioFile[]>([]);
export const historyRecords = writable<RenameRecord[]>([]);
export const currentSessionId = writable<string>('');
export const progressValue = writable<number>(0);
export const progressLabel = writable<string>('');
export const errors = writable<string[]>([]);