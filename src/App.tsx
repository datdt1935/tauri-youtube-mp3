import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import { UrlInput } from "./components/UrlInput/UrlInput";
import { Progress } from "./components/Progress/Progress";
import { History } from "./components/History/History";
import "./App.scss";

type DownloadHistory = {
  url: string;
  title?: string;
  output_path: string;
  bitrate: number;
  timestamp: string;
  duration?: number;
};

type DownloadResult = {
  output_path: string;
  title?: string;
  duration?: number;
  file_size?: number;
};

type PlaylistDownloadResult = {
  output_folder: string;
  total_videos: number;
  downloaded_videos: DownloadResult[];
};

type DownloadResponse = 
  | ({ type: "Single" } & DownloadResult)
  | ({ type: "Playlist" } & PlaylistDownloadResult);

type AppPreferences = {
  output_folder: string | null;
  bitrate: number | null;
  last_url: string | null;
};

type DownloadProgressEvent = {
  overall_progress: number;
  current_song: number | null;
  total_songs: number | null;
  song_progress: number;
  status: string;
  current_title: string | null;
};

export const App = () => {
  const [youtubeUrl, setYoutubeUrl] = useState<string>("");
  const [outputFolder, setOutputFolder] = useState<string | null>(null);
  const [bitrate, setBitrate] = useState<number>(192);
  const [isDownloading, setIsDownloading] = useState<boolean>(false);
  const [downloadProgress, setDownloadProgress] = useState<number>(0);
  const [downloadStatus, setDownloadStatus] = useState<string>("");
  const [isPlaylist, setIsPlaylist] = useState<boolean>(false);
  const [currentSong, setCurrentSong] = useState<number | null>(null);
  const [totalSongs, setTotalSongs] = useState<number | null>(null);
  const [songProgress, setSongProgress] = useState<number>(0);
  const [currentTitle, setCurrentTitle] = useState<string | null>(null);
  const [history, setHistory] = useState<DownloadHistory[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadHistory();
    loadPreferences();

    // Listen for download progress events
    const progressUnlisten = listen<DownloadProgressEvent>("download-progress", (event) => {
      const progress = event.payload;
      setDownloadProgress(progress.overall_progress);
      setDownloadStatus(progress.status);
      
      if (progress.current_song !== null && progress.total_songs !== null) {
        setIsPlaylist(true);
        setCurrentSong(progress.current_song);
        setTotalSongs(progress.total_songs);
        setSongProgress(progress.song_progress);
        if (progress.current_title) {
          setCurrentTitle(progress.current_title);
        }
      } else {
        setIsPlaylist(false);
      }
    });

    return () => {
      progressUnlisten.then(unlisten => unlisten());
    };
  }, []);

  const loadHistory = async () => {
    try {
      const historyData = await invoke<DownloadHistory[]>("get_download_history");
      setHistory(historyData);
    } catch (err) {
      console.error("Failed to load history:", err);
    }
  };

  const loadPreferences = async () => {
    try {
      const prefs = await invoke<AppPreferences>("get_preferences");
      if (prefs.output_folder) {
        setOutputFolder(prefs.output_folder);
      }
      if (prefs.bitrate !== null && prefs.bitrate !== undefined) {
        setBitrate(prefs.bitrate);
      }
      if (prefs.last_url) {
        setYoutubeUrl(prefs.last_url);
      }
      console.log("Loaded preferences:", prefs);
    } catch (err) {
      console.error("Failed to load preferences:", err);
    }
  };

  const savePreferences = async () => {
    try {
      await invoke("save_preferences", {
        outputFolder: outputFolder || null,
        bitrate: bitrate || null,
        lastUrl: youtubeUrl.trim() || null,
      });
      console.log("Saved preferences:", { outputFolder, bitrate, lastUrl: youtubeUrl });
    } catch (err) {
      console.error("Failed to save preferences:", err);
      // Don't show error to user, it's not critical
    }
  };

  const handleUrlChange = (url: string) => {
    setYoutubeUrl(url);
    setError(null);
    console.log("URL changed:", url, "Output folder:", outputFolder, "Can download:", Boolean(url.trim() && outputFolder));
  };

  const handleOutputFolderSelect = async (folderPath: string) => {
    setOutputFolder(folderPath);
    setError(null);
    console.log("Output folder selected:", folderPath, "URL:", youtubeUrl, "Can download:", Boolean(youtubeUrl.trim() && folderPath));
    
    // Save preferences when output folder changes
    await savePreferences();
  };

  const handleBitrateChange = async (newBitrate: number) => {
    setBitrate(newBitrate);
    // Save preferences when bitrate changes
    await savePreferences();
  };

  const handleDownload = async () => {
    console.log("handleDownload called", { youtubeUrl, outputFolder, bitrate });
    
    if (!youtubeUrl.trim() || !outputFolder) {
      const errorMsg = "Please enter a YouTube URL and select an output folder";
      console.error(errorMsg);
      setError(errorMsg);
      return;
    }

    console.log("Starting download process...");
    setIsDownloading(true);
    setDownloadProgress(0);
    setDownloadStatus("Starting download...");
    setError(null);
    
    // Check if it's a playlist
    const isPlaylistUrl = youtubeUrl.includes("list=");
    setIsPlaylist(isPlaylistUrl);
    if (!isPlaylistUrl) {
      setCurrentSong(null);
      setTotalSongs(null);
      setSongProgress(0);
      setCurrentTitle(null);
    }

    try {
      console.log("Calling download_from_youtube command...", {
        url: youtubeUrl.trim(),
        outputFolder: outputFolder,
        bitrate: bitrate,
      });

      const result = await invoke<DownloadResponse>("download_from_youtube", {
        url: youtubeUrl.trim(),
        outputFolder: outputFolder,
        bitrate: bitrate,
      });

      console.log("Download result:", result);

      setDownloadProgress(100);
      
      if (result.type === "Playlist") {
        setDownloadStatus(`Download complete! Downloaded ${result.downloaded_videos.length} videos from playlist.`);
        setSongProgress(100);
      } else {
        setDownloadStatus("Download complete!");
        setIsPlaylist(false);
      }
      
      // Save preferences after successful download (keeps URL, folder, and bitrate for next time)
      await savePreferences();
      
      await loadHistory();

      setTimeout(() => {
        setIsDownloading(false);
        setDownloadProgress(0);
        setDownloadStatus("");
        setIsPlaylist(false);
        setCurrentSong(null);
        setTotalSongs(null);
        setSongProgress(0);
        setCurrentTitle(null);
      }, 2000);
    } catch (err) {
      console.error("Download error:", err);
      const error = err as Error | { toString?: () => string; message?: string };
      let errorMsg = error?.toString?.() || error?.message || "Download failed";
      
      errorMsg = String(errorMsg).replace(/\\n/g, '\n');
      
      setError(errorMsg);
      setIsDownloading(false);
      setDownloadProgress(0);
      setDownloadStatus("");
      setIsPlaylist(false);
      setCurrentSong(null);
      setTotalSongs(null);
      setSongProgress(0);
      setCurrentTitle(null);
    }
  };

  const handleClearHistory = async () => {
    try {
      await invoke("clear_history");
      setHistory([]);
    } catch (err) {
      console.error("Failed to clear history:", err);
    }
  };

  return (
    <div className="app">
      <main className="app__main">
        <div className="app__content">
          <UrlInput
            url={youtubeUrl}
            outputFolder={outputFolder}
            bitrate={bitrate}
            onUrlChange={handleUrlChange}
            onOutputFolderSelect={handleOutputFolderSelect}
            onBitrateChange={handleBitrateChange}
            onDownload={handleDownload}
            disabled={isDownloading}
          />

          {error && (
            <div className="app__error">
              <p className="app__error-text">{error}</p>
            </div>
          )}

          {isDownloading && (
            <Progress
              progress={downloadProgress}
              status={downloadStatus}
              isPlaylist={isPlaylist}
              currentSong={currentSong || undefined}
              totalSongs={totalSongs || undefined}
              songProgress={songProgress}
              currentTitle={currentTitle || undefined}
            />
          )}

          <History
            history={history}
            onClear={handleClearHistory}
          />
        </div>
      </main>
    </div>
  );
};

