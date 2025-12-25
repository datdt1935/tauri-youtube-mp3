import { useEffect } from "react";
import { open } from "@tauri-apps/api/dialog";
import "./UrlInput.scss";

type UrlInputProps = {
  url: string;
  outputFolder: string | null;
  bitrate: number;
  onUrlChange: (url: string) => void;
  onOutputFolderSelect: (folderPath: string) => void;
  onBitrateChange: (bitrate: number) => void;
  onDownload?: () => void;
  disabled?: boolean;
};

export const UrlInput = ({ 
  url, 
  outputFolder,
  bitrate,
  onUrlChange, 
  onOutputFolderSelect,
  onBitrateChange,
  onDownload, 
  disabled = false 
}: UrlInputProps) => {
  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onUrlChange(e.target.value);
  };

  const handleKeyPress = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter" && onDownload && url.trim() && outputFolder && !disabled) {
      onDownload();
    }
  };

  const handleSelectFolder = async () => {
    try {
      console.log("Opening folder dialog...");
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Output Folder",
      });
      
      console.log("Dialog result:", selected);
      
      if (selected) {
        const folderPath = Array.isArray(selected) ? selected[0] : selected;
        console.log("Folder selected:", folderPath);
        if (folderPath) {
          onOutputFolderSelect(folderPath);
          console.log("Output folder set to:", folderPath);
        }
      } else {
        console.log("No folder selected (user cancelled)");
      }
    } catch (err) {
      console.error("Failed to select folder:", err);
    }
  };

  const canDownload = Boolean(url.trim() && outputFolder && !disabled);
  
  // Debug: Log state changes
  useEffect(() => {
    console.log("UrlInput state:", { 
      url: url.trim(), 
      outputFolder, 
      disabled, 
      canDownload,
      hasOnDownload: !!onDownload 
    });
  }, [url, outputFolder, disabled, canDownload, onDownload]);

  return (
    <div className="url-input">
      <h2 className="url-input__title">YouTube Downloader</h2>
      <div className="url-input__content">
        <div className="url-input__section">
          <label className="url-input__label">YouTube URL</label>
          <input
            type="text"
            className="url-input__field"
            placeholder="https://www.youtube.com/watch?v=..."
            value={url}
            onChange={handleChange}
            onKeyPress={handleKeyPress}
            disabled={disabled}
          />
        </div>

        <div className="url-input__section">
          <label className="url-input__label">Bitrate (kbps)</label>
          <div className="url-input__bitrate-options">
            {[128, 192, 320].map((option) => (
              <button
                key={option}
                type="button"
                className={`url-input__bitrate-btn ${
                  bitrate === option ? "url-input__bitrate-btn--active" : ""
                }`}
                onClick={() => onBitrateChange(option)}
                disabled={disabled}
              >
                {option}
              </button>
            ))}
          </div>
        </div>

        <div className="url-input__section">
          <label className="url-input__label">Output Folder</label>
          {outputFolder ? (
            <div className="url-input__folder-selected">
              <span className="url-input__folder-path">{outputFolder}</span>
              <button
                type="button"
                className="url-input__change-btn"
                onClick={handleSelectFolder}
                disabled={disabled}
              >
                Change
              </button>
            </div>
          ) : (
            <button
              type="button"
              className="url-input__select-btn"
              onClick={handleSelectFolder}
              disabled={disabled}
            >
              Choose Output Folder
            </button>
          )}
        </div>

        <button
          type="button"
          className="url-input__download-btn"
          onClick={async (e) => {
            e.preventDefault();
            e.stopPropagation();
            console.log("Start Download button clicked", { 
              disabled, 
              canDownload, 
              hasOnDownload: !!onDownload,
              url: url.trim(),
              outputFolder,
              urlLength: url.trim().length,
              hasOutputFolder: !!outputFolder
            });
            
            if (!onDownload) {
              console.error("onDownload handler is not provided!");
              alert("Download handler is not available. Please refresh the app.");
              return;
            }
            
            if (!canDownload) {
              console.warn("Download blocked - requirements not met:", {
                hasUrl: !!url.trim(),
                hasFolder: !!outputFolder,
                isDisabled: disabled
              });
              alert("Please enter a YouTube URL and select an output folder");
              return;
            }
            
            try {
              console.log("Calling onDownload handler...");
              await onDownload();
              console.log("onDownload handler completed");
            } catch (error) {
              console.error("Error in onDownload handler:", error);
            }
          }}
          disabled={!canDownload || !onDownload}
          title={canDownload ? "Start Download" : "Enter URL and select output folder"}
        >
          Start Download
        </button>
      </div>
      {!canDownload && (
        <p className="url-input__hint">
          {!url.trim() && !outputFolder ? "Enter a YouTube URL and select an output folder" : ""}
          {!url.trim() && outputFolder ? "Enter a YouTube URL to continue" : ""}
          {url.trim() && !outputFolder ? "Select an output folder to continue" : ""}
        </p>
      )}
    </div>
  );
};

