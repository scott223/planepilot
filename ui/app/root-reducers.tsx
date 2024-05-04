import chartDataReducer from './features/chartData/reducers'
import { chartStateType } from './features/chartData/types';

export type StateType = {
    chartDataState: chartStateType,
};

const rootReducers = {
    chartData: chartDataReducer,
};

export default rootReducers;