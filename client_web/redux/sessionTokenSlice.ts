import { createSlice } from '@reduxjs/toolkit';

export const sessionTokenSlice = createSlice({
    name: "sessionToken",
    initialState: { value: "" },
    reducers: {
        modifySessionTokenReducer: (_state, action) => { return { value: action.payload }; },
        clearSessionTokenReducer: (_state) => { return { value: "" }; }
    }
});

export const { modifySessionTokenReducer, clearSessionTokenReducer } = sessionTokenSlice.actions;

export default sessionTokenSlice.reducer;