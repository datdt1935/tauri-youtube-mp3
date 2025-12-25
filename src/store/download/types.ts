export type DownloadHistory = {
  url: string;
  title?: string;
  output_path: string;
  bitrate: number;
  timestamp: string;
  duration?: number;
};

export type DownloadResult = {
  output_path: string;
  title?: string;
  duration?: number;
  file_size?: number;
};

export type PlaylistDownloadResult = {
  output_folder: string;
  total_videos: number;
  downloaded_videos: DownloadResult[];
};

export type DownloadResponse = 
  | ({ type: "Single" } & DownloadResult)
  | ({ type: "Playlist" } & PlaylistDownloadResult);

export type AppPreferences = {
  output_folder: string | null;
  bitrate: number | null;
  last_url: string | null;
};

export type DownloadProgressEvent = {
  overall_progress: number;
  current_song: number | null;
  total_songs: number | null;
  song_progress: number;
  status: string;
  current_title: string | null;
};

export type DownloadState = {
  youtubeUrl: string;
  outputFolder: string | null;
  bitrate: number;
  isDownloading: boolean;
  downloadProgress: number;
  downloadStatus: string;
  isPlaylist: boolean;
  currentSong: number | null;
  totalSongs: number | null;
  songProgress: number;
  currentTitle: string | null;
  history: DownloadHistory[];
  error: string | null;
};

