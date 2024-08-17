import { configureStore } from '@reduxjs/toolkit';

import loginStatusReducer from '../redux/loginStatusSlice.ts';
import sessionTokenReducer from '../redux/sessionTokenSlice.ts';
import loginUsernameReducer from '../redux/loginUsernameSlice.ts';

const store = configureStore({
    reducer: {
        loginStatus: loginStatusReducer,
        sessionToken: sessionTokenReducer,
        loginUsername: loginUsernameReducer
    },
});

export default store;

export type RootState = ReturnType<typeof store.getState>;

export type AppDispatch = typeof store.dispatch;