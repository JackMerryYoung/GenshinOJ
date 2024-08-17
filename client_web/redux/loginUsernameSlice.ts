import { createSlice } from '@reduxjs/toolkit';

export const loginUsernameSlice = createSlice({
    name: "loginUsername",
    initialState: { value: "" },
    reducers: {
        modifyLoginUsernameReducer: (state, action) => { state.value = action.payload; },
        clearLoginUsernameReducer: (state) => { state.value = ""; }
    }
});

export const { modifyLoginUsernameReducer, clearLoginUsernameReducer } = loginUsernameSlice.actions;

export default loginUsernameSlice.reducer;