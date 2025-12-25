import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { UrlInput } from "./components/UrlInput/UrlInput";
import { Progress } from "./components/Progress/Progress";
import { History } from "./components/History/History";
import { ErrorDisplay } from "./components/ErrorDisplay/ErrorDisplay";
import { useAppDispatch } from "./store/hooks";
import { downloadActions } from "./store/download";
import type { DownloadProgressEvent } from "./store/download/types";
import "./App.scss";

export const App = () => {
  const dispatch = useAppDispatch();

  useEffect(() => {
    dispatch(downloadActions.loadHistory());
    dispatch(downloadActions.loadPreferences());

    const progressUnlisten = listen<DownloadProgressEvent>("download-progress", (event) => {
      dispatch(downloadActions.updateDownloadProgress(event.payload));
    });

    return () => {
      progressUnlisten.then(unlisten => unlisten());
    };
  }, [dispatch]);

  return (
    <div className="app">
      <main className="app__main">
        <div className="app__content">
          <UrlInput />
          <ErrorDisplay />
          <Progress />
          <History />
        </div>
      </main>
    </div>
  );
};
