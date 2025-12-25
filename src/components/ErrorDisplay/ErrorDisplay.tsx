import { useAppSelector } from "../../store/hooks";
import { selectError } from "../../store/download/selectors";
import "./ErrorDisplay.scss";

export const ErrorDisplay = () => {
  const error = useAppSelector(selectError);

  if (!error) {
    return null;
  }

  return (
    <div className="app__error">
      <p className="app__error-text">{error}</p>
    </div>
  );
};

