import { createSlice } from '@reduxjs/toolkit';

export const loginUsernameSlice = createSlice({
    name: "loginUsername",
    initialState: { value: "" },
    reducers: {
        modifyLoginUsernameReducer: (_state, action) => { return { value: action.payload }; },
        clearLoginUsernameReducer: (_state) => { return { value: "" }; }
    }
});

export const { modifyLoginUsernameReducer, clearLoginUsernameReducer } = loginUsernameSlice.actions;

export default loginUsernameSlice.reducer;