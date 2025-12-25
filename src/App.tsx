import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
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

export const App = () => {
  const [youtubeUrl, setYoutubeUrl] = useState<string>("https://www.youtube.com/watch?v=UMMZWMbdv2w");
  const [outputFolder, setOutputFolder] = useState<string | null>("C:\\Users\\Admin\\Downloads");
  const [bitrate, setBitrate] = useState<number>(192);
  const [isDownloading, setIsDownloading] = useState<boolean>(false);
  const [downloadProgress, setDownloadProgress] = useState<number>(0);
  const [downloadStatus, setDownloadStatus] = useState<string>("");
  const [history, setHistory] = useState<DownloadHistory[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadHistory();
  }, []);

  const loadHistory = async () => {
    try {
      const historyData = await invoke<DownloadHistory[]>("get_download_history");
      setHistory(historyData);
    } catch (err) {
      console.error("Failed to load history:", err);
    }
  };

  const handleUrlChange = (url: string) => {
    setYoutubeUrl(url);
    setError(null);
    console.log("URL changed:", url, "Output folder:", outputFolder, "Can download:", Boolean(url.trim() && outputFolder));
  };

  const handleOutputFolderSelect = (folderPath: string) => {
    setOutputFolder(folderPath);
    setError(null);
    console.log("Output folder selected:", folderPath, "URL:", youtubeUrl, "Can download:", Boolean(youtubeUrl.trim() && folderPath));
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

    let progressInterval: number | null = null;

    try {
      // Simulate progress (yt-dlp doesn't provide easy real-time progress)
      progressInterval = setInterval(() => {
        setDownloadProgress((prev) => {
          if (prev >= 90) {
            if (progressInterval) clearInterval(progressInterval);
            return prev;
          }
          return prev + 3;
        });
      }, 1000);

      console.log("Calling download_from_youtube command...", {
        url: youtubeUrl.trim(),
        outputFolder: outputFolder,
        bitrate: bitrate,
      });

      const result = await invoke<DownloadResult>("download_from_youtube", {
        url: youtubeUrl.trim(),
        outputFolder: outputFolder,
        bitrate: bitrate,
      });

      console.log("Download result:", result);

      if (progressInterval) clearInterval(progressInterval);
      setDownloadProgress(100);
      setDownloadStatus("Download complete!");
      
      // Clear URL
      setYoutubeUrl("");
      
      // Reload history
      await loadHistory();

      setTimeout(() => {
        setIsDownloading(false);
        setDownloadProgress(0);
        setDownloadStatus("");
      }, 2000);
    } catch (err: any) {
      console.error("Download error:", err);
      let errorMsg = err?.toString() || err?.message || "Download failed";
      
      // Format error message for better display (preserve newlines)
      errorMsg = errorMsg.replace(/\\n/g, '\n');
      
      setError(errorMsg);
      if (progressInterval) clearInterval(progressInterval);
      setIsDownloading(false);
      setDownloadProgress(0);
      setDownloadStatus("");
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
      {/* <header className="app__header">
        <h1 className="app__title">YouTube Downloader</h1>
        <p className="app__subtitle">Download and convert YouTube videos to MP3</p>
      </header> */}

      <main className="app__main">
        <div className="app__content">
          <UrlInput
            url={youtubeUrl}
            outputFolder={outputFolder}
            bitrate={bitrate}
            onUrlChange={handleUrlChange}
            onOutputFolderSelect={handleOutputFolderSelect}
            onBitrateChange={setBitrate}
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

