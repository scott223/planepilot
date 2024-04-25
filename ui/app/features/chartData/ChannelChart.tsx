import React, { useState } from 'react'
import { useSelector, useDispatch } from 'react-redux'
import { fetchDataRequest, fetchChannelRequest } from './actions'
import { channelDataType, chartDataType, chartStateType } from './types'
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';
import moment from 'moment';
import Select from 'react-select'

import { useAppSelector, useAppDispatch } from '../../hooks'

interface DataProps {

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
        value: optionType[], action: string
    ) => {
        console.log(value);
        setSelectedChannels(value);
    };

    return (
        <div>
            <div>
                <Select
                    isMulti
                    name="channelsSelected"
                    className="basic-multi-select"
                    classNamePrefix="select"
                    options={channelOptions}
                    onChange={onInputChange} />

                <button
                    aria-label="Add chart data"
                    onClick={() => dispatch(fetchDataRequest())}
                >
                    Add chart data
                </button>

                <button
                    aria-label="Add channel data"
                    onClick={() => dispatch(fetchChannelRequest())}
                >
                    Add channel data
                </button>

                <LineChart
                    width={500}
                    height={300}
                    margin={{
                        top: 5,
                        right: 30,
                        left: 20,
                        bottom: 5,
                    }}
                >
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis
                        domain={['auto', 'auto']}
                        name="Time"
                        tickFormatter={time => moment(time).format('MM/DD HH:mm')}
                        dataKey="timestamp"
                    />
                    <YAxis />
                    < Tooltip labelFormatter={time => moment(time).format('DD/MM HH:mm:SS')} />
                    <Legend />
                    {selectedChannels?.map(chnl => (
                        <Line name={chnl.label} type="monotone" data={chartData.filter(data => data.channel == chnl.value)} dataKey="value" />
                    ))}

                </LineChart>


            </div>

        </div>

    )
}