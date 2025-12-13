export interface BufferSummary {
  id: string;
  title: string;
  preview: string;
  updated_at: number;
  is_pinned: boolean;
}

export interface SearchResult {
  id: string;
  snippet: string;
  updated_at: number;
}

export interface Buffer {
  id: string;
  content: string;
}

export interface AppSettings {
  font_family: string;
  font_size: number;
  line_height: number;
}
