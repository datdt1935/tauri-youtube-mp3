import "./Progress.scss";

type ProgressProps = {
  progress: number;
  status: string;
};

export const Progress = ({ progress, status }: ProgressProps) => {
  return (
    <div className="progress">
      <div className="progress__header">
        <h3 className="progress__title">Downloading...</h3>
        <span className="progress__percentage">{progress}%</span>
      </div>
      <div className="progress__bar-container">
        <div
          className="progress__bar"
          style={{ width: `${progress}%` }}
        />
      </div>
      {status && (
        <p className="progress__status">{status}</p>
      )}
    </div>
  );
};

