export interface AudioFile {
  path: string;
  title: string;
  artist: string;
  album: string;
  bitrate: number;
  extension: string;
  file_stem: string;
  /** "unique" | "primary" | "duplicate" */
  status: string;
}

export interface RenameRecord {
  id?: number;
  original_path: string;
  new_path: string;
  bitrate: number;
  /** "Renamed" | "Moved" | "Skipped" */
  status: string;
  timestamp: string;
  session_id: string;
}

export interface ProcessResult {
  records: RenameRecord[];
  errors: string[];
  session_id: string;
}

export interface CopyResult {
  copied: number;
  skipped: number;
  errors: string[];
}

export type AppPhase =
  | 'idle'
  | 'scanning'
  | 'review'
  | 'processing'
  | 'done'
  | 'history';