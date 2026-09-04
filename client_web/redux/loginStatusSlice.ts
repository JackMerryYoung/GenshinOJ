import { createSlice } from '@reduxjs/toolkit';

export const loginStatusSlice = createSlice({
    name: "loginStatus",
    // `undefined` means that a saved session is still being restored. Keeping this state
    // distinct from `false` prevents protected routes from redirecting before the WebSocket
    // handshake has had a chance to validate the persisted token.
    initialState: { value: undefined as boolean | undefined },
    reducers: {
        loginReducer: state => {
            state.value = true
        },
        logoutReducer: state => {
            state.value = false
        },
        restoreLoginReducer: state => {
            state.value = undefined
        }
    }
});

export const { loginReducer, logoutReducer, restoreLoginReducer } = loginStatusSlice.actions;

export default loginStatusSlice.reducer;
