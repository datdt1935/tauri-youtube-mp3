import { invoke } from "@tauri-apps/api/tauri";
import "./FileImport.scss";

type FileImportProps = {
  selectedFile: string | null;
  onFileSelect: (filePath: string) => void;
  disabled?: boolean;
};

export const FileImport = ({ selectedFile, onFileSelect, disabled = false }: FileImportProps) => {
  const handleSelectFile = async () => {
    try {
      const filePath = await invoke<string | null>("select_file");
      if (filePath) {
        onFileSelect(filePath);
      }
    } catch (err) {
      console.error("Failed to select file:", err);
    }
  };

  const getFileName = (path: string) => {
    return path.split(/[/\\]/).pop() || path;
  };

  return (
    <div className="file-import">
      <h2 className="file-import__title">Select File</h2>
      <div className="file-import__content">
        {selectedFile ? (
          <div className="file-import__selected">
            <span className="file-import__file-name">{getFileName(selectedFile)}</span>
            <button
              className="file-import__change-btn"
              onClick={handleSelectFile}
              disabled={disabled}
            >
              Change File
            </button>
          </div>
        ) : (
          <button
            className="file-import__select-btn"
            onClick={handleSelectFile}
            disabled={disabled}
          >
            Choose File
          </button>
        )}
      </div>
    </div>
  );
};

