import { createSlice } from '@reduxjs/toolkit';

export const sessionTokenSlice = createSlice({
    name: "sessionToken",
    initialState: { value: "" },
    reducers: {
        modifySessionTokenReducer: (state, action) => { state.value = action.payload; },
        clearSessionTokenReducer: (state) => { state.value = ""; }
    }
});

export const { modifySessionTokenReducer, clearSessionTokenReducer } = sessionTokenSlice.actions;

export default sessionTokenSlice.reducer;