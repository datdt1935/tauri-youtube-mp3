import { createSlice, type PayloadAction } from '@reduxjs/toolkit';
import type { DownloadState, DownloadHistory, DownloadProgressEvent } from './types';
import { createExtendedActions } from './actions';

const initialState: DownloadState = {
  youtubeUrl: "",
  outputFolder: null,
  bitrate: 192,
  isDownloading: false,
  downloadProgress: 0,
  downloadStatus: "",
  isPlaylist: false,
  currentSong: null,
  totalSongs: null,
  songProgress: 0,
  currentTitle: null,
  history: [],
  error: null,
};

const slice = createSlice({
  name: 'download',
  initialState,
  reducers: {
    setYoutubeUrl: (state, action: PayloadAction<string>) => {
      state.youtubeUrl = action.payload;
      state.error = null;
    },
    setOutputFolder: (state, action: PayloadAction<string | null>) => {
      state.outputFolder = action.payload;
      state.error = null;
    },
    setBitrate: (state, action: PayloadAction<number>) => {
      state.bitrate = action.payload;
    },
    setIsDownloading: (state, action: PayloadAction<boolean>) => {
      state.isDownloading = action.payload;
    },
    setDownloadProgress: (state, action: PayloadAction<number>) => {
      state.downloadProgress = action.payload;
    },
    setDownloadStatus: (state, action: PayloadAction<string>) => {
      state.downloadStatus = action.payload;
    },
    setIsPlaylist: (state, action: PayloadAction<boolean>) => {
      state.isPlaylist = action.payload;
    },
    setCurrentSong: (state, action: PayloadAction<number | null>) => {
      state.currentSong = action.payload;
    },
    setTotalSongs: (state, action: PayloadAction<number | null>) => {
      state.totalSongs = action.payload;
    },
    setSongProgress: (state, action: PayloadAction<number>) => {
      state.songProgress = action.payload;
    },
    setCurrentTitle: (state, action: PayloadAction<string | null>) => {
      state.currentTitle = action.payload;
    },
    setHistory: (state, action: PayloadAction<DownloadHistory[]>) => {
      state.history = action.payload;
    },
    setError: (state, action: PayloadAction<string | null>) => {
      state.error = action.payload;
    },
    updateDownloadProgress: (state, action: PayloadAction<DownloadProgressEvent>) => {
      const progress = action.payload;
      state.downloadProgress = progress.overall_progress;
      state.downloadStatus = progress.status;
      
      if (progress.current_song !== null && progress.total_songs !== null) {
        state.isPlaylist = true;
        state.currentSong = progress.current_song;
        state.totalSongs = progress.total_songs;
        state.songProgress = progress.song_progress;
        if (progress.current_title) {
          state.currentTitle = progress.current_title;
        }
      } else {
        state.isPlaylist = false;
      }
    },
    resetPlaylistState: (state) => {
      state.currentSong = null;
      state.totalSongs = null;
      state.songProgress = 0;
      state.currentTitle = null;
    },
    resetDownloadState: (state) => {
      state.isDownloading = false;
      state.downloadProgress = 0;
      state.downloadStatus = "";
      state.isPlaylist = false;
      state.currentSong = null;
      state.totalSongs = null;
      state.songProgress = 0;
      state.currentTitle = null;
    },
  },
});

export const { actions } = slice;
export const downloadReducer = slice.reducer;
const extendActions = createExtendedActions(actions);
export const downloadActions = { ...actions, ...extendActions };

