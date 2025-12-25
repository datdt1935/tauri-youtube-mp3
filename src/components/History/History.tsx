import { useAppDispatch, useAppSelector } from "../../store/hooks";
import { selectHistory } from "../../store/download/selectors";
import { downloadActions } from "../../store/download";
import "./History.scss";

export const History = () => {
  const dispatch = useAppDispatch();
  const history = useAppSelector(selectHistory);

  const handleClear = () => {
    dispatch(downloadActions.clearHistory());
  };
  const formatDate = (timestamp: string) => {
    try {
      const date = new Date(timestamp);
      return date.toLocaleString();
    } catch {
      return timestamp;
    }
  };

  const formatDuration = (seconds?: number) => {
    if (!seconds) return "N/A";
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, "0")}`;
  };

  if (history.length === 0) {
    return null;
  }

  return (
    <div className="history">
      <div className="history__header">
        <h2 className="history__title">Download History</h2>
        <button className="history__clear-btn" onClick={handleClear}>
          Clear History
        </button>
      </div>
      <div className="history__list">
        {history.slice().reverse().map((item, index) => (
          <div key={index} className="history__item">
            <div className="history__item-content">
              <div className="history__item-main">
                <span className="history__title-text">
                  {item.title || item.url}
                </span>
                <span className="history__bitrate">{item.bitrate} kbps</span>
              </div>
              <div className="history__item-meta">
                <span className="history__timestamp">{formatDate(item.timestamp)}</span>
                {item.duration && (
                  <span className="history__duration">
                    Duration: {formatDuration(item.duration)}
                  </span>
                )}
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

