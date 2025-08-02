import { createSlice } from '@reduxjs/toolkit';

export const loginStatusSlice = createSlice({
    name: "loginStatus",
    initialState: { value: false },
    reducers: {
        loginReducer: state => {
            state.value = true
        },
        logoutReducer: state => {
            state.value = false
        }
    }
});

export const { loginReducer, logoutReducer } = loginStatusSlice.actions;

export default loginStatusSlice.reducer;