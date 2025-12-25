import { invoke } from '@tauri-apps/api/tauri';
import type { AppDispatch } from '../index';
import type { DownloadHistory, AppPreferences, DownloadResponse } from './types';
import type { actions } from './index';

type SliceActions = typeof actions;

export const createExtendedActions = (sliceActions: SliceActions) => {
  const loadHistory = () => {
    return async (dispatch: AppDispatch) => {
      try {
        const historyData = await invoke<DownloadHistory[]>("get_download_history");
        dispatch(sliceActions.setHistory(historyData));
      } catch (err) {
        console.error("Failed to load history:", err);
      }
    };
  };

  const loadPreferences = () => {
    return async (dispatch: AppDispatch) => {
      try {
        const prefs = await invoke<AppPreferences>("get_preferences");
        if (prefs.output_folder) {
          dispatch(sliceActions.setOutputFolder(prefs.output_folder));
        }
        if (prefs.bitrate !== null && prefs.bitrate !== undefined) {
          dispatch(sliceActions.setBitrate(prefs.bitrate));
        }
        if (prefs.last_url) {
          dispatch(sliceActions.setYoutubeUrl(prefs.last_url));
        }
        console.log("Loaded preferences:", prefs);
      } catch (err) {
        console.error("Failed to load preferences:", err);
      }
    };
  };

  const savePreferences = (youtubeUrl: string, outputFolder: string | null, bitrate: number) => {
    return async () => {
      try {
        await invoke("save_preferences", {
          outputFolder: outputFolder || null,
          bitrate: bitrate || null,
          lastUrl: youtubeUrl.trim() || null,
        });
        console.log("Saved preferences:", { outputFolder, bitrate, lastUrl: youtubeUrl });
      } catch (err) {
        console.error("Failed to save preferences:", err);
      }
    };
  };

  const downloadFromYoutube = (url: string, outputFolder: string, bitrate: number) => {
    return async (dispatch: AppDispatch) => {
      if (!url.trim() || !outputFolder) {
        const errorMsg = "Please enter a YouTube URL and select an output folder";
        console.error(errorMsg);
        dispatch(sliceActions.setError(errorMsg));
        return;
      }

      console.log("Starting download process...");
      dispatch(sliceActions.setIsDownloading(true));
      dispatch(sliceActions.setDownloadProgress(0));
      dispatch(sliceActions.setDownloadStatus("Starting download..."));
      dispatch(sliceActions.setError(null));
      
      const isPlaylistUrl = url.includes("list=");
      dispatch(sliceActions.setIsPlaylist(isPlaylistUrl));
      if (!isPlaylistUrl) {
        dispatch(sliceActions.resetPlaylistState());
      }

      try {
        console.log("Calling download_from_youtube command...", {
          url: url.trim(),
          outputFolder: outputFolder,
          bitrate: bitrate,
        });

        const result = await invoke<DownloadResponse>("download_from_youtube", {
          url: url.trim(),
          outputFolder: outputFolder,
          bitrate: bitrate,
        });

        console.log("Download result:", result);

        dispatch(sliceActions.setDownloadProgress(100));
        
        if (result.type === "Playlist") {
          dispatch(sliceActions.setDownloadStatus(`Download complete! Downloaded ${result.downloaded_videos.length} videos from playlist.`));
          dispatch(sliceActions.setSongProgress(100));
        } else {
          dispatch(sliceActions.setDownloadStatus("Download complete!"));
          dispatch(sliceActions.setIsPlaylist(false));
        }
        
        await savePreferences(url, outputFolder, bitrate)();
        await loadHistory()(dispatch);

        setTimeout(() => {
          dispatch(sliceActions.resetDownloadState());
        }, 2000);
      } catch (err) {
        console.error("Download error:", err);
        const error = err as Error | { toString?: () => string; message?: string };
        let errorMsg = error?.toString?.() || error?.message || "Download failed";
        
        errorMsg = String(errorMsg).replace(/\\n/g, '\n');
        
        dispatch(sliceActions.setError(errorMsg));
        dispatch(sliceActions.resetDownloadState());
      }
    };
  };

  const clearHistory = () => {
    return async (dispatch: AppDispatch) => {
      try {
        await invoke("clear_history");
        dispatch(sliceActions.setHistory([]));
      } catch (err) {
        console.error("Failed to clear history:", err);
      }
    };
  };

  return {
    loadHistory,
    loadPreferences,
    savePreferences,
    downloadFromYoutube,
    clearHistory,
  };
};

