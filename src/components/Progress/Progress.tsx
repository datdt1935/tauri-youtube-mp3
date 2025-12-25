import { useAppSelector } from "../../store/hooks";
import {
  selectIsDownloading,
  selectDownloadProgress,
  selectDownloadStatus,
  selectIsPlaylist,
  selectCurrentSong,
  selectTotalSongs,
  selectSongProgress,
  selectCurrentTitle,
} from "../../store/download/selectors";
import "./Progress.scss";

export const Progress = () => {
  const isDownloading = useAppSelector(selectIsDownloading);
  const progress = useAppSelector(selectDownloadProgress);
  const status = useAppSelector(selectDownloadStatus);
  const isPlaylist = useAppSelector(selectIsPlaylist);
  const currentSong = useAppSelector(selectCurrentSong);
  const totalSongs = useAppSelector(selectTotalSongs);
  const songProgress = useAppSelector(selectSongProgress);
  const currentTitle = useAppSelector(selectCurrentTitle);

  if (!isDownloading) {
    return null;
  }
  return (
    <div className="progress">
      <div className="progress__header">
        <h3 className="progress__title">
          {isPlaylist ? "Downloading Playlist..." : "Downloading..."}
        </h3>
        <span className="progress__percentage">{Math.round(progress)}%</span>
      </div>
      
      {/* Overall progress bar */}
      <div className="progress__bar-container">
        <div
          className="progress__bar"
          style={{ width: `${progress}%` }}
        />
      </div>

      {/* Playlist summary info */}
      {isPlaylist && currentSong !== null && totalSongs !== null && (
        <div className="progress__playlist-info">
          <p className="progress__playlist-summary">
            Song {currentSong} of {totalSongs}
          </p>
        </div>
      )}

      {/* Current song progress bar (for playlists) */}
      {isPlaylist && currentSong !== null && totalSongs !== null && (
        <div className="progress__song-section">
          <div className="progress__song-header">
            <span className="progress__song-title">
              {currentTitle || `Song ${currentSong}`}
            </span>
            <span className="progress__song-percentage">{Math.round(songProgress)}%</span>
          </div>
          <div className="progress__bar-container progress__bar-container--song">
            <div
              className="progress__bar progress__bar--song"
              style={{ width: `${songProgress}%` }}
            />
          </div>
        </div>
      )}

      {status && (
        <p className="progress__status">{status}</p>
      )}
    </div>
  );
};

