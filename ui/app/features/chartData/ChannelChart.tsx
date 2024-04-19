import React, { useState } from 'react'
import { useSelector, useDispatch } from 'react-redux'
import { fetchDataRequest, fetchChannelRequest } from './actions'
import { channelDataType, chartDataType, chartStateType } from './types'
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';
import moment from 'moment';
import Select from 'react-select'

import { useAppSelector, useAppDispatch } from '../../hooks'

interface DataProps {
    channel: number,
}

interface optionType {
    value: number,
    label: string,
}

export const ChannelChart: React.FC<DataProps> = props => {
    const { channel } = props;

    const chartDataState = useAppSelector((state) => state.chartData);
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
                    onClick={() => dispatch(fetchDataRequest(channel))}
                >
                    Add chart data
                </button>

                <button
                    aria-label="Add channel data"
                    onClick={() => dispatch(fetchChannelRequest())}
                >
                    Add channel data
                </button>

            </div>

        </div>

    )
}