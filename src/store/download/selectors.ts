import { createSelector } from '@reduxjs/toolkit';
import type { RootState } from '../index';
import type { DownloadState } from './types';

export const selectDownloadState = (state: RootState): DownloadState => state.download;

export const selectYoutubeUrl = createSelector(
  [selectDownloadState],
  (download) => download.youtubeUrl
);

export const selectOutputFolder = createSelector(
  [selectDownloadState],
  (download) => download.outputFolder
);

export const selectBitrate = createSelector(
  [selectDownloadState],
  (download) => download.bitrate
);

export const selectIsDownloading = createSelector(
  [selectDownloadState],
  (download) => download.isDownloading
);

export const selectDownloadProgress = createSelector(
  [selectDownloadState],
  (download) => download.downloadProgress
);

export const selectDownloadStatus = createSelector(
  [selectDownloadState],
  (download) => download.downloadStatus
);

export const selectIsPlaylist = createSelector(
  [selectDownloadState],
  (download) => download.isPlaylist
);

export const selectCurrentSong = createSelector(
  [selectDownloadState],
  (download) => download.currentSong
);

export const selectTotalSongs = createSelector(
  [selectDownloadState],
  (download) => download.totalSongs
);

export const selectSongProgress = createSelector(
  [selectDownloadState],
  (download) => download.songProgress
);

export const selectCurrentTitle = createSelector(
  [selectDownloadState],
  (download) => download.currentTitle
);

export const selectHistory = createSelector(
  [selectDownloadState],
  (download) => download.history
);

export const selectError = createSelector(
  [selectDownloadState],
  (download) => download.error
);

