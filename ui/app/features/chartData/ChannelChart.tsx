import React from 'react'
import { useDispatch } from 'react-redux'
import { fetchDataRequest, fetchChannelRequest } from './actions'
import { channelDataType, chartDataType } from './types'
import { LineChart, ScatterChart, Scatter, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';
import moment from 'moment';
import Select, { ActionMeta, MultiValue } from 'react-select'

import { useAppSelector, useAppDispatch } from '../../hooks'

interface DataProps {
    timeframeMinutes: number,
}

export const ChannelChart: React.FC<DataProps> = props => {

    interface optionType {
        value: number,
        label: string,
    }

    const chartDataState = useAppSelector((state) => state.chartData);
    const chartData: chartDataType[] = chartDataState.chartData;
    const dispatch = useDispatch();

    const [selectedChannels, setSelectedChannels] = React.useState<optionType[]>();

    const channelOptions: optionType[] = chartDataState.channels?.map((row: channelDataType) => {
        return { value: row.channel_id, label: row.channel_name }
    });

    const onInputChange = (
        value: MultiValue<optionType>, action: ActionMeta<optionType>
    ) => {
        //console.log(value);
        setSelectedChannels(value as optionType[]);
    };

    const chartColors = ["#33658A", "#86BBD8", "#2F4858", "#F6AE2D", "#F26419"]

    let max: number = (Date.now());
    let min: number = max - props.timeframeMinutes * 60 * 1000;

    return (
        <div>
            <div>
                <div>
                    <Select
                        isMulti
                        name="channelsSelected"
                        className="basic-multi-select"
                        classNamePrefix="select"
                        options={channelOptions}
                        onChange={onInputChange} />
                </div>
                <div className="mt-4">
                    <ResponsiveContainer width="95%" height={400}>
                        <ScatterChart
                            margin={{
                                top: 5,
                                right: 30,
                                left: 20,
                                bottom: 5,
                            }}
                        >
                            <CartesianGrid strokeDasharray="3 3" />

                            {selectedChannels?.map((chnl, index) => (
                                <XAxis
                                    name="Time"
                                    tickFormatter={time => moment(time).format('HH:mm:ss')}
                                    dataKey="timestamp"
                                    xAxisId={chnl.label}
                                    domain={[min, max]}
                                    allowDataOverflow={true}
                                    type="number"
                                    hide={index == 0 ? false : true}
                                    key={index}
                                />))}

                            <YAxis />
                            <Legend />
                            {selectedChannels?.map((chnl, index) => (
                                <Scatter key={index} fill={chartColors[chnl.value % 5]} xAxisId={chnl.label} name={chnl.label} type="monotone" data={chartData.filter(data => data.channel == chnl.value)} dataKey="value" />
                            ))}

                        </ScatterChart>
                    </ResponsiveContainer>
                </div>
            </div>
        </div>
    )
}