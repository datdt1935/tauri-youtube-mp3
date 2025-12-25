import { configureStore } from '@reduxjs/toolkit';
import { downloadReducer } from './download';

export const store = configureStore({
  reducer: {
    download: downloadReducer,
  },
});

export type RootState = ReturnType<typeof store.getState>;
export type AppDispatch = typeof store.dispatch;

